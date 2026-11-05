//! PGN out of a finished game and back into replay frames.

use std::fmt::Write as _;
use std::io::Cursor;
use std::ops::ControlFlow;

use gb_model::{GameResult, ReplayGame, TimeControl};
use pgn_reader::{Outcome, RawTag, Reader, SanPlus, Skip, Visitor};
use shakmaty::san::San;

use crate::error::ChessError;
use crate::game::Game;

const MS_PER_SECOND: u32 = 1_000;
const UNKNOWN_RESULT: &str = "*";
const UNKNOWN_PLAYER: &str = "?";
const LINE_WIDTH: usize = 80;
const NO_TIME_CONTROL: &str = "-";

#[derive(Debug, Clone)]
pub struct PgnHeaders {
    pub white: String,
    pub black: String,
    /// `YYYY.MM.DD`.
    pub date: String,
    pub time_control: Option<TimeControl>,
}

/// The PGN `TimeControl` value: `300+2`, or `-` for no clock.
pub fn format_time_control(control: Option<TimeControl>) -> String {
    match control {
        Some(limit) => format!(
            "{}+{}",
            limit.base_ms / MS_PER_SECOND,
            limit.increment_ms / MS_PER_SECOND
        ),
        None => NO_TIME_CONTROL.to_owned(),
    }
}

fn result_text(game: &Game) -> &'static str {
    match game.outcome().map(|outcome| outcome.result) {
        Some(GameResult::WhiteWins) => "1-0",
        Some(GameResult::BlackWins) => "0-1",
        Some(GameResult::Draw) => "1/2-1/2",
        None => UNKNOWN_RESULT,
    }
}

