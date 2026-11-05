//! One game: position, history and the automatic end conditions.

use gb_model::{
    CapturedPieces, ClockView, Color, GameEndReason, GameOutcome, GameResult, GameView, LastMove,
    MoveOption, MoveRecord, MoveSound, PieceKind, ReplayFrame,
};
use shakmaty::fen::Fen;
use shakmaty::san::{San, SanPlus, Suffix};
use shakmaty::uci::UciMove;
use shakmaty::zobrist::Zobrist64;
use shakmaty::{CastlingMode, Chess, EnPassantMode, Move, Position};

use crate::convert::{chess_color, move_option, move_squares, uci_text, wire_color};
use crate::error::ChessError;
use crate::premove::premove_options;
use crate::tracker::{Capture, PieceTracker};

const FIFTY_MOVE_HALFMOVES: u32 = 100;
const THREEFOLD_COUNT: usize = 3;

#[derive(Debug, Clone)]
struct PlyEntry {
    record: MoveRecord,
    capture: Option<Capture>,
    hash_after: Zobrist64,
    /// The state before this move, to take it back.
    position_before: Chess,
    tracker_before: PieceTracker,
}

/// What the session knows and the rules do not.
#[derive(Debug, Clone)]
pub struct GameExtras {
    pub human_color: Color,
    pub opponent_name: String,
    pub premove: Option<MoveOption>,
    pub clock: Option<ClockView>,
    pub engine_thinking: bool,
    pub can_takeback: bool,
}

#[derive(Debug, Clone)]
pub struct Game {
    position: Chess,
    tracker: PieceTracker,
    start_hash: Zobrist64,
    custom_start_fen: Option<String>,
    plies: Vec<PlyEntry>,
    outcome: Option<GameOutcome>,
}

fn hash_of(position: &Chess) -> Zobrist64 {
    position.zobrist_hash::<Zobrist64>(EnPassantMode::Legal)
}

fn parse_fen(fen: &str) -> Result<Chess, ChessError> {
    let parsed: Fen = fen
        .parse()
        .map_err(|error: shakmaty::fen::ParseFenError| ChessError::InvalidFen(error.to_string()))?;

    parsed
        .into_position(CastlingMode::Standard)
        .map_err(|error| ChessError::InvalidFen(error.to_string()))
}

fn win_for(color: Color) -> GameResult {
    match color {
        Color::White => GameResult::WhiteWins,
        Color::Black => GameResult::BlackWins,
    }
}

const fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Queen => 9,
        PieceKind::Rook => 5,
        PieceKind::Bishop | PieceKind::Knight => 3,
        PieceKind::Pawn => 1,
        PieceKind::King => 0,
    }
}

fn sound_of(chess_move: Move, san: &SanPlus) -> MoveSound {
    if matches!(san.suffix, Some(Suffix::Check | Suffix::Checkmate)) {
        return MoveSound::Check;
    }

    if chess_move.is_promotion() {
        return MoveSound::Promote;
    }

    if matches!(san.san, San::Castle(_)) {
        return MoveSound::Castle;
    }

    if chess_move.is_capture() {
        return MoveSound::Capture;
    }

    MoveSound::Move
}

impl Game {
    pub fn new(start_fen: Option<&str>) -> Result<Self, ChessError> {
        let position = match start_fen {
            Some(fen) => parse_fen(fen)?,
            None => Chess::default(),
        };
        let mut game = Self {
            tracker: PieceTracker::new(position.board()),
            start_hash: hash_of(&position),
            custom_start_fen: start_fen.map(str::to_owned),
            position,
            plies: Vec::new(),
            outcome: None,
        };

        game.outcome = game.automatic_outcome();

        Ok(game)
    }

    pub fn turn(&self) -> Color {
        wire_color(self.position.turn())
    }

    pub fn outcome(&self) -> Option<GameOutcome> {
        self.outcome
    }

    pub fn ply_count(&self) -> usize {
        self.plies.len()
    }

    pub fn moves(&self) -> Vec<MoveRecord> {
        self.plies.iter().map(|ply| ply.record.clone()).collect()
    }

    pub fn fen(&self) -> String {
        Fen::from_position(&self.position, EnPassantMode::Legal).to_string()
    }

    pub fn custom_start_fen(&self) -> Option<&str> {
        self.custom_start_fen.as_deref()
    }

    fn is_threefold(&self, hash: Zobrist64) -> bool {
        let in_history = self
            .plies
            .iter()
            .filter(|ply| ply.hash_after == hash)
            .count();
        let at_start = usize::from(self.start_hash == hash);

        in_history + at_start >= THREEFOLD_COUNT
    }

