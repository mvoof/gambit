//! The game in progress: everything the session decides, with no I/O and no
//! async. The caller passes the time, so all of it is testable.

use std::mem::take;

use gb_archive::NewGame;
use gb_chess::{
    export_pgn, format_time_control, ChessError, Clock, ClockSnapshot, Game, GameExtras, PgnHeaders,
};
use gb_engine::{accepts_draw, Score, SearchLimit};
use gb_model::{
    Color, GameResult, GameView, MoveMark, MoveOption, NewGameOptions, ReplayFrame, TimeControl,
};
use serde::{Deserialize, Serialize};

/// The opponent's thinking time when there is no clock.
const UNLIMITED_MOVE_TIME_MS: u32 = 1_500;
const HUMAN_NAME: &str = "Player";
const PLIES_IN_A_ROUND: usize = 2;
const UNFINISHED_RESULT: &str = "*";

/// A move of the player, waiting for the engine to measure it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayedMove {
    pub ply: u32,
    pub color: Color,
    pub uci: String,
    pub fen_before: String,
    pub fen_after: String,
}

/// All it takes to play on later, stored with the unfinished game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeState {
    start_fen: Option<String>,
    moves: Vec<String>,
    human_color: Color,
    opponent_name: String,
    opponent_elo: Option<u32>,
    time_control: Option<TimeControl>,
    is_training: bool,
    /// The clocks at the moment the game was stopped.
    clock: Option<ClockSnapshot>,
    clock_history: Vec<ClockSnapshot>,
    move_marks: Vec<MoveMark>,
}

impl ResumeState {
    pub fn opponent_elo(&self) -> Option<u32> {
        self.opponent_elo
    }

    pub fn is_training(&self) -> bool {
        self.is_training
    }
}

pub struct ActiveGame {
    game: Game,
    human_color: Color,
    opponent_name: String,
    opponent_elo: Option<u32>,
    time_control: Option<TimeControl>,
    is_training: bool,
    clock: Option<Clock>,
    /// The clock as each ply began; the first entry is the start of the game.
    clock_history: Vec<ClockSnapshot>,
    premove: Option<MoveOption>,
    played_moves: Vec<PlayedMove>,
    move_marks: Vec<MoveMark>,
    engine_score: Option<Score>,
    engine_thinking: bool,
    engine_failed: bool,
    /// Its row in the archive, once it has one.
    archive_id: Option<u32>,
    /// The plies and the end of the game as last written to the archive.
    saved_state: (usize, bool),
}

fn clamp_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

impl ActiveGame {
    pub fn new(
        options: &NewGameOptions,
        human_color: Color,
        opponent_name: String,
    ) -> Result<Self, ChessError> {
        let clock = options.time_control.map(Clock::new);

        Ok(Self {
            game: Game::new(options.start_fen.as_deref())?,
            human_color,
            opponent_name,
            opponent_elo: options.elo,
            time_control: options.time_control,
            is_training: options.training,
            clock_history: clock.iter().map(|start| start.snapshot(0)).collect(),
            clock,
            premove: None,
            played_moves: Vec::new(),
            move_marks: Vec::new(),
            engine_score: None,
            engine_thinking: false,
            engine_failed: false,
            archive_id: None,
            saved_state: (0, false),
        })
    }

    /// The game as it was stopped. The clocks run from `now_ms`.
    pub fn resume(state: ResumeState, archive_id: u32, now_ms: u64) -> Result<Self, ChessError> {
        let mut game = Game::new(state.start_fen.as_deref())?;

        for uci in &state.moves {
            game.play_uci(uci)?;
        }

        let clock = state.time_control.map(Clock::new).map(|mut clock| {
            if let Some(snapshot) = state.clock {
                clock.restore(snapshot, now_ms);
            }

            clock
        });
        let saved_state = (game.ply_count(), false);

        Ok(Self {
            game,
            human_color: state.human_color,
            opponent_name: state.opponent_name,
            opponent_elo: state.opponent_elo,
            time_control: state.time_control,
            is_training: state.is_training,
            clock,
            clock_history: state.clock_history,
            premove: None,
            played_moves: Vec::new(),
            move_marks: state.move_marks,
            engine_score: None,
            engine_thinking: false,
            engine_failed: false,
            archive_id: Some(archive_id),
            saved_state,
        })
    }

