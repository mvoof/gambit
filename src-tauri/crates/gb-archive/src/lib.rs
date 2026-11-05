//! The saved games: SQLite. A game is its PGN plus the columns a list needs.

use std::fmt;
use std::path::Path;

use gb_model::{
    ArchiveStats, Color, EvalPoint, GameEndReason, GameSummary, MoveMark, PlayerResult,
};
use rusqlite::{params, Connection, OptionalExtension, Row};

/// Bump and add a step to `migrate` when the schema changes.
const SCHEMA_VERSION: i32 = 3;

const CREATE_GAMES: &str = "
    CREATE TABLE games (
        id INTEGER PRIMARY KEY,
        white TEXT NOT NULL,
        black TEXT NOT NULL,
        result TEXT NOT NULL,
        reason TEXT,
        played_at INTEGER NOT NULL,
        ply_count INTEGER NOT NULL,
        time_control TEXT NOT NULL,
        pgn TEXT NOT NULL
    );
    CREATE INDEX games_played_at ON games (played_at DESC);
";

/// JSON, empty until the engine has measured the game.
const ADD_MARKS: &str = "ALTER TABLE games ADD COLUMN marks TEXT;";

/// Marks of the older format name no player and have no graph: they go, and the
/// game is measured again when its replay is opened.
const ADD_PLAYER_AND_RESUME: &str = "
    ALTER TABLE games ADD COLUMN human_color TEXT;
    ALTER TABLE games ADD COLUMN opponent_elo INTEGER;
    ALTER TABLE games ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE games ADD COLUMN final_fen TEXT;
    ALTER TABLE games ADD COLUMN resume TEXT;
    ALTER TABLE games ADD COLUMN points TEXT;
    ALTER TABLE games ADD COLUMN mistake_count INTEGER NOT NULL DEFAULT 0;
    UPDATE games SET human_color = 'white' WHERE white = 'Player';
    UPDATE games SET human_color = 'black' WHERE black = 'Player' AND human_color IS NULL;
    UPDATE games SET marks = NULL;
";

const SUMMARY_COLUMNS: &str = "id, white, black, result, reason, played_at, ply_count,
    time_control, human_color, opponent_elo, is_favorite, final_fen, resume IS NOT NULL,
    marks IS NOT NULL AND points IS NOT NULL, mistake_count";

const WHITE_WINS: &str = "1-0";
const BLACK_WINS: &str = "0-1";
const DRAW: &str = "1/2-1/2";
const PERCENT: u32 = 100;

#[derive(Debug)]
pub enum ArchiveError {
    Database(rusqlite::Error),
    Marks(serde_json::Error),
    NotFound(u32),
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "archive database: {error}"),
            Self::Marks(error) => write!(formatter, "cannot store the marks: {error}"),
            Self::NotFound(id) => write!(formatter, "no saved game {id}"),
        }
    }
}

impl std::error::Error for ArchiveError {}

impl From<rusqlite::Error> for ArchiveError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewGame {
    pub white: String,
    pub black: String,
    pub result: String,
    pub reason: Option<GameEndReason>,
    pub played_at: u32,
    pub ply_count: u32,
    pub time_control: String,
    pub pgn: String,
    pub human_color: Option<Color>,
    pub opponent_elo: Option<u32>,
    pub final_fen: Option<String>,
    /// What it takes to play on; `None` for a finished game.
    pub resume: Option<String>,
}

/// What the engine measured about a game.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredReview {
    pub marks: Vec<MoveMark>,
    pub points: Vec<EvalPoint>,
}

pub struct Archive {
    connection: Connection,
}

fn reason_text(reason: Option<GameEndReason>) -> Option<String> {
    reason.and_then(|value| {
        serde_json::to_value(value)
            .ok()?
            .as_str()
            .map(str::to_owned)
    })
}

fn reason_from(text: Option<String>) -> Option<GameEndReason> {
    serde_json::from_value(serde_json::Value::String(text?)).ok()
}

