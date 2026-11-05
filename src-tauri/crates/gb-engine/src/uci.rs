//! The engine process. Reading and writing are boxed, so tests drive a fake.

use std::collections::BTreeMap;
use std::future::Future;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use gb_model::EngineInfo;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, Command};
use tokio::time::timeout;

use crate::error::EngineError;
use crate::protocol::{
    fallback_depth, go_command, parse_best_move, position_command, strength_commands,
    EngineIdentity, SearchLimit, Strength,
};
use crate::score::{parse_candidate, parse_info, CandidateLine, Score, SearchInfo};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(5);
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

type Writer = Box<dyn AsyncWrite + Send + Unpin>;
type Reader = Box<dyn AsyncBufRead + Send + Unpin>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOutcome {
    /// `None` when the search was cancelled, or the position has no move.
    pub best_move: Option<String>,
    /// From the point of view of the side that searched.
    pub score: Option<Score>,
}

pub struct UciEngine {
    writer: Writer,
    lines: Lines<Reader>,
    identity: EngineIdentity,
    strength: Strength,
    path: String,
    // Held so that dropping the engine kills the process.
    _child: Option<Child>,
}

impl UciEngine {
    pub async fn start(path: &Path) -> Result<Self, EngineError> {
        let mut command = Command::new(path);

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);