    pub fn is_over(&self) -> bool {
        self.game.outcome().is_some()
    }

    pub fn is_engine_turn(&self) -> bool {
        !self.is_over() && !self.engine_failed && self.game.turn() != self.human_color
    }

    pub fn is_thinking(&self) -> bool {
        self.engine_thinking
    }

    pub fn set_thinking(&mut self, thinking: bool) {
        self.engine_thinking = thinking;
    }

    pub fn fail_engine(&mut self) {
        self.engine_thinking = false;
        self.engine_failed = true;
    }

    fn stop_clock_if_over(&mut self, now_ms: u64) {
        if self.is_over() {
            self.premove = None;

            if let Some(clock) = self.clock.as_mut() {
                clock.stop(now_ms);
            }
        }
    }

    /// `true` when the side to move has no time left, and the game is now over.
    fn flag_mover_if_out_of_time(&mut self, now_ms: u64) -> bool {
        let mover = self.game.turn();
        let is_out = self
            .clock
            .as_ref()
            .and_then(|clock| clock.ms_to_flag(now_ms))
            .is_some_and(|(running, left)| running == mover && left == 0);

        if is_out {
            self.game.flag_fall(mover);
            self.stop_clock_if_over(now_ms);
        }

        is_out
    }

    fn play(&mut self, uci: &str, now_ms: u64) -> Result<(), ChessError> {
        if self.flag_mover_if_out_of_time(now_ms) {
            return Ok(());
        }

        let mover = self.game.turn();

        self.game.play_uci(uci)?;

        if let Some(clock) = self.clock.as_mut() {
            clock.on_move(mover, now_ms);
            self.clock_history.push(clock.snapshot(now_ms));
        }

        self.stop_clock_if_over(now_ms);

        Ok(())
    }

    pub fn human_move(&mut self, uci: &str, now_ms: u64) -> Result<(), ChessError> {
        if self.game.turn() != self.human_color {
            return Err(ChessError::IllegalMove(format!("{uci}: not your turn")));
        }

        self.premove = None;

        let fen_before = self.game.fen();
        let plies_before = self.game.ply_count();

        self.play(uci, now_ms)?;

        // Out of time, the move was not played.
        if self.game.ply_count() > plies_before {
            self.played_moves.push(PlayedMove {
                ply: clamp_u32(self.game.ply_count() as u64),
                color: self.human_color,
                uci: uci.to_owned(),
                fen_before,
                fen_after: self.game.fen(),
            });
        }

        Ok(())
    }

    pub fn take_played_moves(&mut self) -> Vec<PlayedMove> {
        take(&mut self.played_moves)
    }

    /// A takeback may have replaced the move while it was being measured.
    pub fn add_move_mark(&mut self, uci: &str, mark: MoveMark) {
        let moves = self.game.moves();
        let is_current = (mark.ply as usize)
            .checked_sub(1)
            .and_then(|index| moves.get(index))
            .is_some_and(|record| record.uci == uci);

        if is_current {
            self.move_marks.push(mark);
        }
    }

    pub fn engine_move(
        &mut self,
        uci: &str,
        score: Option<Score>,
        now_ms: u64,
    ) -> Result<(), ChessError> {
        self.engine_thinking = false;
        self.engine_score = score;
        self.play(uci, now_ms)
    }

    /// Stores a premove. It must be one of the offered options.
    pub fn set_premove(&mut self, uci: &str) -> Result<(), ChessError> {
        if self.is_over() || self.game.turn() == self.human_color {
            return Err(ChessError::IllegalMove(format!(
                "{uci}: no premove on your turn"
            )));
        }

        let option = self
            .game
            .premove_options(self.human_color)
            .into_iter()
            .find(|option| option.uci == uci)
            .ok_or_else(|| ChessError::IllegalMove(uci.to_owned()))?;

        self.premove = Some(option);

        Ok(())
    }

    pub fn clear_premove(&mut self) {
        self.premove = None;
    }

