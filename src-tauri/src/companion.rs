//! The conversation with the local model. Rust keeps it whole and the frontend
//! renders it. Every position fact comes from the rules code or the engine.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gb_chess::san_line;
use gb_engine::{evaluation_of, CandidateLine, EngineError, SearchInfo, UciEngine};
use gb_llm::{
    is_model_loaded, stream_chat, system_prompt, unload_model, ChatMessage, ChatRequest,
    EngineLine, LlmError, LoadedModel, PositionFacts, Speaker, ADVICE_REQUEST,
};
use gb_model::{ChatSpeaker, ChatTurn, CompanionView, LlmSettings};
use tauri::async_runtime::{spawn, JoinHandle};
use tokio::sync::watch;
use tokio::time::sleep;

use crate::events::EventSink;
use crate::state::lock;

const BUSY: &str = "the companion is still answering";
const NO_MODEL: &str = "choose a model in the settings";
const SILENT_MODEL: &str = "the model gave no answer";
/// What the model is reminded of with each question.
const PROMPT_TURNS: usize = 12;
/// What the chat keeps on screen.
const KEPT_TURNS: usize = 60;
const TURN_SEPARATOR: &str = "\n\n";
/// The idle timeout LM Studio gives a model it loads for a request.
const OWN_MODEL_IDLE: Duration = Duration::from_secs(600);
const CANDIDATE_LINES: u32 = 3;
const CANDIDATE_THINK_MS: u32 = 700;
const LINE_PLIES: usize = 5;
/// The depth of a measurement is not part of what the model is told.
const UNREPORTED_DEPTH: u32 = 0;

/// Everything a question needs, read before the reply starts.
pub struct Question {
    pub facts: PositionFacts,
    pub settings: LlmSettings,
    pub language: String,
    pub engine_path: Option<PathBuf>,
    /// The coach speaks by itself: no player turn stands behind this reply.
    pub is_advice: bool,
}

/// The engine's process for these measurements: the opponent is limited in
/// strength and the evaluation bar is switched off by a setting.
struct Measurer {
    engine: UciEngine,
    path: PathBuf,
}

pub struct Companion {
    view: Mutex<CompanionView>,
    /// Bumped by stop and reset: what the old reply still sends is dropped.
    generation: watch::Sender<u64>,
    measurer: tokio::sync::Mutex<Option<Measurer>>,
    /// Only a model this app loaded: the user's own are not ours to unload.
    own_model: Mutex<Option<LoadedModel>>,
    idle_timer: Mutex<Option<JoinHandle<()>>>,
    sink: Arc<dyn EventSink>,
}

fn add_turn(view: &mut CompanionView, speaker: ChatSpeaker, text: String) {
    view.turns.push(ChatTurn { speaker, text });

    let excess = view.turns.len().saturating_sub(KEPT_TURNS);

    view.turns.drain(..excess);
}

/// Ends the reply. Text that already arrived stays in the chat.
fn settle(view: &mut CompanionView, error: Option<String>) {
    let draft = view.draft.take().unwrap_or_default();

    view.loading_model = false;

    if !draft.trim().is_empty() {
        add_turn(view, ChatSpeaker::Companion, draft);
    }

    view.error = error;
}

/// The system prompt, then the recent turns, which end with the question.
/// Advice leaves the coach speaking several times in a row, and some models'
/// chat templates refuse that, so those turns merge into one.
fn chat_messages(system: String, turns: &[ChatTurn]) -> Vec<ChatMessage> {
    let recent = turns.len().saturating_sub(PROMPT_TURNS);
    let mut messages = vec![ChatMessage {
        role: Speaker::System,
        content: system,
    }];

    for turn in &turns[recent..] {
        let role = match turn.speaker {
            ChatSpeaker::Player => Speaker::User,
            ChatSpeaker::Companion => Speaker::Assistant,
        };

        match messages.last_mut() {
            Some(last) if last.role == role => {
                last.content.push_str(TURN_SEPARATOR);
                last.content.push_str(&turn.text);
            }
            _ => messages.push(ChatMessage {
                role,
                content: turn.text.clone(),
            }),
        }
    }

    messages
}

fn engine_line(fen: &str, candidate: &CandidateLine) -> Option<EngineLine> {
    let shown = candidate.moves.len().min(LINE_PLIES);
    let moves = san_line(fen, &candidate.moves[..shown]);
    let evaluation = evaluation_of(
        fen,
        SearchInfo {
            depth: UNREPORTED_DEPTH,
            score: candidate.score,
        },
    );

    (!moves.is_empty()).then_some(EngineLine {
        moves,
        white_centipawns: evaluation.centipawns,
        white_mate_in: evaluation.mate_in,
    })
}

impl Companion {
    pub fn new(sink: Arc<dyn EventSink>) -> Arc<Self> {
        let (generation, _) = watch::channel(0);

        Arc::new(Self {
            view: Mutex::new(CompanionView::default()),
            generation,
            measurer: tokio::sync::Mutex::new(None),
            own_model: Mutex::new(None),
            idle_timer: Mutex::new(None),
            sink,
        })
    }