fn color_text(color: Option<Color>) -> Option<&'static str> {
    color.map(|value| match value {
        Color::White => "white",
        Color::Black => "black",
    })
}

fn color_from(text: Option<String>) -> Option<Color> {
    match text?.as_str() {
        "white" => Some(Color::White),
        "black" => Some(Color::Black),
        _ => None,
    }
}

fn player_result(result: &str, human_color: Option<Color>, is_unfinished: bool) -> PlayerResult {
    if is_unfinished {
        return PlayerResult::Unfinished;
    }

    match (result, human_color) {
        (DRAW, Some(_)) => PlayerResult::Draw,
        (WHITE_WINS, Some(Color::White)) | (BLACK_WINS, Some(Color::Black)) => PlayerResult::Win,
        (WHITE_WINS, Some(Color::Black)) | (BLACK_WINS, Some(Color::White)) => PlayerResult::Loss,
        _ => PlayerResult::Unknown,
    }
}

fn opponent_of(white: &str, black: &str, human_color: Option<Color>) -> String {
    match human_color {
        Some(Color::White) => black.to_owned(),
        Some(Color::Black) => white.to_owned(),
        None => format!("{white} — {black}"),
    }
}

fn summary_from(row: &Row<'_>) -> rusqlite::Result<GameSummary> {
    let white: String = row.get(1)?;
    let black: String = row.get(2)?;
    let result: String = row.get(3)?;
    let human_color = color_from(row.get(8)?);
    let is_unfinished: bool = row.get(12)?;

    Ok(GameSummary {
        id: row.get(0)?,
        opponent: opponent_of(&white, &black, human_color),
        opponent_elo: row.get(9)?,
        human_color,
        player_result: player_result(&result, human_color, is_unfinished),
        white,
        black,
        result,
        reason: reason_from(row.get(4)?),
        played_at: row.get(5)?,
        ply_count: row.get(6)?,
        time_control: row.get(7)?,
        is_favorite: row.get(10)?,
        final_fen: row.get(11)?,
        is_reviewed: row.get(13)?,
        mistake_count: row.get(14)?,
    })
}

fn count_of(games: &[GameSummary], matches: impl Fn(&GameSummary) -> bool) -> u32 {
    u32::try_from(games.iter().filter(|game| matches(game)).count()).unwrap_or(u32::MAX)
}

fn stats_of(games: &[GameSummary], size_bytes: u32) -> ArchiveStats {
    let result_count = |wanted: PlayerResult| count_of(games, |game| game.player_result == wanted);
    let wins = result_count(PlayerResult::Win);
    let losses = result_count(PlayerResult::Loss);
    let draws = result_count(PlayerResult::Draw);

    ArchiveStats {
        total: u32::try_from(games.len()).unwrap_or(u32::MAX),
        wins,
        losses,
        draws,
        unfinished: result_count(PlayerResult::Unfinished),
        reviewed: count_of(games, |game| game.is_reviewed),
        with_mistakes: count_of(games, |game| game.mistake_count > 0),
        favorites: count_of(games, |game| game.is_favorite),
        win_percent: (wins * PERCENT)
            .checked_div(wins + losses + draws)
            .unwrap_or(0),
        size_bytes,
    }
}