    /// Plays the stored premove if it turned out legal; drops it either way.
    pub fn play_premove(&mut self, now_ms: u64) -> Result<(), ChessError> {
        let Some(option) = self.premove.take() else {
            return Ok(());
        };

        if self.is_over() || !self.game.is_legal_uci(&option.uci) {
            return Ok(());
        }

        self.human_move(&option.uci, now_ms)
    }

    fn plies_to_take_back(&self) -> usize {
        if self.game.turn() == self.human_color {
            PLIES_IN_A_ROUND
        } else {
            1
        }
    }

    fn can_take_back(&self) -> bool {
        !self.is_over() && self.game.ply_count() >= self.plies_to_take_back()
    }

    /// Back to the human's last decision, with the clock of that ply.
    pub fn take_back(&mut self, now_ms: u64) -> Result<(), ChessError> {
        if !self.can_take_back() {
            return Err(ChessError::NothingToUndo);
        }

        for _ in 0..self.plies_to_take_back() {
            self.game.undo()?;
            self.clock_history.pop();
        }

        if let (Some(clock), Some(snapshot)) = (self.clock.as_mut(), self.clock_history.last()) {
            clock.restore(*snapshot, now_ms);
        }

        let ply_count = self.game.ply_count();

        self.move_marks
            .retain(|mark| (mark.ply as usize) <= ply_count);
        self.premove = None;
        self.engine_thinking = false;

        Ok(())
    }

    pub fn resign(&mut self, now_ms: u64) {
        self.game.resign(self.human_color);
        self.stop_clock_if_over(now_ms);
    }

    /// The engine agrees when its own last score says it is not better.
    pub fn offer_draw(&mut self, now_ms: u64) -> bool {
        let accepted = !self.is_over() && accepts_draw(self.engine_score);

        if accepted {
            self.game.agree_draw();
            self.stop_clock_if_over(now_ms);
        }

        accepted
    }

    pub fn ms_to_flag(&self, now_ms: u64) -> Option<u64> {
        self.clock
            .as_ref()
            .and_then(|clock| clock.ms_to_flag(now_ms))
            .map(|(_, left)| left)
    }

    /// For the flag timer: the running side may be either player.
    pub fn flag_if_due(&mut self, now_ms: u64) {
        let flagged = self
            .clock
            .as_ref()
            .and_then(|clock| clock.ms_to_flag(now_ms))
            .filter(|(_, left)| *left == 0)
            .map(|(color, _)| color);

        if let Some(color) = flagged {
            self.game.flag_fall(color);
            self.stop_clock_if_over(now_ms);
        }
    }

    pub fn search_limit(&self, now_ms: u64) -> SearchLimit {
        match (&self.clock, self.time_control) {
            (Some(clock), Some(control)) => SearchLimit::Clock {
                white_ms: clamp_u32(clock.remaining_ms(Color::White, now_ms)),
                black_ms: clamp_u32(clock.remaining_ms(Color::Black, now_ms)),
                white_increment_ms: control.increment_ms,
                black_increment_ms: control.increment_ms,
            },
            _ => SearchLimit::MoveTime(UNLIMITED_MOVE_TIME_MS),
        }
    }

    pub fn start_fen(&self) -> Option<String> {
        self.game.custom_start_fen().map(str::to_owned)
    }

    pub fn move_list(&self) -> Vec<String> {
        self.game
            .moves()
            .into_iter()
            .map(|record| record.uci)
            .collect()
    }

    pub fn frames(&self) -> Result<Vec<ReplayFrame>, ChessError> {
        self.game.frames()
    }

    pub fn view(&self, now_ms: u64) -> GameView {
        self.game.view(GameExtras {
            human_color: self.human_color,
            opponent_name: self.opponent_name.clone(),
            opponent_elo: self.opponent_elo,
            premove: self.premove.clone(),
            move_marks: self.move_marks.clone(),
            clock: self.clock.as_ref().map(|clock| clock.view(now_ms)),
            engine_thinking: self.engine_thinking,
            engine_failed: self.engine_failed,
            can_takeback: self.can_take_back(),
            is_training: self.is_training,
            archive_id: self.archive_id,
        })
    }

