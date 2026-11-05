//! Runs the game: the opponent process, the flag timer and autosave. The
//! decisions live in `ActiveGame`; this is what happens around them.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use gb_archive::Archive;
use gb_chess::ChessError;
use gb_engine::{EngineError, SearchOutcome, Strength, UciEngine};
use gb_model::{Color, ColorChoice, GameView, MoveMark, NewGameOptions, ReplayFrame};
use tauri::async_runtime::{spawn, JoinHandle};
use tokio::sync::watch;
use tokio::time::sleep;

use crate::active_game::{ActiveGame, ResumeState};
use crate::civil_date::pgn_date;
use crate::events::EventSink;
use crate::move_review::{MoveReviewer, ReviewJob};
use crate::state::lock;

const NO_GAME: &str = "no game in progress";
const FINISHED_GAME: &str = "this game is already finished";

struct Opponent {
    engine: UciEngine,
    path: PathBuf,
}

#[derive(Default)]
struct Inner {
    active: Option<ActiveGame>,
    flag_timer: Option<JoinHandle<()>>,
}

pub struct Session {
    inner: Mutex<Inner>,
    opponent: tokio::sync::Mutex<Option<Opponent>>,
    /// Bumped whenever a running search stops mattering: its reply is dropped.
    generation: watch::Sender<u64>,
    /// Which game a late measurement belongs to.
    game_id: AtomicU64,
    /// `None` when the player's moves are not being measured.
    review_engine: Mutex<Option<PathBuf>>,
    reviewer: MoveReviewer,
    origin: Instant,
    archive: Arc<Mutex<Archive>>,
    sink: Arc<dyn EventSink>,
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

fn pick_color(choice: ColorChoice) -> Color {
    match choice {
        ColorChoice::White => Color::White,
        ColorChoice::Black => Color::Black,
        ColorChoice::Random => {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.subsec_nanos());

            if nanos.is_multiple_of(2) {
                Color::White
            } else {
                Color::Black
            }
        }
    }
}

fn message(error: ChessError) -> String {
    error.to_string()
}

impl Session {
    pub fn new(archive: Arc<Mutex<Archive>>, sink: Arc<dyn EventSink>) -> Arc<Self> {
        let (generation, _) = watch::channel(0);

        Arc::new_cyclic(|session| Self {
            inner: Mutex::new(Inner::default()),
            opponent: tokio::sync::Mutex::new(None),
            generation,
            game_id: AtomicU64::new(0),
            review_engine: Mutex::new(None),
            reviewer: MoveReviewer::start(session.clone()),
            origin: Instant::now(),
            archive,
            sink,
        })
    }