impl Archive {
    pub fn open(path: &Path) -> Result<Self, ArchiveError> {
        Self::with_connection(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, ArchiveError> {
        Self::with_connection(Connection::open_in_memory()?)
    }

    fn with_connection(connection: Connection) -> Result<Self, ArchiveError> {
        let archive = Self { connection };

        archive.migrate()?;

        Ok(archive)
    }

    fn migrate(&self) -> Result<(), ArchiveError> {
        let version: i32 = self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;

        if version < 1 {
            self.connection.execute_batch(CREATE_GAMES)?;
        }

        if version < 2 {
            self.connection.execute_batch(ADD_MARKS)?;
        }

        if version < 3 {
            self.connection.execute_batch(ADD_PLAYER_AND_RESUME)?;
        }

        if version < SCHEMA_VERSION {
            self.connection
                .pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }

        Ok(())
    }

    pub fn save(&self, game: &NewGame) -> Result<u32, ArchiveError> {
        self.connection.execute(
            "INSERT INTO games
                (white, black, result, reason, played_at, ply_count, time_control, pgn,
                 human_color, opponent_elo, final_fen, resume)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                game.white,
                game.black,
                game.result,
                reason_text(game.reason),
                game.played_at,
                game.ply_count,
                game.time_control,
                game.pgn,
                color_text(game.human_color),
                game.opponent_elo,
                game.final_fen,
                game.resume,
            ],
        )?;

        Ok(u32::try_from(self.connection.last_insert_rowid()).unwrap_or(u32::MAX))
    }

    /// The game went on: what the engine measured no longer describes it.
    pub fn update(&self, id: u32, game: &NewGame) -> Result<(), ArchiveError> {
        let updated = self.connection.execute(
            "UPDATE games SET
                 result = ?1, reason = ?2, played_at = ?3, ply_count = ?4, pgn = ?5,
                 final_fen = ?6, resume = ?7, marks = NULL, points = NULL, mistake_count = 0
             WHERE id = ?8",
            params![
                game.result,
                reason_text(game.reason),
                game.played_at,
                game.ply_count,
                game.pgn,
                game.final_fen,
                game.resume,
                id,
            ],
        )?;

        if updated == 0 {
            return Err(ArchiveError::NotFound(id));
        }

        Ok(())
    }

    /// Newest first.
    pub fn list(&self) -> Result<Vec<GameSummary>, ArchiveError> {
        let mut statement = self.connection.prepare(&format!(
            "SELECT {SUMMARY_COLUMNS} FROM games ORDER BY played_at DESC, id DESC"
        ))?;
        let rows = statement.query_map([], summary_from)?;

        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn stats(&self) -> Result<ArchiveStats, ArchiveError> {
        let size: i64 = self.connection.query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get(0),
        )?;

        Ok(stats_of(
            &self.list()?,
            u32::try_from(size).unwrap_or(u32::MAX),
        ))
    }

    pub fn set_favorite(&self, id: u32, is_favorite: bool) -> Result<(), ArchiveError> {
        let updated = self.connection.execute(
            "UPDATE games SET is_favorite = ?1 WHERE id = ?2",
            params![is_favorite, id],
        )?;

        if updated == 0 {
            return Err(ArchiveError::NotFound(id));
        }

        Ok(())
    }

    /// `None` when the game is finished.
    pub fn resume_state(&self, id: u32) -> Result<Option<String>, ArchiveError> {
        self.connection
            .query_row("SELECT resume FROM games WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?
            .ok_or(ArchiveError::NotFound(id))
    }

    pub fn human_color(&self, id: u32) -> Result<Option<Color>, ArchiveError> {
        let text: Option<String> = self
            .connection
            .query_row("SELECT human_color FROM games WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?
            .ok_or(ArchiveError::NotFound(id))?;

        Ok(color_from(text))
    }

    /// Games saved before the thumbnails: their id and PGN.
    pub fn without_final_fen(&self) -> Result<Vec<(u32, String)>, ArchiveError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, pgn FROM games WHERE final_fen IS NULL")?;
        let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn set_final_fen(&self, id: u32, fen: &str) -> Result<(), ArchiveError> {
        self.connection.execute(
            "UPDATE games SET final_fen = ?1 WHERE id = ?2",
            params![fen, id],
        )?;

        Ok(())
    }

    pub fn pgn(&self, id: u32) -> Result<String, ArchiveError> {
        self.connection
            .query_row("SELECT pgn FROM games WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?
            .ok_or(ArchiveError::NotFound(id))
    }

    /// `None` until the game has been measured.
    pub fn review(&self, id: u32) -> Result<Option<StoredReview>, ArchiveError> {
        let texts: (Option<String>, Option<String>) = self
            .connection
            .query_row(
                "SELECT marks, points FROM games WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or(ArchiveError::NotFound(id))?;

        // A review that cannot be read is measured again.
        Ok(texts.0.zip(texts.1).and_then(|(marks, points)| {
            Some(StoredReview {
                marks: serde_json::from_str(&marks).ok()?,
                points: serde_json::from_str(&points).ok()?,
            })
        }))
    }

    /// Only the player's own marks count as the game's mistakes.
    pub fn save_review(&self, id: u32, review: &StoredReview) -> Result<(), ArchiveError> {
        let human_color = self.human_color(id)?;
        let mistake_count = review
            .marks
            .iter()
            .filter(|mark| human_color.is_none_or(|color| color == mark.color))
            .count();
        let marks = serde_json::to_string(&review.marks).map_err(ArchiveError::Marks)?;
        let points = serde_json::to_string(&review.points).map_err(ArchiveError::Marks)?;
        let updated = self.connection.execute(
            "UPDATE games SET marks = ?1, points = ?2, mistake_count = ?3 WHERE id = ?4",
            params![
                marks,
                points,
                u32::try_from(mistake_count).unwrap_or(u32::MAX),
                id
            ],
        )?;

        if updated == 0 {
            return Err(ArchiveError::NotFound(id));
        }

        Ok(())
    }

    pub fn delete(&self, id: u32) -> Result<(), ArchiveError> {
        let deleted = self
            .connection
            .execute("DELETE FROM games WHERE id = ?1", [id])?;

        if deleted == 0 {
            return Err(ArchiveError::NotFound(id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gb_model::MoveQuality;
    use tempfile::tempdir;

    use super::*;

    fn sample(played_at: u32) -> NewGame {
        NewGame {
            white: "Human".to_owned(),
            black: "Stockfish 19".to_owned(),
            result: "1-0".to_owned(),
            reason: Some(GameEndReason::Checkmate),
            played_at,
            ply_count: 7,
            time_control: "300+2".to_owned(),
            pgn: "1. e4 1-0".to_owned(),
            human_color: Some(Color::White),
            opponent_elo: Some(1500),
            final_fen: Some("8/8/8/8/8/8/8/8 w - - 0 1".to_owned()),
            resume: None,
        }
    }

    fn mark(ply: u32, color: Color) -> MoveMark {
        MoveMark {
            ply,
            color,
            quality: MoveQuality::Blunder,
            best_san: "Nf3".to_owned(),
        }
    }

    fn review_of(marks: Vec<MoveMark>) -> StoredReview {
        StoredReview {
            marks,
            points: vec![EvalPoint {
                ply: 0,
                white_win_percent: 52.5,
                centipawns: Some(30),
                mate_in: None,
                is_best: false,
            }],
        }
    }

    #[test]
    fn a_saved_game_comes_back() {
        let archive = Archive::open_in_memory().unwrap();
        let id = archive.save(&sample(100)).unwrap();
        let games = archive.list().unwrap();

        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, id);
        assert_eq!(games[0].reason, Some(GameEndReason::Checkmate));
        assert_eq!(games[0].time_control, "300+2");
        assert_eq!(archive.pgn(id).unwrap(), "1. e4 1-0");
    }

    #[test]
    fn a_game_with_no_known_reason_is_stored_as_is() {
        let archive = Archive::open_in_memory().unwrap();
        let mut imported = sample(100);

        imported.reason = None;
        archive.save(&imported).unwrap();

        assert_eq!(archive.list().unwrap()[0].reason, None);
    }

    #[test]
    fn the_list_is_newest_first() {
        let archive = Archive::open_in_memory().unwrap();

        archive.save(&sample(100)).unwrap();
        archive.save(&sample(300)).unwrap();
        archive.save(&sample(200)).unwrap();

        let dates: Vec<u32> = archive
            .list()
            .unwrap()
            .iter()
            .map(|game| game.played_at)
            .collect();

        assert_eq!(dates, vec![300, 200, 100]);
    }

    #[test]
    fn delete_removes_and_a_missing_id_is_an_error() {
        let archive = Archive::open_in_memory().unwrap();
        let id = archive.save(&sample(100)).unwrap();

        archive.delete(id).unwrap();

        assert!(matches!(archive.delete(id), Err(ArchiveError::NotFound(_))));
        assert!(matches!(archive.pgn(id), Err(ArchiveError::NotFound(_))));
    }

    #[test]
    fn a_review_stays_with_its_game() {
        let archive = Archive::open_in_memory().unwrap();
        let id = archive.save(&sample(100)).unwrap();
        let review = review_of(vec![mark(3, Color::White)]);

        assert_eq!(archive.review(id).unwrap(), None);

        archive.save_review(id, &review).unwrap();

        assert_eq!(archive.review(id).unwrap(), Some(review));
        assert!(archive.list().unwrap()[0].is_reviewed);
    }

    #[test]
    fn a_measured_game_with_no_mistakes_is_not_measured_again() {
        let archive = Archive::open_in_memory().unwrap();
        let id = archive.save(&sample(100)).unwrap();
        let review = review_of(Vec::new());

        archive.save_review(id, &review).unwrap();

        assert_eq!(archive.review(id).unwrap(), Some(review));
    }

    #[test]
    fn a_review_for_a_deleted_game_is_an_error() {
        let archive = Archive::open_in_memory().unwrap();

        assert!(matches!(
            archive.save_review(9, &review_of(Vec::new())),
            Err(ArchiveError::NotFound(9))
        ));
    }

    #[test]
    fn only_the_players_own_marks_count_as_mistakes() {
        let archive = Archive::open_in_memory().unwrap();
        let id = archive.save(&sample(100)).unwrap();
        let review = review_of(vec![
            mark(1, Color::White),
            mark(2, Color::Black),
            mark(4, Color::Black),
        ]);

        archive.save_review(id, &review).unwrap();

        assert_eq!(archive.list().unwrap()[0].mistake_count, 1);
    }

    #[test]
    fn the_result_is_read_from_the_players_side() {
        let archive = Archive::open_in_memory().unwrap();
        let mut lost = sample(100);
        let mut unfinished = sample(100);
        let mut imported = sample(100);

        lost.result = "0-1".to_owned();
        unfinished.result = "*".to_owned();
        unfinished.resume = Some("{}".to_owned());
        imported.human_color = None;

        archive.save(&sample(100)).unwrap();
        archive.save(&lost).unwrap();
        archive.save(&unfinished).unwrap();
        archive.save(&imported).unwrap();

        let games = archive.list().unwrap();
        let results: Vec<PlayerResult> = games.iter().map(|game| game.player_result).collect();

        assert_eq!(
            results,
            vec![
                PlayerResult::Unknown,
                PlayerResult::Unfinished,
                PlayerResult::Loss,
                PlayerResult::Win,
            ]
        );
        assert_eq!(games[0].opponent, "Human — Stockfish 19");
        assert_eq!(games[3].opponent, "Stockfish 19");
    }

    #[test]
    fn the_stats_count_every_section() {
        let archive = Archive::open_in_memory().unwrap();
        let won = archive.save(&sample(100)).unwrap();
        let mut lost = sample(100);
        let mut drawn = sample(100);
        let mut unfinished = sample(100);

        lost.result = "0-1".to_owned();
        drawn.result = "1/2-1/2".to_owned();
        unfinished.resume = Some("{}".to_owned());
        archive.save(&lost).unwrap();
        archive.save(&drawn).unwrap();
        archive.save(&sample(100)).unwrap();
        archive.save(&unfinished).unwrap();
        archive.set_favorite(won, true).unwrap();
        archive
            .save_review(won, &review_of(vec![mark(1, Color::White)]))
            .unwrap();

        let stats = archive.stats().unwrap();

        assert_eq!(stats.total, 5);
        assert_eq!(stats.wins, 2);
        assert_eq!(stats.losses, 1);
        assert_eq!(stats.draws, 1);
        assert_eq!(stats.unfinished, 1);
        assert_eq!(stats.win_percent, 50);
        assert_eq!(stats.favorites, 1);
        assert_eq!(stats.reviewed, 1);
        assert_eq!(stats.with_mistakes, 1);
        assert!(stats.size_bytes > 0);
    }

    #[test]
    fn an_empty_archive_has_no_percent() {
        assert_eq!(stats_of(&[], 0).win_percent, 0);
    }

    #[test]
    fn a_game_played_on_is_finished_and_measured_anew() {
        let archive = Archive::open_in_memory().unwrap();
        let mut game = sample(100);

        game.result = "*".to_owned();
        game.resume = Some("{\"moves\":[]}".to_owned());

        let id = archive.save(&game).unwrap();

        assert_eq!(archive.resume_state(id).unwrap(), game.resume);

        archive.save_review(id, &review_of(Vec::new())).unwrap();
        archive.update(id, &sample(200)).unwrap();

        let saved = &archive.list().unwrap()[0];

        assert_eq!(saved.player_result, PlayerResult::Win);
        assert_eq!(saved.played_at, 200);
        assert_eq!(archive.resume_state(id).unwrap(), None);
        assert_eq!(archive.review(id).unwrap(), None);
    }

    #[test]
    fn updating_a_deleted_game_is_an_error() {
        let archive = Archive::open_in_memory().unwrap();

        assert!(matches!(
            archive.update(9, &sample(100)),
            Err(ArchiveError::NotFound(9))
        ));
    }

    #[test]
    fn games_without_a_final_position_are_found_and_filled() {
        let archive = Archive::open_in_memory().unwrap();
        let mut game = sample(100);

        game.final_fen = None;

        let id = archive.save(&game).unwrap();

        assert_eq!(archive.without_final_fen().unwrap()[0].0, id);

        archive
            .set_final_fen(id, "8/8/8/8/8/8/8/8 w - - 0 1")
            .unwrap();

        assert!(archive.without_final_fen().unwrap().is_empty());
    }

    #[test]
    fn a_database_of_the_first_version_keeps_its_games() {
        let connection = Connection::open_in_memory().unwrap();

        connection.execute_batch(CREATE_GAMES).unwrap();
        connection
            .execute_batch(
                "INSERT INTO games
                     (white, black, result, played_at, ply_count, time_control, pgn)
                 VALUES ('A', 'B', '1-0', 1, 1, '-', '1. e4 1-0');
                 PRAGMA user_version = 1;",
            )
            .unwrap();

        let archive = Archive::with_connection(connection).unwrap();
        let id = archive.list().unwrap()[0].id;

        assert_eq!(archive.review(id).unwrap(), None);
    }

    #[test]
    fn the_migration_finds_the_player_and_forgets_the_old_marks() {
        let connection = Connection::open_in_memory().unwrap();

        connection.execute_batch(CREATE_GAMES).unwrap();
        connection.execute_batch(ADD_MARKS).unwrap();
        connection
            .execute_batch(
                "INSERT INTO games
                     (white, black, result, played_at, ply_count, time_control, pgn, marks)
                 VALUES ('Stockfish 19', 'Player', '0-1', 1, 1, '-', '1. e4 0-1', '[]');
                 PRAGMA user_version = 2;",
            )
            .unwrap();

        let archive = Archive::with_connection(connection).unwrap();
        let game = &archive.list().unwrap()[0];

        assert_eq!(game.human_color, Some(Color::Black));
        assert_eq!(game.player_result, PlayerResult::Win);
        assert_eq!(game.opponent, "Stockfish 19");
        assert!(!game.is_reviewed);
    }

    #[test]
    fn games_survive_reopening_the_file() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("games.sqlite");

        Archive::open(&path).unwrap().save(&sample(100)).unwrap();

        assert_eq!(Archive::open(&path).unwrap().list().unwrap().len(), 1);
    }
}