    fn automatic_outcome(&self) -> Option<GameOutcome> {
        let decided = |result, reason| Some(GameOutcome { result, reason });

        if self.position.is_checkmate() {
            return decided(win_for(self.turn().opposite()), GameEndReason::Checkmate);
        }

        if self.position.is_stalemate() {
            return decided(GameResult::Draw, GameEndReason::Stalemate);
        }

        if self.position.is_insufficient_material() {
            return decided(GameResult::Draw, GameEndReason::InsufficientMaterial);
        }

        if self.is_threefold(hash_of(&self.position)) {
            return decided(GameResult::Draw, GameEndReason::ThreefoldRepetition);
        }

        if self.position.halfmoves() >= FIFTY_MOVE_HALFMOVES {
            return decided(GameResult::Draw, GameEndReason::FiftyMoveRule);
        }

        None
    }

    /// Plays a legal move and records it. Does not look at the outcome.
    fn advance(&mut self, chess_move: Move) -> MoveRecord {
        let position_before = self.position.clone();
        let tracker_before = self.tracker.clone();
        let mover = self.position.turn();
        let uci = uci_text(chess_move);
        let (from, to) = move_squares(chess_move);
        let capture = self.tracker.apply(chess_move, mover);
        let san = SanPlus::from_move_and_play_unchecked(&mut self.position, chess_move);
        let record = MoveRecord {
            ply: u32::try_from(self.plies.len() + 1).unwrap_or(u32::MAX),
            san: san.to_string(),
            uci,
            from,
            to,
            color: wire_color(mover),
            sound: sound_of(chess_move, &san),
        };

        self.plies.push(PlyEntry {
            record: record.clone(),
            capture,
            hash_after: hash_of(&self.position),
            position_before,
            tracker_before,
        });

        record
    }

    fn find_move(&self, uci: &str) -> Result<Move, ChessError> {
        let illegal = || ChessError::IllegalMove(uci.to_owned());
        let parsed: UciMove = uci.parse().map_err(|_| illegal())?;

        parsed.to_move(&self.position).map_err(|_| illegal())
    }

    /// Plays a UCI move and ends the game if the move ended it.
    pub fn play_uci(&mut self, uci: &str) -> Result<MoveRecord, ChessError> {
        if self.outcome.is_some() {
            return Err(ChessError::GameOver);
        }

        let chess_move = self.find_move(uci)?;
        let record = self.advance(chess_move);

        self.outcome = self.automatic_outcome();

        Ok(record)
    }

    pub fn is_legal_uci(&self, uci: &str) -> bool {
        self.find_move(uci).is_ok()
    }

    pub(crate) fn play_san(&mut self, san: &San) -> Result<MoveRecord, ChessError> {
        let chess_move = san
            .to_move(&self.position)
            .map_err(|error| ChessError::IllegalMove(format!("{san}: {error}")))?;

        Ok(self.advance(chess_move))
    }

    /// Undoes the last move and reopens the game.
    pub fn undo(&mut self) -> Result<(), ChessError> {
        let ply = self.plies.pop().ok_or(ChessError::NothingToUndo)?;

        self.position = ply.position_before;
        self.tracker = ply.tracker_before;
        self.outcome = None;

        Ok(())
    }

    pub fn resign(&mut self, loser: Color) {
        self.finish(win_for(loser.opposite()), GameEndReason::Resignation);
    }

    pub fn agree_draw(&mut self) {
        self.finish(GameResult::Draw, GameEndReason::DrawAgreement);
    }

    /// FIDE 6.9: a flag loses, unless the opponent has no way to mate.
    pub fn flag_fall(&mut self, loser: Color) {
        let winner = chess_color(loser.opposite());

        if self.position.has_insufficient_material(winner) {
            self.finish(GameResult::Draw, GameEndReason::TimeoutInsufficientMaterial);
        } else {
            self.finish(win_for(loser.opposite()), GameEndReason::Timeout);
        }
    }

    fn finish(&mut self, result: GameResult, reason: GameEndReason) {
        if self.outcome.is_none() {
            self.outcome = Some(GameOutcome { result, reason });
        }
    }

    fn captured(&self) -> CapturedPieces {
        let mut captured = CapturedPieces::default();

        for capture in self.plies.iter().filter_map(|ply| ply.capture) {
            match capture.color {
                Color::White => captured.black.push(capture.kind),
                Color::Black => captured.white.push(capture.kind),
            }
        }

        captured
    }

    fn material_diff(&self) -> i32 {
        self.tracker
            .pieces()
            .iter()
            .map(|piece| match piece.color {
                Color::White => piece_value(piece.kind),
                Color::Black => -piece_value(piece.kind),
            })
            .sum()
    }

    fn check_square(&self) -> Option<String> {
        if !self.position.is_check() {
            return None;
        }

        self.position
            .board()
            .king_of(self.position.turn())
            .map(|square| square.to_string())
    }

    fn last_move(&self) -> Option<LastMove> {
        self.plies.last().map(|ply| LastMove {
            from: ply.record.from.clone(),
            to: ply.record.to.clone(),
        })
    }