fn escape_tag(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn movetext(game: &Game, result: &str) -> String {
    let mut tokens = Vec::new();
    let starts_with_black = game.custom_start_fen().is_some_and(fen_has_black_to_move);
    let first_number = game.custom_start_fen().map_or(1, fen_move_number);

    for (index, record) in game.moves().iter().enumerate() {
        let half_moves = index + usize::from(starts_with_black);
        let number = first_number + half_moves / 2;
        let is_white_move = half_moves % 2 == 0;

        if is_white_move {
            tokens.push(format!("{number}."));
        } else if index == 0 {
            tokens.push(format!("{number}..."));
        }

        tokens.push(record.san.clone());
    }

    tokens.push(result.to_owned());

    wrap_tokens(&tokens)
}

fn wrap_tokens(tokens: &[String]) -> String {
    let mut text = String::new();
    let mut line_length = 0;

    for token in tokens {
        if line_length > 0 && line_length + 1 + token.len() > LINE_WIDTH {
            text.push('\n');
            line_length = 0;
        } else if line_length > 0 {
            text.push(' ');
            line_length += 1;
        }

        text.push_str(token);
        line_length += token.len();
    }

    text
}

fn fen_has_black_to_move(fen: &str) -> bool {
    fen.split_whitespace().nth(1) == Some("b")
}

fn fen_move_number(fen: &str) -> usize {
    fen.split_whitespace()
        .nth(5)
        .and_then(|number| number.parse().ok())
        .unwrap_or(1)
}

pub fn export_pgn(game: &Game, headers: &PgnHeaders) -> String {
    let result = result_text(game);
    let mut pgn = String::new();
    let mut tag = |name: &str, value: &str| {
        // Writing to a String cannot fail.
        let _ = writeln!(pgn, "[{name} \"{}\"]", escape_tag(value));
    };

    tag("Event", "Gambit");
    tag("Site", "Gambit");
    tag("Date", &headers.date);
    tag("Round", "-");
    tag("White", &headers.white);
    tag("Black", &headers.black);
    tag("Result", result);
    tag("TimeControl", &format_time_control(headers.time_control));

    if let Some(fen) = game.custom_start_fen() {
        tag("SetUp", "1");
        tag("FEN", fen);
    }

    pgn.push('\n');
    pgn.push_str(&movetext(game, result));
    pgn.push('\n');

    pgn
}

#[derive(Debug, Default)]
struct ParsedTags {
    white: Option<String>,
    black: Option<String>,
    result: Option<String>,
    fen: Option<String>,
    time_control: Option<String>,
}

#[derive(Debug, Default)]
struct ParsedGame {
    tags: ParsedTags,
    sans: Vec<San>,
    outcome: Option<String>,
}

#[derive(Debug, Default)]
struct Collector;

impl Visitor for Collector {
    type Tags = ParsedTags;
    type Movetext = ParsedGame;
    type Output = ParsedGame;

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        ControlFlow::Continue(ParsedTags::default())
    }

    fn tag(
        &mut self,
        tags: &mut Self::Tags,
        name: &[u8],
        value: RawTag<'_>,
    ) -> ControlFlow<Self::Output> {
        let text = value.decode_utf8_lossy().into_owned();

        match name {
            b"White" => tags.white = Some(text),
            b"Black" => tags.black = Some(text),
            b"Result" => tags.result = Some(text),
            b"FEN" => tags.fen = Some(text),
            b"TimeControl" => tags.time_control = Some(text),
            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(ParsedGame {
            tags,
            ..ParsedGame::default()
        })
    }

    fn san(&mut self, movetext: &mut Self::Movetext, san: SanPlus) -> ControlFlow<Self::Output> {
        movetext.sans.push(san.san);

        ControlFlow::Continue(())
    }

    /// Only the main line is replayed.
    fn begin_variation(
        &mut self,
        _movetext: &mut Self::Movetext,
    ) -> ControlFlow<Self::Output, Skip> {
        ControlFlow::Continue(Skip(true))
    }

    fn outcome(
        &mut self,
        movetext: &mut Self::Movetext,
        outcome: Outcome,
    ) -> ControlFlow<Self::Output> {
        movetext.outcome = Some(outcome.to_string());

        ControlFlow::Continue(())
    }

    fn end_game(&mut self, movetext: Self::Movetext) -> Self::Output {
        movetext
    }
}

/// Reads the first game of a PGN text into replay frames.
pub fn parse_pgn(pgn: &str) -> Result<ReplayGame, ChessError> {
    let mut reader = Reader::new(Cursor::new(pgn.as_bytes()));
    let parsed = reader
        .read_game(&mut Collector)
        .map_err(|error| ChessError::InvalidPgn(error.to_string()))?
        .ok_or_else(|| ChessError::InvalidPgn("no game found".to_owned()))?;

    let mut game = Game::new(parsed.tags.fen.as_deref())?;
    let mut moves = Vec::with_capacity(parsed.sans.len());
    let mut frames = Vec::with_capacity(parsed.sans.len() + 1);

    frames.push(game.frame());

    for san in &parsed.sans {
        moves.push(game.play_san(san)?);
        frames.push(game.frame());
    }

    let result = parsed
        .tags
        .result
        .or(parsed.outcome)
        .unwrap_or_else(|| UNKNOWN_RESULT.to_owned());

    Ok(ReplayGame {
        white: parsed
            .tags
            .white
            .unwrap_or_else(|| UNKNOWN_PLAYER.to_owned()),
        black: parsed
            .tags
            .black
            .unwrap_or_else(|| UNKNOWN_PLAYER.to_owned()),
        result,
        time_control: parsed
            .tags
            .time_control
            .unwrap_or_else(|| NO_TIME_CONTROL.to_owned()),
        human_color: None,
        moves,
        frames,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHOLARS_MATE: [&str; 7] = ["e2e4", "e7e5", "f1c4", "b8c6", "d1h5", "g8f6", "h5f7"];

    fn headers(time_control: Option<TimeControl>) -> PgnHeaders {
        PgnHeaders {
            white: "Human".to_owned(),
            black: "Stockfish \"19\"".to_owned(),
            date: "2026.11.05".to_owned(),
            time_control,
        }
    }

    #[test]
    fn time_control_header() {
        let blitz = TimeControl {
            base_ms: 300_000,
            increment_ms: 2_000,
        };

        assert_eq!(format_time_control(Some(blitz)), "300+2");
        assert_eq!(format_time_control(None), "-");
    }

    #[test]
    fn a_game_survives_export_and_import() {
        let mut game = Game::new(None).unwrap();

        for uci in SCHOLARS_MATE {
            game.play_uci(uci).unwrap();
        }

        let pgn = export_pgn(&game, &headers(None));

        assert!(pgn.contains("[Result \"1-0\"]"));
        assert!(pgn.contains("[TimeControl \"-\"]"));
        assert!(pgn.contains("[Black \"Stockfish \\\"19\\\"\"]"));
        assert!(pgn.contains("1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7# 1-0"));

        let replay = parse_pgn(&pgn).unwrap();

        assert_eq!(replay.moves, game.moves());
        assert_eq!(replay.frames.len(), SCHOLARS_MATE.len() + 1);
        assert_eq!(replay.frames.last().unwrap().fen, game.fen());
        assert_eq!(replay.result, "1-0");
        assert_eq!(replay.white, "Human");
        assert_eq!(replay.time_control, "-");
    }

    #[test]
    fn a_fen_setup_survives_with_black_to_move() {
        let fen = "4k3/8/8/8/8/8/4P3/4K3 b - - 0 12";
        let mut game = Game::new(Some(fen)).unwrap();

        game.play_uci("e8d7").unwrap();
        game.play_uci("e2e4").unwrap();

        let pgn = export_pgn(&game, &headers(None));

        assert!(pgn.contains("[SetUp \"1\"]"));
        assert!(pgn.contains(&format!("[FEN \"{fen}\"]")));
        assert!(pgn.contains("12... Kd7 13. e4 *"));

        let replay = parse_pgn(&pgn).unwrap();

        assert_eq!(replay.frames[0].fen, fen);
        assert_eq!(replay.moves.len(), 2);
    }

    #[test]
    fn variations_and_comments_are_skipped() {
        let replay = parse_pgn("1. e4 {best} (1. d4 d5) e5 2. Nf3 *").unwrap();

        assert_eq!(replay.moves.len(), 3);
        assert_eq!(replay.result, "*");
    }

    #[test]
    fn an_illegal_move_is_an_error() {
        assert!(matches!(
            parse_pgn("1. e5 *"),
            Err(ChessError::IllegalMove(_))
        ));
    }

    #[test]
    fn a_game_played_past_a_repetition_still_replays() {
        let replay = parse_pgn("1. Nf3 Nf6 2. Ng1 Ng8 3. Nf3 Nf6 4. Ng1 Ng8 5. Nf3 Nf6 *").unwrap();

        assert_eq!(replay.moves.len(), 10);
    }

    #[test]
    fn long_movetext_is_wrapped() {
        let tokens: Vec<String> = (0..60).map(|index| format!("move{index}")).collect();
        let wrapped = wrap_tokens(&tokens);

        assert!(wrapped.lines().all(|line| line.len() <= LINE_WIDTH));
    }
}