    fn players(&self) -> (String, String) {
        match self.human_color {
            Color::White => (HUMAN_NAME.to_owned(), self.opponent_name.clone()),
            Color::Black => (self.opponent_name.clone(), HUMAN_NAME.to_owned()),
        }
    }

    fn current_state(&self) -> (usize, bool) {
        (self.game.ply_count(), self.is_over())
    }

    pub fn is_saved(&self) -> bool {
        self.saved_state == self.current_state()
    }

    pub fn archive_id(&self) -> Option<u32> {
        self.archive_id
    }

    pub fn mark_saved(&mut self, archive_id: u32) {
        self.archive_id = Some(archive_id);
        self.saved_state = self.current_state();
    }

    /// Nothing is left to keep: its row, if it had one, should go.
    pub fn forget_saved(&mut self) -> Option<u32> {
        self.saved_state = self.current_state();
        self.archive_id.take()
    }

    fn resume_state(&self, now_ms: u64) -> ResumeState {
        ResumeState {
            start_fen: self.start_fen(),
            moves: self.move_list(),
            human_color: self.human_color,
            opponent_name: self.opponent_name.clone(),
            opponent_elo: self.opponent_elo,
            time_control: self.time_control,
            is_training: self.is_training,
            clock: self.clock.as_ref().map(|clock| clock.snapshot(now_ms)),
            clock_history: self.clock_history.clone(),
            move_marks: self.move_marks.clone(),
        }
    }

    /// The game as the archive should hold it: `None` before the first move.
    pub fn record(&self, now_ms: u64, played_at: u32, date: String) -> Option<NewGame> {
        if self.game.ply_count() == 0 {
            return None;
        }

        let outcome = self.game.outcome();
        let resume = outcome.is_none().then(|| self.resume_state(now_ms));
        let (white, black) = self.players();
        let pgn = export_pgn(
            &self.game,
            &PgnHeaders {
                white: white.clone(),
                black: black.clone(),
                date,
                time_control: self.time_control,
            },
        );
        let result = match outcome.map(|finished| finished.result) {
            Some(GameResult::WhiteWins) => "1-0",
            Some(GameResult::BlackWins) => "0-1",
            Some(GameResult::Draw) => "1/2-1/2",
            None => UNFINISHED_RESULT,
        };

        Some(NewGame {
            white,
            black,
            result: result.to_owned(),
            reason: outcome.map(|finished| finished.reason),
            played_at,
            ply_count: clamp_u32(self.game.ply_count() as u64),
            time_control: format_time_control(self.time_control),
            pgn,
            human_color: Some(self.human_color),
            opponent_elo: self.opponent_elo,
            final_fen: Some(self.game.fen()),
            resume: resume.and_then(|state| serde_json::to_string(&state).ok()),
        })
    }
}

#[cfg(test)]
mod review_tests {
    use gb_model::{ColorChoice, MoveQuality};

    use super::*;

    fn game() -> ActiveGame {
        let options = NewGameOptions {
            color: ColorChoice::White,
            time_control: None,
            elo: None,
            start_fen: None,
            training: true,
        };

        ActiveGame::new(&options, Color::White, "Fake".to_owned()).unwrap()
    }

    fn mark(ply: u32) -> MoveMark {
        MoveMark {
            ply,
            color: Color::White,
            quality: MoveQuality::Mistake,
            best_san: "d4".to_owned(),
        }
    }

    #[test]
    fn only_the_players_moves_wait_for_measuring() {
        let mut game = game();

        game.human_move("e2e4", 0).unwrap();
        game.engine_move("e7e5", None, 0).unwrap();

        let played = game.take_played_moves();

        assert_eq!(played.len(), 1);
        assert_eq!(played[0].ply, 1);
        assert_eq!(played[0].uci, "e2e4");
        assert_ne!(played[0].fen_before, played[0].fen_after);
        assert!(game.take_played_moves().is_empty());
    }

    #[test]
    fn a_mark_reaches_the_view_and_a_takeback_removes_it() {
        let mut game = game();

        game.human_move("e2e4", 0).unwrap();
        game.engine_move("e7e5", None, 0).unwrap();
        game.add_move_mark("e2e4", mark(1));

        assert_eq!(game.view(0).move_marks, vec![mark(1)]);

        game.take_back(0).unwrap();

        assert!(game.view(0).move_marks.is_empty());
    }

