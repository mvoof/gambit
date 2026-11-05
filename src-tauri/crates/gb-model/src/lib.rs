//! The wire contract: every type and event name that crosses to the frontend.
//!
//! Numbers stay within `u32`/`i32`/`f64`: a `u64` is not a safe JS number.
//! Payloads are finite, so a missing value is an `Option`, never NaN.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const EVENT_GAME_UPDATED: &str = "game-updated";
pub const EVENT_EVALUATION: &str = "evaluation";
pub const EVENT_ENGINE_INSTALL: &str = "engine-install-progress";
pub const EVENT_GAME_REVIEW: &str = "game-review";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

/// A piece on the board. `id` survives moves, so the frontend animates by key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct BoardPiece {
    pub id: u32,
    pub color: Color,
    pub kind: PieceKind,
    pub square: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum MoveSound {
    Move,
    Capture,
    Castle,
    Check,
    Promote,
}

/// A move the player may choose. A promotion is one option per piece.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct MoveOption {
    pub from: String,
    pub to: String,
    pub uci: String,
    pub promotion: Option<PieceKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct MoveRecord {
    pub ply: u32,
    pub san: String,
    pub uci: String,
    pub from: String,
    pub to: String,
    pub color: Color,
    pub sound: MoveSound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct LastMove {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum GameEndReason {
    Checkmate,
    Stalemate,
    ThreefoldRepetition,
    FiftyMoveRule,
    InsufficientMaterial,
    Resignation,
    DrawAgreement,
    Timeout,
    /// The flag fell, but the opponent had no mating material (FIDE 6.9).
    TimeoutInsufficientMaterial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GameOutcome {
    pub result: GameResult,
    pub reason: GameEndReason,
}

/// Pieces each side has captured.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct CapturedPieces {
    pub white: Vec<PieceKind>,
    pub black: Vec<PieceKind>,
}

/// Rust owns the clock. The frontend counts `running` down from the moment
/// `sync_ms` changes, for display only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ClockView {
    pub white_ms: u32,
    pub black_ms: u32,
    pub running: Option<Color>,
    /// When this snapshot was taken, in milliseconds on the backend's own clock.
    pub sync_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum MoveQuality {
    Inaccuracy,
    Mistake,
    Blunder,
}

/// What the engine measured about one of the player's moves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct MoveMark {
    pub ply: u32,
    pub quality: MoveQuality,
    pub best_san: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum GameReviewStatus {
    Measuring,
    Done,
    Failed,
}

/// How far the measuring of a saved game has come. `marks` holds all found so far.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GameReview {
    pub game_id: u32,
    pub status: GameReviewStatus,
    /// Positions measured, out of `total`.
    pub measured: u32,
    pub total: u32,
    pub marks: Vec<MoveMark>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub pieces: Vec<BoardPiece>,
    pub fen: String,
    pub turn: Color,
    pub human_color: Color,
    pub opponent_name: String,
    pub outcome: Option<GameOutcome>,
    pub legal_moves: Vec<MoveOption>,
    /// Geometrically possible moves of the human while the engine is to play.
    pub premove_options: Vec<MoveOption>,
    pub premove: Option<MoveOption>,
    pub last_move: Option<LastMove>,
    pub check_square: Option<String>,
    pub moves: Vec<MoveRecord>,
    pub move_marks: Vec<MoveMark>,
    pub captured: CapturedPieces,
    /// Positive when White is ahead.
    pub material_diff: i32,
    pub clock: Option<ClockView>,
    pub engine_thinking: bool,
    /// The opponent process died or gave no move: the game cannot go on.
    pub engine_failed: bool,
    pub can_takeback: bool,
    pub is_training: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TimeControl {
    pub base_ms: u32,
    pub increment_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum ColorChoice {
    White,
    Black,
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct NewGameOptions {
    pub color: ColorChoice,
    /// `None` is unlimited time.
    pub time_control: Option<TimeControl>,
    /// `None` is full strength.
    pub elo: Option<u32>,
    pub start_fen: Option<String>,
    /// The engine helps during the game. An ordinary game shows nothing.
    #[serde(default)]
    pub training: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EloRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub name: String,
    pub path: String,
    /// `None` when the engine has no `UCI_LimitStrength`.
    pub elo_range: Option<EloRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum EngineInstallStage {
    Downloading,
    Verifying,
    Extracting,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EngineInstallProgress {
    pub stage: EngineInstallStage,
    pub downloaded_bytes: u32,
    pub total_bytes: u32,
}

/// One of the engine's best lines, White's point of view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EvaluationLine {
    /// SAN with move numbers, first move first.
    pub moves: Vec<String>,
    pub centipawns: Option<i32>,
    pub mate_in: Option<i32>,
}

/// A measurement from the full-strength analysis engine, White's point of view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Evaluation {
    /// The position measured: a reply for another position is dropped.
    pub fen: String,
    pub turn: Color,
    pub centipawns: Option<i32>,
    pub mate_in: Option<i32>,
    /// White's chance from 0 to 100, so the bar only sets a height.
    pub white_win_percent: f64,
    pub depth: u32,
    /// `None` until the search is deep enough for its first move to settle.
    pub best_move: Option<LastMove>,
    /// Best first.
    pub lines: Vec<EvaluationLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ReplayFrame {
    pub pieces: Vec<BoardPiece>,
    pub fen: String,
    pub last_move: Option<LastMove>,
    pub check_square: Option<String>,
    pub sound: Option<MoveSound>,
    pub captured: CapturedPieces,
    pub material_diff: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ReplayGame {
    pub white: String,
    pub black: String,
    pub result: String,
    /// The PGN `TimeControl` value, `-` when there is none.
    pub time_control: String,
    pub moves: Vec<MoveRecord>,
    /// One more than `moves`: frame 0 is the starting position.
    pub frames: Vec<ReplayFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    pub id: u32,
    pub white: String,
    pub black: String,
    pub result: String,
    pub reason: Option<GameEndReason>,
    /// Unix seconds.
    pub played_at: u32,
    pub ply_count: u32,
    pub time_control: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum SheetColumnOrder {
    Kqrbnp,
    Kqbnrp,
}

/// `wK…bP` to an image URL. Bundled and custom sets look the same to the UI.
pub type PieceImages = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct PieceSetView {
    pub id: String,
    pub name: String,
    pub images: PieceImages,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct BoardSettings {
    pub light_square: String,
    pub dark_square: String,
    pub show_coordinates: bool,
    pub show_legal_moves: bool,
    pub show_last_move: bool,
    pub show_check: bool,
}

impl Default for BoardSettings {
    fn default() -> Self {
        Self {
            light_square: "#e3d3b5".to_owned(),
            dark_square: "#8a6a4a".to_owned(),
            show_coordinates: true,
            show_legal_moves: true,
            show_last_move: true,
            show_check: true,
        }
    }
}

const DEFAULT_SOUND_VOLUME: u32 = 70;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub piece_set: String,
    pub board: BoardSettings,
    /// From 0 to 100.
    pub sound_volume: u32,
    pub eval_bar_enabled: bool,
    pub engine_path: Option<String>,
    pub last_game: Option<NewGameOptions>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_owned(),
            piece_set: "cburnett".to_owned(),
            board: BoardSettings::default(),
            sound_volume: DEFAULT_SOUND_VOLUME,
            eval_bar_enabled: true,
            engine_path: None,
            last_game: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names_are_camel_case() {
        let clock = ClockView {
            white_ms: 1,
            black_ms: 2,
            running: Some(Color::White),
            sync_ms: 3,
        };
        let json = serde_json::to_string(&clock).unwrap();

        assert_eq!(
            json,
            r#"{"whiteMs":1,"blackMs":2,"running":"white","syncMs":3}"#
        );
    }
}