    fn now_ms(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    fn bump_generation(&self) -> u64 {
        self.generation.send_modify(|value| *value += 1);

        *self.generation.borrow()
    }

    pub fn current_view(&self) -> Option<GameView> {
        let now = self.now_ms();

        lock(&self.inner)
            .active
            .as_ref()
            .map(|active| active.view(now))
    }

    pub fn frames(&self) -> Result<Vec<ReplayFrame>, String> {
        lock(&self.inner)
            .active
            .as_ref()
            .ok_or_else(|| NO_GAME.to_owned())?
            .frames()
            .map_err(message)
    }

    async fn prepare_opponent(
        &self,
        path: &Path,
        strength: Strength,
    ) -> Result<String, EngineError> {
        let mut slot = self.opponent.lock().await;

        if slot.as_ref().is_none_or(|opponent| opponent.path != path) {
            *slot = Some(Opponent {
                engine: UciEngine::start(path).await?,
                path: path.to_owned(),
            });
        }

        let Some(opponent) = slot.as_mut() else {
            return Err(EngineError::Closed);
        };

        opponent.engine.set_strength(strength).await?;
        opponent.engine.new_game().await?;

        Ok(opponent.engine.info().name)
    }

    /// Drops the process after a failure, so the next game starts a fresh one.
    async fn discard_opponent(&self) {
        *self.opponent.lock().await = None;
    }

    /// Writes the game as it stands. A game with no moves left has no row.
    fn save_game(&self, active: &mut ActiveGame, now_ms: u64) {
        let seconds = unix_seconds();
        let played_at = u32::try_from(seconds).unwrap_or(u32::MAX);
        let archive = lock(&self.archive);

        let Some(record) = active.record(now_ms, played_at, pgn_date(seconds)) else {
            if let Some(id) = active.forget_saved() {
                if let Err(error) = archive.delete(id) {
                    tracing::error!("cannot delete the game: {error}");
                }
            }

            return;
        };
        let saved = match active.archive_id() {
            Some(id) => archive.update(id, &record).map(|()| id),
            None => archive.save(&record),
        };

        match saved {
            Ok(id) => active.mark_saved(id),
            Err(error) => tracing::error!("cannot save the game: {error}"),
        }
    }

    /// Stops the game on the board: it stays in the archive, to be played on.
    pub fn suspend(&self) {
        let now = self.now_ms();
        let mut inner = lock(&self.inner);

        self.bump_generation();

        if let Some(handle) = inner.flag_timer.take() {
            handle.abort();
        }

        if let Some(mut active) = inner.active.take() {
            if !active.is_over() {
                self.save_game(&mut active, now);
            }
        }

        drop(inner);

        *lock(&self.review_engine) = None;
        self.game_id.fetch_add(1, Ordering::SeqCst);
    }

    pub async fn resume_game(
        self: &Arc<Self>,
        id: u32,
        engine_path: &Path,
    ) -> Result<GameView, String> {
        let text = lock(&self.archive)
            .resume_state(id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| FINISHED_GAME.to_owned())?;
        let state: ResumeState = serde_json::from_str(&text).map_err(|error| error.to_string())?;
        let is_training = state.is_training();
        let strength = state.opponent_elo().map_or(Strength::Full, Strength::Elo);

        self.suspend();

        if let Err(error) = self.prepare_opponent(engine_path, strength).await {
            self.discard_opponent().await;

            return Err(error.to_string());
        }

        let active = ActiveGame::resume(state, id, self.now_ms()).map_err(message)?;

        lock(&self.inner).active = Some(active);
        *lock(&self.review_engine) = is_training.then(|| engine_path.to_owned());

        self.after_change().ok_or_else(|| NO_GAME.to_owned())
    }

    pub async fn new_game(
        self: &Arc<Self>,
        options: NewGameOptions,
        engine_path: &Path,
    ) -> Result<GameView, String> {
        self.suspend();

        let strength = options.elo.map_or(Strength::Full, Strength::Elo);
        let opponent_name = match self.prepare_opponent(engine_path, strength).await {
            Ok(name) => name,
            Err(error) => {
                self.discard_opponent().await;

                return Err(error.to_string());
            }
        };
        let human_color = pick_color(options.color);
        let active = ActiveGame::new(&options, human_color, opponent_name).map_err(message)?;

        lock(&self.inner).active = Some(active);
        *lock(&self.review_engine) = options.training.then(|| engine_path.to_owned());
        self.game_id.fetch_add(1, Ordering::SeqCst);

        self.after_change().ok_or_else(|| NO_GAME.to_owned())
    }

    /// Applies `change` to the game, then runs what must follow it.
    fn change<T>(
        self: &Arc<Self>,
        change: impl FnOnce(&mut ActiveGame, u64) -> Result<T, ChessError>,
    ) -> Result<(T, GameView), String> {
        let now = self.now_ms();
        let answer = {
            let mut inner = lock(&self.inner);
            let active = inner.active.as_mut().ok_or_else(|| NO_GAME.to_owned())?;

            change(active, now).map_err(message)?
        };
        let view = self.after_change().ok_or_else(|| NO_GAME.to_owned())?;

        Ok((answer, view))
    }

    pub fn make_move(self: &Arc<Self>, uci: &str) -> Result<GameView, String> {
        self.change(|active, now| active.human_move(uci, now))
            .map(|(_, view)| view)
    }

    pub fn set_premove(self: &Arc<Self>, uci: &str) -> Result<GameView, String> {
        self.change(|active, _| active.set_premove(uci))
            .map(|(_, view)| view)
    }

    pub fn clear_premove(self: &Arc<Self>) -> Result<GameView, String> {
        self.change(|active, _| {
            active.clear_premove();

            Ok(())
        })
        .map(|(_, view)| view)
    }

    pub fn take_back(self: &Arc<Self>) -> Result<GameView, String> {
        // Only a real takeback cancels the search: a refused one must not
        // leave the engine's first move hanging.
        self.change(|active, now| {
            active.take_back(now)?;
            self.bump_generation();

            Ok(())
        })
        .map(|(_, view)| view)
    }

    pub fn resign(self: &Arc<Self>) -> Result<GameView, String> {
        self.change(|active, now| {
            active.resign(now);

            Ok(())
        })
        .map(|(_, view)| view)
    }

    pub fn offer_draw(self: &Arc<Self>) -> Result<bool, String> {
        self.change(|active, now| Ok(active.offer_draw(now)))
            .map(|(accepted, _)| accepted)
    }

    /// The measurement of a move arrived, which may be long after the move.
    pub fn add_move_mark(&self, game_id: u64, uci: &str, mark: MoveMark) {
        if self.game_id.load(Ordering::SeqCst) != game_id {
            return;
        }

        let now = self.now_ms();
        let view = lock(&self.inner).active.as_mut().map(|active| {
            active.add_move_mark(uci, mark);
            active.view(now)
        });

        if let Some(view) = view {
            self.sink.game_updated(&view);
        }
    }

    fn submit_reviews(&self, active: &mut ActiveGame) {
        let played_moves = active.take_played_moves();
        let engine_path = lock(&self.review_engine).clone();
        let game_id = self.game_id.load(Ordering::SeqCst);

        if let Some(engine_path) = engine_path {
            for played in played_moves {
                self.reviewer.submit(ReviewJob {
                    game_id,
                    played,
                    engine_path: engine_path.clone(),
                });
            }
        }
    }

    /// What every change leads to: saving, the timer, the next search, the event.
    fn after_change(self: &Arc<Self>) -> Option<GameView> {
        let now = self.now_ms();
        let mut inner = lock(&self.inner);
        let timer = inner.flag_timer.take();

        if let Some(handle) = timer {
            handle.abort();
        }

        let active = inner.active.as_mut()?;
        let flag_wait = active.ms_to_flag(now);
        let mut search_generation = None;

        self.submit_reviews(active);

        if active.is_over() {
            self.bump_generation();
            active.set_thinking(false);
        } else if active.is_engine_turn() && !active.is_thinking() {
            active.set_thinking(true);
            search_generation = Some(*self.generation.borrow());
        }

        // Every move is written at once, so closing the app loses nothing.
        if !active.is_saved() {
            self.save_game(active, now);
        }

        let view = active.view(now);

        if let Some(wait_ms) = flag_wait {
            let session = Arc::clone(self);

            inner.flag_timer = Some(spawn(async move {
                sleep(Duration::from_millis(wait_ms)).await;
                session.on_flag_timer();
            }));
        }

        drop(inner);

        if let Some(generation) = search_generation {
            let session = Arc::clone(self);

            spawn(async move {
                session.run_search(generation).await;
            });
        }

        self.sink.game_updated(&view);

        Some(view)
    }

    fn on_flag_timer(self: &Arc<Self>) {
        let now = self.now_ms();

        {
            let mut inner = lock(&self.inner);

            if let Some(active) = inner.active.as_mut() {
                active.flag_if_due(now);
            }
        }

        self.after_change();
    }

    fn search_outcome_is_stale(&self, generation: u64) -> bool {
        *self.generation.borrow() != generation
    }

    async fn run_search(self: Arc<Self>, generation: u64) {
        let mut slot = self.opponent.lock().await;

        if self.search_outcome_is_stale(generation) {
            return;
        }

        let now = self.now_ms();
        let setup = lock(&self.inner).active.as_ref().map(|active| {
            (
                active.start_fen(),
                active.move_list(),
                active.search_limit(now),
            )
        });
        let (Some(opponent), Some((start_fen, moves, limit))) = (slot.as_mut(), setup) else {
            drop(slot);
            self.on_search_result(generation, Err(EngineError::Closed));

            return;
        };

        let mut watcher = self.generation.subscribe();
        let cancel = async move {
            // An error means the session is gone: nobody waits for the move.
            let _ = watcher.wait_for(|current| *current != generation).await;
        };
        let result = opponent
            .engine
            .search(start_fen.as_deref(), &moves, limit, cancel, |_| {})
            .await;

        drop(slot);
        self.on_search_result(generation, result);
    }

    fn on_search_result(
        self: &Arc<Self>,
        generation: u64,
        result: Result<SearchOutcome, EngineError>,
    ) {
        if self.search_outcome_is_stale(generation) {
            return;
        }

        let now = self.now_ms();

        {
            let mut inner = lock(&self.inner);
            let Some(active) = inner.active.as_mut() else {
                return;
            };
            let played = match &result {
                Ok(SearchOutcome {
                    best_move: Some(uci),
                    score,
                }) => active
                    .engine_move(uci, *score, now)
                    .and_then(|()| active.play_premove(now))
                    .map_err(message),
                Ok(_) => Err("the engine gave no move".to_owned()),
                Err(error) => Err(error.to_string()),
            };

            if let Err(error) = played {
                tracing::error!("opponent failed: {error}");
                active.fail_engine();
            }
        }

        self.after_change();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chosen_color_is_kept() {
        assert_eq!(pick_color(ColorChoice::White), Color::White);
        assert_eq!(pick_color(ColorChoice::Black), Color::Black);
    }
}
