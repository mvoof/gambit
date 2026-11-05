//! The saved games: SQLite. A game is its PGN plus the columns a list needs.

use std::fmt;
use std::path::Path;

use gb_model::{GameEndReason, GameSummary, MoveMark};
use rusqlite::{params, Connection, OptionalExtension, Row};

/// Bump and add a step to `migrate` when the schema changes.
const SCHEMA_VERSION: i32 = 2;

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

fn summary_from(row: &Row<'_>) -> rusqlite::Result<GameSummary> {
    Ok(GameSummary {
        id: row.get(0)?,
        white: row.get(1)?,
        black: row.get(2)?,
        result: row.get(3)?,
        reason: reason_from(row.get(4)?),
        played_at: row.get(5)?,
        ply_count: row.get(6)?,
        time_control: row.get(7)?,
    })
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

        if version < SCHEMA_VERSION {
            self.connection
                .pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }

        Ok(())
    }

    pub fn save(&self, game: &NewGame) -> Result<u32, ArchiveError> {
        self.connection.execute(
            "INSERT INTO games
                (white, black, result, reason, played_at, ply_count, time_control, pgn)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                game.white,
                game.black,
                game.result,
                reason_text(game.reason),
                game.played_at,
                game.ply_count,
                game.time_control,
                game.pgn,
            ],
        )?;

        Ok(u32::try_from(self.connection.last_insert_rowid()).unwrap_or(u32::MAX))
    }

    /// Newest first.
    pub fn list(&self) -> Result<Vec<GameSummary>, ArchiveError> {
        let mut statement = self.connection.prepare(
            "SELECT id, white, black, result, reason, played_at, ply_count, time_control
             FROM games ORDER BY played_at DESC, id DESC",
        )?;
        let rows = statement.query_map([], summary_from)?;

        Ok(rows.collect::<Result<_, _>>()?)
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
    pub fn marks(&self, id: u32) -> Result<Option<Vec<MoveMark>>, ArchiveError> {
        let text: Option<String> = self
            .connection
            .query_row("SELECT marks FROM games WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?
            .ok_or(ArchiveError::NotFound(id))?;

        // Marks that cannot be read are measured again.
        Ok(text.and_then(|json| serde_json::from_str(&json).ok()))
    }

    pub fn save_marks(&self, id: u32, marks: &[MoveMark]) -> Result<(), ArchiveError> {
        let json = serde_json::to_string(marks).map_err(ArchiveError::Marks)?;
        let updated = self.connection.execute(
            "UPDATE games SET marks = ?1 WHERE id = ?2",
            params![json, id],
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
    fn marks_stay_with_their_game() {
        let archive = Archive::open_in_memory().unwrap();
        let id = archive.save(&sample(100)).unwrap();
        let marks = vec![MoveMark {
            ply: 3,
            quality: MoveQuality::Blunder,
            best_san: "Nf3".to_owned(),
        }];

        assert_eq!(archive.marks(id).unwrap(), None);

        archive.save_marks(id, &marks).unwrap();

        assert_eq!(archive.marks(id).unwrap(), Some(marks));
    }

    #[test]
    fn a_measured_game_with_no_mistakes_is_not_measured_again() {
        let archive = Archive::open_in_memory().unwrap();
        let id = archive.save(&sample(100)).unwrap();

        archive.save_marks(id, &[]).unwrap();

        assert_eq!(archive.marks(id).unwrap(), Some(Vec::new()));
    }

    #[test]
    fn marks_for_a_deleted_game_are_an_error() {
        let archive = Archive::open_in_memory().unwrap();

        assert!(matches!(
            archive.save_marks(9, &[]),
            Err(ArchiveError::NotFound(9))
        ));
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

        assert_eq!(archive.marks(id).unwrap(), None);
    }

    #[test]
    fn games_survive_reopening_the_file() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("games.sqlite");

        Archive::open(&path).unwrap().save(&sample(100)).unwrap();

        assert_eq!(Archive::open(&path).unwrap().list().unwrap().len(), 1);
    }
}