        let mut child = command
            .spawn()
            .map_err(|error| EngineError::Spawn(error.to_string()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| EngineError::Spawn("no stdin".to_owned()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| EngineError::Spawn("no stdout".to_owned()))?;

        Self::connect(
            Box::new(stdin),
            Box::new(BufReader::new(stdout)),
            Some(child),
            path.display().to_string(),
        )
        .await
    }

    async fn connect(
        writer: Writer,
        reader: Reader,
        child: Option<Child>,
        path: String,
    ) -> Result<Self, EngineError> {
        let mut engine = Self {
            writer,
            lines: reader.lines(),
            identity: EngineIdentity::default(),
            strength: Strength::Full,
            path,
            _child: child,
        };

        engine.send("uci").await?;

        timeout(HANDSHAKE_TIMEOUT, engine.read_identity())
            .await
            .map_err(|_| EngineError::Timeout("uci"))??;

        Ok(engine)
    }

    async fn send(&mut self, command: &str) -> Result<(), EngineError> {
        self.writer.write_all(command.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;

        Ok(())
    }

    async fn next_line(&mut self) -> Result<String, EngineError> {
        self.lines.next_line().await?.ok_or(EngineError::Closed)
    }

    async fn read_identity(&mut self) -> Result<(), EngineError> {
        loop {
            let line = self.next_line().await?;

            if line.trim() == "uciok" {
                return Ok(());
            }

            self.identity.read_line(&line);
        }
    }

    async fn read_until_ready(&mut self) -> Result<(), EngineError> {
        while self.next_line().await?.trim() != "readyok" {}

        Ok(())
    }

    async fn sync(&mut self) -> Result<(), EngineError> {
        self.send("isready").await?;

        timeout(HANDSHAKE_TIMEOUT, self.read_until_ready())
            .await
            .map_err(|_| EngineError::Timeout("isready"))?
    }

    pub fn info(&self) -> EngineInfo {
        let fallback_name = || {
            Path::new(&self.path)
                .file_stem()
                .map_or_else(|| self.path.clone(), |stem| stem.to_string_lossy().into())
        };

        EngineInfo {
            name: self.identity.name.clone().unwrap_or_else(fallback_name),
            path: self.path.clone(),
            elo_range: self.identity.usable_elo_range(),
        }
    }

    pub async fn set_strength(&mut self, strength: Strength) -> Result<(), EngineError> {
        self.strength = strength;

        for command in strength_commands(strength, &self.identity) {
            self.send(&command).await?;
        }

        self.sync().await
    }

    pub async fn new_game(&mut self) -> Result<(), EngineError> {
        self.send("ucinewgame").await?;
        self.sync().await
    }

    /// Searches until `bestmove`. When `cancel` resolves, the engine is told to
    /// stop and its reply is read and thrown away, so the next search starts clean.
    pub async fn search(
        &mut self,
        start_fen: Option<&str>,
        moves: &[String],
        limit: SearchLimit,
        cancel: impl Future<Output = ()>,
        mut on_info: impl FnMut(SearchInfo),
    ) -> Result<SearchOutcome, EngineError> {
        let depth = fallback_depth(self.strength, &self.identity);
        let go = go_command(limit, depth);

        self.send(&position_command(start_fen, moves)).await?;
        self.send(&go).await?;

        let mut score = None;

        tokio::pin!(cancel);

        loop {
            let line = tokio::select! {
                line = self.next_line() => line?,
                () = &mut cancel => {
                    self.send("stop").await?;

                    return self.drain_after_stop(score).await;
                }
            };

            if let Some(info) = parse_info(&line) {
                score = Some(info.score);
                on_info(info);
            } else if let Some(best_move) = parse_best_move(&line) {
                return Ok(SearchOutcome { best_move, score });
            }
        }
    }

    async fn read_candidates(&mut self) -> Result<Vec<CandidateLine>, EngineError> {
        let mut lines = BTreeMap::new();

        loop {
            let line = self.next_line().await?;

            // A later line of a rank comes from a deeper search and replaces it.
            if let Some((rank, candidate)) = parse_candidate(&line) {
                lines.insert(rank, candidate);
            } else if parse_best_move(&line).is_some() {
                return Ok(lines.into_values().collect());
            }
        }
    }

    async fn set_line_count(&mut self, count: u32) -> Result<(), EngineError> {
        self.send(&format!("setoption name MultiPV value {count}"))
            .await?;
        self.sync().await
    }

    /// Searches `fen` until `cancel` resolves or the search ends by itself, with
    /// the lines so far, best first, after every line the engine reports.
    pub async fn search_lines(
        &mut self,
        fen: &str,
        count: u32,
        cancel: impl Future<Output = ()>,
        mut on_lines: impl FnMut(&[CandidateLine]),
    ) -> Result<(), EngineError> {
        self.set_line_count(count).await?;
        self.send(&position_command(Some(fen), &[])).await?;
        self.send(&go_command(SearchLimit::Infinite, None)).await?;

        let mut lines = BTreeMap::new();

        tokio::pin!(cancel);

        loop {
            let line = tokio::select! {
                line = self.next_line() => line?,
                () = &mut cancel => {
                    self.send("stop").await?;

                    return self.drain_after_stop(None).await.map(|_| ());
                }
            };

            if let Some((rank, candidate)) = parse_candidate(&line) {
                lines.insert(rank, candidate);
                on_lines(&lines.values().cloned().collect::<Vec<_>>());
            } else if parse_best_move(&line).is_some() {
                return Ok(());
            }
        }
    }

    /// The engine's `count` best lines, best first. An engine without `MultiPV`
    /// gives one line, and a finished position gives none.
    pub async fn candidates(
        &mut self,
        fen: &str,
        count: u32,
        think_ms: u32,
    ) -> Result<Vec<CandidateLine>, EngineError> {
        self.set_line_count(count).await?;
        self.send(&position_command(Some(fen), &[])).await?;
        self.send(&go_command(SearchLimit::MoveTime(think_ms), None))
            .await?;

        timeout(
            Duration::from_millis(u64::from(think_ms)) + STOP_TIMEOUT,
            self.read_candidates(),
        )
        .await
        .map_err(|_| EngineError::Timeout("candidates"))?
    }

    async fn drain_after_stop(
        &mut self,
        score: Option<Score>,
    ) -> Result<SearchOutcome, EngineError> {
        timeout(STOP_TIMEOUT, async {
            while parse_best_move(&self.next_line().await?).is_none() {}

            Ok(SearchOutcome {
                best_move: None,
                score,
            })
        })
        .await
        .map_err(|_| EngineError::Timeout("stop"))?
    }

    pub async fn quit(&mut self) -> Result<(), EngineError> {
        self.send("quit").await
    }
}

/// Starts an engine only to read its name and Elo range.
pub async fn probe_engine(path: &Path) -> Result<EngineInfo, EngineError> {
    let mut engine = UciEngine::start(path).await?;
    let info = engine.info();

    engine.quit().await?;

    Ok(info)
}

#[cfg(test)]
mod tests {
    use std::future::pending;

    use tokio::io::{duplex, split, DuplexStream};
    use tokio::sync::oneshot;

    use super::*;

    const PIPE_BYTES: usize = 4096;
    const FAKE_ENGINE: &str = "/engines/fake.exe";

    /// Answers each command it reads with the scripted reply lines.
    async fn fake_engine(stream: DuplexStream, reply: fn(&str) -> Vec<String>) {
        let (reader, mut writer) = split(stream);
        let mut lines = BufReader::new(reader).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            for answer in reply(&line) {
                let text = format!("{answer}\n");

                if writer.write_all(text.as_bytes()).await.is_err() {
                    return;
                }
            }
        }
    }

    fn standard_reply(line: &str) -> Vec<String> {
        match line {
            "uci" => vec![
                "id name Fake 1".to_owned(),
                "option name UCI_LimitStrength type check default false".to_owned(),
                "option name UCI_Elo type spin default 1500 min 1000 max 2000".to_owned(),
                "uciok".to_owned(),
            ],
            "isready" => vec!["readyok".to_owned()],
            command if command.starts_with("go") => vec![
                "info depth 3 score cp 25 pv e2e4".to_owned(),
                "bestmove e2e4".to_owned(),
            ],
            _ => Vec::new(),
        }
    }

    fn ranked_reply(line: &str) -> Vec<String> {
        if line.starts_with("go") {
            return vec![
                "info depth 1 multipv 1 score cp 5 pv d2d4".to_owned(),
                "info depth 2 multipv 2 score cp 1 pv g1f3 d7d5".to_owned(),
                "info depth 2 multipv 1 score cp 30 pv e2e4 e7e5".to_owned(),
                "bestmove e2e4".to_owned(),
            ];
        }

        standard_reply(line)
    }

    fn silent_search_reply(line: &str) -> Vec<String> {
        if line.starts_with("go") {
            return vec!["info depth 1 score cp 5".to_owned()];
        }

        if line == "stop" {
            return vec!["bestmove d2d4".to_owned()];
        }

        standard_reply(line)
    }

    #[tokio::test]
    async fn search_lines_reports_every_line_best_first() {
        let mut engine = connected(ranked_reply).await;
        let mut last = Vec::new();

        engine
            .search_lines("4k3/8/8/8/8/8/8/4K2R w - - 0 1", 2, pending(), |lines| {
                last = lines.to_vec()
            })
            .await
            .unwrap();

        assert_eq!(last.len(), 2);
        assert_eq!(last[0].moves, vec!["e2e4", "e7e5"]);
        assert_eq!(last[1].moves, vec!["g1f3", "d7d5"]);
    }

    #[tokio::test]
    async fn candidates_come_best_first_from_the_deepest_search() {
        let mut engine = connected(ranked_reply).await;
        let lines = engine
            .candidates("4k3/8/8/8/8/8/8/4K2R w - - 0 1", 2, 10)
            .await
            .unwrap();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].moves, vec!["e2e4", "e7e5"]);
        assert_eq!(lines[0].score, Score::Centipawns(30));
        assert_eq!(lines[1].moves, vec!["g1f3", "d7d5"]);
    }

    async fn connected(reply: fn(&str) -> Vec<String>) -> UciEngine {
        let (ours, theirs) = duplex(PIPE_BYTES);
        let (reader, writer) = split(ours);

        tokio::spawn(fake_engine(theirs, reply));

        UciEngine::connect(
            Box::new(writer),
            Box::new(BufReader::new(reader)),
            None,
            FAKE_ENGINE.to_owned(),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn the_handshake_reads_the_identity() {
        let engine = connected(standard_reply).await;
        let info = engine.info();

        assert_eq!(info.name, "Fake 1");
        assert_eq!(info.elo_range.map(|range| range.max), Some(2000));
    }

    #[tokio::test]
    async fn a_search_returns_the_move_and_the_last_score() {
        let mut engine = connected(standard_reply).await;

        engine.new_game().await.unwrap();

        let mut depths = Vec::new();
        let outcome = engine
            .search(None, &[], SearchLimit::MoveTime(10), pending(), |info| {
                depths.push(info.depth);
            })
            .await
            .unwrap();

        assert_eq!(outcome.best_move.as_deref(), Some("e2e4"));
        assert_eq!(outcome.score, Some(Score::Centipawns(25)));
        assert_eq!(depths, vec![3]);
    }

    #[tokio::test]
    async fn a_cancelled_search_drops_the_reply_and_the_engine_stays_usable() {
        let mut engine = connected(silent_search_reply).await;
        let (cancel, cancelled) = oneshot::channel::<()>();

        cancel.send(()).unwrap();

        let outcome = engine
            .search(
                None,
                &[],
                SearchLimit::Infinite,
                async {
                    let _ = cancelled.await;
                },
                |_| {},
            )
            .await
            .unwrap();

        assert_eq!(outcome.best_move, None);

        engine.sync().await.unwrap();
    }

    #[tokio::test]
    async fn a_dead_engine_is_an_error() {
        let (ours, theirs) = duplex(PIPE_BYTES);
        let (reader, writer) = split(ours);

        drop(theirs);

        let result = UciEngine::connect(
            Box::new(writer),
            Box::new(BufReader::new(reader)),
            None,
            FAKE_ENGINE.to_owned(),
        )
        .await;

        assert!(result.is_err());
    }
}