    pub fn view(&self, extras: GameExtras) -> GameView {
        let is_over = self.outcome.is_some();
        let human_to_move = self.turn() == extras.human_color;
        let legal_moves = if is_over || !human_to_move {
            Vec::new()
        } else {
            self.position
                .legal_moves()
                .into_iter()
                .map(move_option)
                .collect()
        };
        let premove_list = if is_over || human_to_move {
            Vec::new()
        } else {
            premove_options(&self.position, chess_color(extras.human_color))
        };

        GameView {
            pieces: self.tracker.pieces(),
            fen: self.fen(),
            turn: self.turn(),
            human_color: extras.human_color,
            opponent_name: extras.opponent_name,
            outcome: self.outcome,
            legal_moves,
            premove_options: premove_list,
            premove: extras.premove,
            last_move: self.last_move(),
            check_square: self.check_square(),
            moves: self.moves(),
            captured: self.captured(),
            material_diff: self.material_diff(),
            clock: extras.clock,
            engine_thinking: extras.engine_thinking,
            can_takeback: extras.can_takeback,
        }
    }

    pub(crate) fn frame(&self) -> ReplayFrame {
        ReplayFrame {
            pieces: self.tracker.pieces(),
            fen: self.fen(),
            last_move: self.last_move(),
            check_square: self.check_square(),
            sound: self.plies.last().map(|ply| ply.record.sound),
            captured: self.captured(),
            material_diff: self.material_diff(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play_all(game: &mut Game, moves: &[&str]) {
        for uci in moves {
            game.play_uci(uci).unwrap();
        }
    }

    fn piece_id(game: &Game, square: &str) -> u32 {
        game.tracker
            .pieces()
            .iter()
            .find(|piece| piece.square == square)
            .unwrap()
            .id
    }

    fn extras() -> GameExtras {
        GameExtras {
            human_color: Color::White,
            opponent_name: "Engine".to_owned(),
            premove: None,
            clock: None,
            engine_thinking: false,
            can_takeback: false,
        }
    }

    #[test]
    fn fools_mate_ends_by_checkmate() {
        let mut game = Game::new(None).unwrap();

        play_all(&mut game, &["f2f3", "e7e5", "g2g4", "d8h4"]);

        assert_eq!(
            game.outcome(),
            Some(GameOutcome {
                result: GameResult::BlackWins,
                reason: GameEndReason::Checkmate,
            })
        );
        assert_eq!(game.moves()[3].san, "Qh4#");
        assert_eq!(game.moves()[3].sound, MoveSound::Check);
        assert_eq!(game.play_uci("a2a3"), Err(ChessError::GameOver));
    }

    #[test]
    fn an_illegal_move_changes_nothing() {
        let mut game = Game::new(None).unwrap();

        assert!(matches!(
            game.play_uci("e2e5"),
            Err(ChessError::IllegalMove(_))
        ));
        assert_eq!(game.ply_count(), 0);
    }

    #[test]
    fn stalemate_is_a_draw() {
        let mut game = Game::new(Some("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1")).unwrap();

        assert_eq!(
            game.outcome().map(|outcome| outcome.reason),
            Some(GameEndReason::Stalemate)
        );
        assert_eq!(game.undo(), Err(ChessError::NothingToUndo));
    }

    #[test]
    fn threefold_repetition_is_automatic() {
        let mut game = Game::new(None).unwrap();

        play_all(
            &mut game,
            &[
                "g1f3", "g8f6", "f3g1", "f6g8", "g1f3", "g8f6", "f3g1", "f6g8",
            ],
        );

        assert_eq!(
            game.outcome().map(|outcome| outcome.reason),
            Some(GameEndReason::ThreefoldRepetition)
        );
    }

    #[test]
    fn the_fifty_move_rule_is_automatic() {
        let mut game = Game::new(Some("8/8/8/3k4/8/8/4R3/4K3 w - - 99 80")).unwrap();

        game.play_uci("e2a2").unwrap();

        assert_eq!(
            game.outcome().map(|outcome| outcome.reason),
            Some(GameEndReason::FiftyMoveRule)
        );
    }

    #[test]
    fn bare_kings_are_insufficient_material() {
        let mut game = Game::new(Some("8/8/8/8/k3r3/4K3/8/8 w - - 0 1")).unwrap();

        game.play_uci("e3e4").unwrap();

        assert_eq!(
            game.outcome().map(|outcome| outcome.reason),
            Some(GameEndReason::InsufficientMaterial)
        );
    }

    #[test]
    fn a_flag_against_a_bare_king_is_a_draw() {
        let mut game = Game::new(Some("8/8/8/4k3/8/8/4P3/4K3 b - - 0 1")).unwrap();

        game.flag_fall(Color::White);

        assert_eq!(
            game.outcome(),
            Some(GameOutcome {
                result: GameResult::Draw,
                reason: GameEndReason::TimeoutInsufficientMaterial,
            })
        );
    }

    #[test]
    fn a_flag_loses_when_the_opponent_can_mate() {
        let mut game = Game::new(Some("8/8/8/4k3/8/4r3/8/4K3 w - - 0 1")).unwrap();

        game.flag_fall(Color::White);

        assert_eq!(
            game.outcome(),
            Some(GameOutcome {
                result: GameResult::BlackWins,
                reason: GameEndReason::Timeout,
            })
        );
    }

    #[test]
    fn ids_survive_capture_and_a_moved_piece_keeps_its_id() {
        let mut game = Game::new(None).unwrap();
        let knight = piece_id(&game, "g1");

        play_all(&mut game, &["g1f3", "e7e5", "f3e5"]);

        assert_eq!(piece_id(&game, "e5"), knight);
        assert_eq!(game.tracker.pieces().len(), 31);
        assert_eq!(game.captured().white, vec![PieceKind::Pawn]);
        assert_eq!(game.material_diff(), 1);
        assert_eq!(game.moves()[2].sound, MoveSound::Capture);
    }

    #[test]
    fn castling_moves_both_pieces_and_keeps_their_ids() {
        let mut game = Game::new(Some("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")).unwrap();
        let king = piece_id(&game, "e1");
        let rook = piece_id(&game, "h1");

        let record = game.play_uci("e1g1").unwrap();

        assert_eq!(record.san, "O-O");
        assert_eq!(record.to, "g1", "the king's square, not the rook's");
        assert_eq!(record.sound, MoveSound::Castle);
        assert_eq!(piece_id(&game, "g1"), king);
        assert_eq!(piece_id(&game, "f1"), rook);
    }

    #[test]
    fn en_passant_removes_the_passed_pawn() {
        let mut game = Game::new(None).unwrap();
        let pawn = piece_id(&game, "e2");

        play_all(&mut game, &["e2e4", "a7a6", "e4e5", "d7d5", "e5d6"]);

        assert_eq!(piece_id(&game, "d6"), pawn);
        assert!(game
            .tracker
            .pieces()
            .iter()
            .all(|piece| piece.square != "d5"));
        assert_eq!(game.moves()[4].sound, MoveSound::Capture);
    }

    #[test]
    fn promotion_keeps_the_id_and_changes_the_kind() {
        let mut game = Game::new(Some("8/4P3/8/8/8/8/8/k1K5 w - - 0 1")).unwrap();
        let pawn = piece_id(&game, "e7");

        let record = game.play_uci("e7e8q").unwrap();

        assert_eq!(record.sound, MoveSound::Promote);
        assert_eq!(piece_id(&game, "e8"), pawn);
        assert_eq!(
            game.tracker
                .pieces()
                .iter()
                .find(|piece| piece.id == pawn)
                .unwrap()
                .kind,
            PieceKind::Queen
        );
    }

    #[test]
    fn undo_restores_the_position_and_reopens_the_game() {
        let mut game = Game::new(None).unwrap();
        let start = game.fen();

        play_all(&mut game, &["f2f3", "e7e5", "g2g4", "d8h4"]);
        game.undo().unwrap();

        assert_eq!(game.outcome(), None);
        assert_eq!(game.ply_count(), 3);

        game.undo().unwrap();
        game.undo().unwrap();
        game.undo().unwrap();

        assert_eq!(game.fen(), start);
    }

    #[test]
    fn the_view_offers_moves_only_to_the_side_to_move() {
        let mut game = Game::new(None).unwrap();

        let human_turn = game.view(extras());

        assert_eq!(human_turn.legal_moves.len(), 20);
        assert!(human_turn.premove_options.is_empty());

        game.play_uci("e2e4").unwrap();

        let engine_turn = game.view(extras());

        assert!(engine_turn.legal_moves.is_empty());
        assert!(!engine_turn.premove_options.is_empty());
    }

    #[test]
    fn promotion_is_one_option_per_piece() {
        let game = Game::new(Some("8/4P3/8/8/8/8/8/k1K5 w - - 0 1")).unwrap();
        let view = game.view(extras());
        let promotions = view
            .legal_moves
            .iter()
            .filter(|option| option.from == "e7" && option.to == "e8")
            .count();

        assert_eq!(promotions, 4);
    }

    #[test]
    fn the_check_square_is_the_king_in_check() {
        let mut game = Game::new(None).unwrap();

        play_all(&mut game, &["e2e4", "f7f6", "d2d4", "g7g5", "a2a3"]);
        game.play_uci("h7h6").unwrap();
        game.play_uci("d1h5").unwrap();

        assert_eq!(game.check_square().as_deref(), Some("e8"));
    }
}