    pub fn view(&self) -> CompanionView {
        lock(&self.view).clone()
    }

    fn change(&self, change: impl FnOnce(&mut CompanionView)) {
        let snapshot = {
            let mut view = lock(&self.view);

            change(&mut view);
            view.clone()
        };

        self.sink.companion_updated(&snapshot);
    }

    fn generation_now(&self) -> u64 {
        *self.generation.borrow()
    }

    fn is_stale(&self, generation: u64) -> bool {
        self.generation_now() != generation
    }

    fn bump_generation(&self) {
        self.generation.send_modify(|value| *value += 1);
    }

    pub fn ask(self: &Arc<Self>, text: String, question: Question) -> Result<(), String> {
        self.start(Some(text), question)
    }

    /// Advice waits for no one: while the coach is busy, it is simply skipped.
    pub fn advise(self: &Arc<Self>, question: Question) -> Result<(), String> {
        if lock(&self.view).draft.is_some() {
            return Ok(());
        }

        self.start(None, question)
    }

    fn start(
        self: &Arc<Self>,
        player_text: Option<String>,
        question: Question,
    ) -> Result<(), String> {
        if question.settings.model.is_none() {
            return Err(NO_MODEL.to_owned());
        }

        {
            let mut view = lock(&self.view);

            if view.draft.is_some() {
                return Err(BUSY.to_owned());
            }

            view.error = None;
            view.draft = Some(String::new());

            if let Some(text) = player_text {
                add_turn(&mut view, ChatSpeaker::Player, text);
            }
        }

        self.cancel_idle_timer();
        self.sink.companion_updated(&self.view());

        let companion = Arc::clone(self);
        let generation = self.generation_now();

        spawn(async move {
            let result = companion.reply(generation, question).await;

            companion.finish(generation, result);
            companion.start_idle_timer();
        });

        Ok(())
    }

    fn cancel_idle_timer(&self) {
        if let Some(timer) = lock(&self.idle_timer).take() {
            timer.abort();
        }
    }

    /// A later question cancels this; while one is answered it must not start.
    fn start_idle_timer(self: &Arc<Self>) {
        if lock(&self.own_model).is_none() || lock(&self.view).draft.is_some() {
            return;
        }

        let companion = Arc::clone(self);

        self.cancel_idle_timer();
        *lock(&self.idle_timer) = Some(spawn(async move {
            sleep(OWN_MODEL_IDLE).await;
            companion.unload_own_model().await;
        }));
    }

    async fn unload_own_model(&self) {
        let own_model = lock(&self.own_model).take();

        if let Some(model) = own_model {
            if let Err(error) = unload_model(&model).await {
                tracing::warn!("cannot unload the model: {error}");
            }
        }
    }

    /// For the app closing, or the connection being switched off.
    pub async fn release_model(&self) {
        self.cancel_idle_timer();
        self.unload_own_model().await;
    }

    /// The connection was switched off: nothing may stay running or in memory.
    pub fn switch_off(self: &Arc<Self>) {
        let companion = Arc::clone(self);

        self.stop();
        spawn(async move {
            companion.release_model().await;
        });
    }

    /// The reply stops; what it has said so far stays.
    pub fn stop(&self) {
        self.bump_generation();
        self.change(|view| settle(view, None));
    }

    /// A new game: the old conversation talks about another position.
    pub fn reset(&self) {
        self.bump_generation();
        self.change(|view| *view = CompanionView::default());
    }

    async fn measure_with(
        slot: &mut Option<Measurer>,
        path: &Path,
        fen: &str,
    ) -> Result<Vec<CandidateLine>, EngineError> {
        if slot.as_ref().is_none_or(|measurer| measurer.path != path) {
            *slot = Some(Measurer {
                engine: UciEngine::start(path).await?,
                path: path.to_owned(),
            });
        }

        let Some(measurer) = slot.as_mut() else {
            return Err(EngineError::Closed);
        };

        measurer
            .engine
            .candidates(fen, CANDIDATE_LINES, CANDIDATE_THINK_MS)
            .await
    }

    /// No measurement is an answer too: the prompt says so, and the model
    /// declines to judge rather than the chat failing.
    async fn measure(&self, path: &Path, fen: &str) -> Vec<EngineLine> {
        let mut slot = self.measurer.lock().await;

        match Self::measure_with(&mut slot, path, fen).await {
            Ok(candidates) => candidates
                .iter()
                .filter_map(|candidate| engine_line(fen, candidate))
                .collect(),
            Err(error) => {
                tracing::warn!("companion engine: {error}");
                *slot = None;

                Vec::new()
            }
        }
    }