    #[test]
    fn a_late_mark_for_a_replaced_move_is_dropped() {
        let mut game = game();

        game.human_move("d2d4", 0).unwrap();
        game.add_move_mark("e2e4", mark(1));
        game.add_move_mark("d2d4", mark(5));

        assert!(game.view(0).move_marks.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use gb_model::{ColorChoice, GameEndReason, MoveQuality};

    use super::*;

    const MINUTE_MS: u32 = 60_000;
    const INCREMENT_MS: u32 = 1_000;

    fn options(time_control: Option<TimeControl>) -> NewGameOptions {
        NewGameOptions {
            color: ColorChoice::White,
            time_control,
            elo: None,
            start_fen: None,
            training: false,
        }
    }

    fn timed() -> Option<TimeControl> {
        Some(TimeControl {
            base_ms: MINUTE_MS,
            increment_ms: INCREMENT_MS,
        })
    }

    fn as_white(time_control: Option<TimeControl>) -> ActiveGame {
        ActiveGame::new(&options(time_control), Color::White, "Fake".to_owned()).unwrap()
    }

    fn reason(game: &ActiveGame) -> Option<GameEndReason> {
        game.view(0).outcome.map(|outcome| outcome.reason)
    }

    #[test]
    fn the_engine_opens_when_the_human_plays_black() {
        let game = ActiveGame::new(&options(None), Color::Black, "Fake".to_owned()).unwrap();

        assert!(game.is_engine_turn());
    }

    #[test]
    fn a_human_move_on_the_engine_turn_is_refused() {
        let mut game = as_white(None);

        game.human_move("e2e4", 0).unwrap();

        assert!(matches!(
            game.human_move("d2d4", 0),
            Err(ChessError::IllegalMove(_))
        ));
        assert!(game.is_engine_turn());
    }

    #[test]
    fn the_engine_gets_the_real_clock_or_a_move_time() {
        let mut game = as_white(timed());

        game.human_move("e2e4", 0).unwrap();

        assert_eq!(
            game.search_limit(1_000),
            SearchLimit::Clock {
                white_ms: MINUTE_MS,
                black_ms: MINUTE_MS - 1_000,
                white_increment_ms: INCREMENT_MS,
                black_increment_ms: INCREMENT_MS,
            }
        );
        assert_eq!(
            as_white(None).search_limit(0),
            SearchLimit::MoveTime(UNLIMITED_MOVE_TIME_MS)
        );
    }

    #[test]
    fn a_move_after_the_flag_loses_on_time_and_is_not_played() {
        let mut game = as_white(timed());

        game.human_move("e2e4", 0).unwrap();
        game.engine_move("e7e5", None, 1_000).unwrap();
        game.human_move("g1f3", 1_000 + u64::from(MINUTE_MS) + 1)
            .unwrap();

        assert_eq!(reason(&game), Some(GameEndReason::Timeout));
        assert_eq!(game.move_list().len(), 2);
    }

    #[test]
    fn the_flag_timer_ends_the_game_only_when_due() {
        let mut game = as_white(timed());

        game.human_move("e2e4", 0).unwrap();
        game.flag_if_due(1_000);

        assert_eq!(reason(&game), None);

        game.flag_if_due(u64::from(MINUTE_MS));

        assert_eq!(reason(&game), Some(GameEndReason::Timeout));
        assert_eq!(game.ms_to_flag(u64::from(MINUTE_MS)), None);
    }

    #[test]
    fn take_back_on_your_turn_undoes_a_full_round_with_its_clock() {
        let mut game = as_white(timed());

        game.human_move("e2e4", 0).unwrap();
        game.engine_move("e7e5", None, 2_000).unwrap();
        game.human_move("g1f3", 12_000).unwrap();
        game.engine_move("b8c6", None, 15_000).unwrap();

        let before = game.view(15_000).clock.unwrap();

        assert_eq!(before.white_ms, MINUTE_MS - 10_000 + INCREMENT_MS);

        game.take_back(20_000).unwrap();

        let restored = game.view(20_000);
        let clock = restored.clock.unwrap();

        assert_eq!(restored.moves.len(), 2);
        assert_eq!(clock.white_ms, MINUTE_MS);
        assert_eq!(clock.running, Some(Color::White));
    }

    #[test]
    fn take_back_while_the_engine_thinks_undoes_only_your_move() {
        let mut game = as_white(None);

        game.human_move("e2e4", 0).unwrap();
        game.set_thinking(true);
        game.take_back(0).unwrap();

        assert_eq!(game.move_list().len(), 0);
        assert!(!game.is_thinking());
    }

    #[test]
    fn take_back_needs_a_move_of_yours() {
        let mut game = ActiveGame::new(&options(None), Color::Black, "Fake".to_owned()).unwrap();

        game.engine_move("e2e4", None, 0).unwrap();

        assert!(!game.view(0).can_takeback);
        assert!(game.take_back(0).is_err());
    }

    #[test]
    fn a_legal_premove_is_played_after_the_reply() {
        let mut game = as_white(None);

        game.human_move("e2e4", 0).unwrap();
        game.set_premove("g1f3").unwrap();
        game.engine_move("e7e5", None, 0).unwrap();
        game.play_premove(0).unwrap();

        assert_eq!(game.move_list().last().map(String::as_str), Some("g1f3"));
        assert_eq!(game.view(0).premove, None);
    }

    #[test]
    fn a_premove_made_illegal_by_the_reply_is_dropped() {
        let mut game = as_white(None);

        game.human_move("e2e4", 0).unwrap();
        game.engine_move("e7e5", None, 0).unwrap();
        game.human_move("g1f3", 0).unwrap();
        game.set_premove("e4e5").unwrap();
        game.engine_move("b8c6", None, 0).unwrap();
        game.play_premove(0).unwrap();

        assert_eq!(game.move_list().len(), 4);
        assert_eq!(game.view(0).premove, None);
    }

    #[test]
    fn a_premove_is_only_taken_on_the_engine_turn_from_the_offered_list() {
        let mut game = as_white(None);

        assert!(game.set_premove("e2e4").is_err());

        game.human_move("e2e4", 0).unwrap();

        assert!(game.set_premove("a1h8").is_err());
    }

    #[test]
    fn the_engine_takes_a_draw_only_when_it_is_not_better() {
        let mut game = as_white(None);

        assert!(!game.offer_draw(0), "no score yet");

        game.human_move("e2e4", 0).unwrap();
        game.engine_move("e7e5", Some(Score::Centipawns(200)), 0)
            .unwrap();

        assert!(!game.offer_draw(0));

        game.human_move("g1f3", 0).unwrap();
        game.engine_move("b8c6", Some(Score::Centipawns(-50)), 0)
            .unwrap();

        assert!(game.offer_draw(0));
        assert_eq!(reason(&game), Some(GameEndReason::DrawAgreement));
    }

    #[test]
    fn a_failed_engine_stops_asking_for_its_move() {
        let mut game = as_white(None);

        game.human_move("e2e4", 0).unwrap();
        game.fail_engine();

        assert!(!game.is_engine_turn());
        assert!(game.view(0).engine_failed);
    }

    #[test]
    fn a_finished_game_gives_a_full_record_with_nothing_to_resume() {
        let mut game = as_white(timed());

        assert!(game.record(0, 1, "2026.11.05".to_owned()).is_none());

        game.human_move("e2e4", 0).unwrap();
        game.resign(0);

        let record = game.record(0, 1, "2026.11.05".to_owned()).unwrap();

        assert_eq!(record.result, "0-1");
        assert_eq!(record.white, HUMAN_NAME);
        assert_eq!(record.black, "Fake");
        assert_eq!(record.time_control, "60+1");
        assert_eq!(record.ply_count, 1);
        assert_eq!(record.human_color, Some(Color::White));
        assert_eq!(record.resume, None);
        assert!(record.final_fen.is_some());
        assert!(record.pgn.contains("[Date \"2026.11.05\"]"));
    }

    #[test]
    fn a_game_with_no_moves_has_no_record() {
        let mut game = as_white(None);

        game.resign(0);

        assert!(game.record(0, 1, "2026.11.05".to_owned()).is_none());
    }

    #[test]
    fn a_new_move_or_the_end_needs_saving_again() {
        let mut game = as_white(None);

        assert!(game.is_saved());

        game.human_move("e2e4", 0).unwrap();

        assert!(!game.is_saved());

        game.mark_saved(4);

        assert!(game.is_saved());
        assert_eq!(game.archive_id(), Some(4));

        game.resign(0);

        assert!(!game.is_saved());
    }

    #[test]
    fn taking_back_every_move_forgets_the_row() {
        let mut game = as_white(None);

        game.human_move("e2e4", 0).unwrap();
        game.mark_saved(4);
        game.take_back(0).unwrap();

        assert_eq!(game.forget_saved(), Some(4));
        assert!(game.is_saved());
        assert_eq!(game.archive_id(), None);
    }

    fn mark(ply: u32) -> MoveMark {
        MoveMark {
            ply,
            color: Color::White,
            quality: MoveQuality::Mistake,
            best_san: "d4".to_owned(),
        }
    }

    fn stopped_game(game: &ActiveGame, now_ms: u64) -> ResumeState {
        let record = game.record(now_ms, 1, "2026.11.05".to_owned()).unwrap();

        serde_json::from_str(&record.resume.unwrap()).unwrap()
    }

    #[test]
    fn an_unfinished_game_comes_back_as_it_was() {
        let mut game = as_white(timed());

        game.human_move("e2e4", 0).unwrap();
        game.engine_move("e7e5", None, 5_000).unwrap();
        game.add_move_mark("e2e4", mark(1));

        let record = game.record(15_000, 1, "2026.11.05".to_owned()).unwrap();

        assert_eq!(record.result, UNFINISHED_RESULT);
        assert_eq!(record.reason, None);

        let state = stopped_game(&game, 15_000);
        let resumed = ActiveGame::resume(state, 7, 900_000).unwrap();
        let view = resumed.view(900_000);
        let clock = view.clock.unwrap();

        assert_eq!(view.moves.len(), 2);
        assert_eq!(view.move_marks, vec![mark(1)]);
        assert_eq!(view.opponent_name, "Fake");
        assert_eq!(clock.white_ms, MINUTE_MS - 10_000);
        assert_eq!(clock.running, Some(Color::White));
        assert_eq!(resumed.archive_id(), Some(7));
        assert!(resumed.is_saved());
    }

    #[test]
    fn a_resumed_game_can_still_take_a_move_back_with_its_clock() {
        let mut game = as_white(timed());

        game.human_move("e2e4", 0).unwrap();
        game.engine_move("e7e5", None, 2_000).unwrap();
        game.human_move("g1f3", 12_000).unwrap();
        game.engine_move("b8c6", None, 15_000).unwrap();

        let state = stopped_game(&game, 20_000);
        let mut resumed = ActiveGame::resume(state, 7, 0).unwrap();

        resumed.take_back(1_000).unwrap();

        let view = resumed.view(1_000);

        assert_eq!(view.moves.len(), 2);
        assert_eq!(view.clock.unwrap().white_ms, MINUTE_MS);
    }

    #[test]
    fn a_game_waiting_for_the_engine_asks_it_again_after_resuming() {
        let mut game = as_white(None);

        game.human_move("e2e4", 0).unwrap();

        let resumed = ActiveGame::resume(stopped_game(&game, 0), 7, 0).unwrap();

        assert!(resumed.is_engine_turn());
        assert!(!resumed.is_thinking());
    }

    #[test]
    fn a_resumed_game_keeps_the_strength_and_the_help() {
        let mut game = ActiveGame::new(
            &NewGameOptions {
                elo: Some(1900),
                training: true,
                ..options(None)
            },
            Color::White,
            "Fake".to_owned(),
        )
        .unwrap();

        game.human_move("e2e4", 0).unwrap();

        let state = stopped_game(&game, 0);

        assert_eq!(state.opponent_elo(), Some(1900));
        assert!(state.is_training());
    }
}