    async fn reply(&self, generation: u64, question: Question) -> Result<(), LlmError> {
        let Question {
            mut facts,
            settings,
            language,
            engine_path,
            is_advice,
        } = question;

        if let (None, Some(path)) = (facts.outcome, engine_path) {
            facts.lines = self.measure(&path, &facts.fen).await;
        }

        let mut messages = chat_messages(system_prompt(&language, &facts), &self.view().turns);

        if is_advice {
            messages.push(ChatMessage {
                role: Speaker::User,
                content: ADVICE_REQUEST.to_owned(),
            });
        }

        let request = ChatRequest {
            base_url: settings.base_url,
            model: settings.model.unwrap_or_default(),
            messages,
        };

        if is_model_loaded(&request.base_url, &request.model).await == Some(false)
            && !self.is_stale(generation)
        {
            self.change(|view| view.loading_model = true);
        }

        let mut watcher = self.generation.subscribe();
        let cancel = async move {
            // An error means the companion is gone: nobody waits for the reply.
            let _ = watcher.wait_for(|current| *current != generation).await;
        };

        let remember_model = |model| *lock(&self.own_model) = Some(model);

        stream_chat(&request, cancel, remember_model, |piece| {
            if !self.is_stale(generation) {
                self.change(|view| {
                    view.loading_model = false;

                    if let Some(draft) = view.draft.as_mut() {
                        draft.push_str(piece);
                    }
                });
            }
        })
        .await
    }

    fn finish(&self, generation: u64, result: Result<(), LlmError>) {
        if self.is_stale(generation) {
            return;
        }

        self.change(|view| {
            let is_silent = view
                .draft
                .as_ref()
                .is_none_or(|draft| draft.trim().is_empty());
            let error = match result {
                Err(error) => Some(error.to_string()),
                Ok(()) if is_silent => Some(SILENT_MODEL.to_owned()),
                Ok(()) => None,
            };

            settle(view, error);
        });
    }
}

#[cfg(test)]
mod tests {
    use gb_engine::Score;

    use super::*;

    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    fn turn(speaker: ChatSpeaker, text: &str) -> ChatTurn {
        ChatTurn {
            speaker,
            text: text.to_owned(),
        }
    }

    #[test]
    fn the_prompt_starts_with_the_facts_and_keeps_recent_turns_only() {
        let turns: Vec<ChatTurn> = (0..PROMPT_TURNS + 3)
            .map(|index| {
                if index % 2 == 0 {
                    turn(ChatSpeaker::Player, "question")
                } else {
                    turn(ChatSpeaker::Companion, "answer")
                }
            })
            .collect();
        let messages = chat_messages("facts".to_owned(), &turns);

        assert_eq!(messages.len(), PROMPT_TURNS + 1);
        assert_eq!(messages[0].role, Speaker::System);
    }

    #[test]
    fn advice_in_a_row_reaches_the_model_as_one_turn() {
        let turns = [
            turn(ChatSpeaker::Companion, "first"),
            turn(ChatSpeaker::Companion, "second"),
            turn(ChatSpeaker::Player, "why?"),
        ];
        let messages = chat_messages("facts".to_owned(), &turns);

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[1].content, "first\n\nsecond");
        assert_eq!(messages[2].role, Speaker::User);
    }

    #[test]
    fn a_stopped_reply_keeps_its_words_and_an_empty_one_leaves_nothing() {
        let mut view = CompanionView {
            draft: Some("half a sen".to_owned()),
            ..CompanionView::default()
        };

        settle(&mut view, None);

        assert_eq!(view.draft, None);
        assert_eq!(view.turns[0].text, "half a sen");

        view.draft = Some("  ".to_owned());
        settle(&mut view, Some("broken".to_owned()));

        assert_eq!(view.turns.len(), 1);
        assert_eq!(view.error.as_deref(), Some("broken"));
    }

    #[test]
    fn the_chat_keeps_a_bounded_history() {
        let mut view = CompanionView::default();

        for index in 0..KEPT_TURNS + 5 {
            add_turn(&mut view, ChatSpeaker::Player, index.to_string());
        }

        assert_eq!(view.turns.len(), KEPT_TURNS);
        assert_eq!(view.turns[0].text, "5");
    }

    #[test]
    fn an_engine_line_is_white_pov_san_and_stops_at_nonsense() {
        let candidate = CandidateLine {
            score: Score::Centipawns(30),
            moves: vec![
                "e2e4".to_owned(),
                "e7e5".to_owned(),
                "e1e8".to_owned(),
                "g1f3".to_owned(),
            ],
        };
        let line = engine_line(START_FEN, &candidate).unwrap();

        assert_eq!(line.moves, vec!["e4", "e5"]);
        assert_eq!(line.white_centipawns, Some(30));
    }

    #[test]
    fn a_line_with_no_legal_first_move_is_dropped() {
        let candidate = CandidateLine {
            score: Score::Centipawns(0),
            moves: vec!["e1e8".to_owned()],
        };

        assert_eq!(engine_line(START_FEN, &candidate), None);
    }
}
