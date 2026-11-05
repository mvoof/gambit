//! The system prompt: the coach plus the facts the rules code and the engine
//! measured. The rules below keep the model from playing chess in its head.

use std::fmt::Write;
use std::iter::once;

use gb_model::{Color, GameOutcome, GameResult};

const CENTIPAWNS_IN_A_PAWN: f64 = 100.0;
const PLIES_IN_A_ROUND: usize = 2;
const PLAYER: &str = "the player";
const OPPONENT: &str = "the opponent";

const PERSONA: &str = "\
You are the player's chess coach and helper: warm, patient and honest. Explain \
the idea behind the engine's moves at the player's level, and call the player \
\"you\".";

const GROUND_RULES: &str = "\
You talk with a chess player inside a chess app. You cannot calculate chess and \
you must not try: you lose track of positions and invent moves. The facts below \
were measured by the rules code and a chess engine, and they are all you know.
- The only moves you may recommend are the first moves of the engine's lines. Quote \
them exactly as written there, in chess notation.
- Never describe a move in your own words and never invent one: no captures, piece \
developments or plans beyond what the engine's lines show.
- Never contradict an evaluation. Do not invent evaluations, lines or threats.
- Do not name openings, players or famous games: the facts do not cover them.
- If asked something the facts do not cover, say the engine has not measured it.
- Be brief: two to four sentences, plain text, no lists and no markdown.";

/// Stands in for a question when the coach speaks up by itself.
pub const ADVICE_REQUEST: &str = "\
Without waiting for a question, give me one short piece of advice or a comment \
on the position right now, using only the facts.";

/// A line the engine found for the side to move, in White's point of view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineLine {
    /// SAN, first move first.
    pub moves: Vec<String>,
    pub white_centipawns: Option<i32>,
    pub white_mate_in: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionFacts {
    pub fen: String,
    pub turn: Color,
    pub human_color: Color,
    pub opponent_name: String,
    /// SAN of every move up to this position.
    pub moves: Vec<String>,
    /// Positive when White is ahead, in pawns of material.
    pub material_diff: i32,
    pub in_check: bool,
    pub outcome: Option<GameOutcome>,
    /// Best first. Empty when the engine was not asked or could not answer.
    pub lines: Vec<EngineLine>,
}

fn color_name(color: Color) -> &'static str {
    match color {
        Color::White => "White",
        Color::Black => "Black",
    }
}

fn language_name(code: &str) -> &'static str {
    match code {
        "ru" => "Russian",
        _ => "English",
    }
}

fn numbered_moves(moves: &[String]) -> String {
    moves
        .chunks(PLIES_IN_A_ROUND)
        .enumerate()
        .map(|(index, round)| format!("{}. {}", index + 1, round.join(" ")))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Who a colour is, as the player sees the game.
fn side(color: Color, human_color: Color) -> &'static str {
    if color == human_color {
        PLAYER
    } else {
        OPPONENT
    }
}

/// Turns a White point of view into the player's.
fn for_player(white_value: i32, human_color: Color) -> i32 {
    match human_color {
        Color::White => white_value,
        Color::Black => -white_value,
    }
}

fn material_words(facts: &PositionFacts) -> String {
    match for_player(facts.material_diff, facts.human_color) {
        0 => "level".to_owned(),
        ahead if ahead > 0 => format!("{PLAYER} is ahead by {ahead}"),
        behind => format!("{OPPONENT} is ahead by {}", behind.abs()),
    }
}

fn score_words(line: &EngineLine, human_color: Color) -> String {
    if let Some(moves) = line.white_mate_in {
        let player_mate = for_player(moves, human_color);
        let mating_side = if player_mate > 0 { PLAYER } else { OPPONENT };

        return format!("{mating_side} mates in {}", moves.abs());
    }

    line.white_centipawns.map_or_else(
        || "not measured".to_owned(),
        |centipawns| {
            format!(
                "{:+.2}",
                f64::from(for_player(centipawns, human_color)) / CENTIPAWNS_IN_A_PAWN
            )
        },
    )
}

fn outcome_words(outcome: GameOutcome, human_color: Color) -> String {
    let result = match (outcome.result, human_color) {
        (GameResult::Draw, _) => "a draw",
        (GameResult::WhiteWins, Color::White) | (GameResult::BlackWins, Color::Black) => {
            "the player won"
        }
        _ => "the player lost",
    };

    format!("The game is over: {result}, {:?}.", outcome.reason)
}

/// The move apart from what follows it: a line read as one move is advice nobody
/// can play.
fn line_words(line: &EngineLine, human_color: Color) -> String {
    let score = score_words(line, human_color);

    match line.moves.split_first() {
        Some((first, rest)) if !rest.is_empty() => {
            format!("- {first} (then {}): {score}", rest.join(" "))
        }
        Some((first, _)) => format!("- {first}: {score}"),
        None => format!("- {score}"),
    }
}

fn engine_lines(facts: &PositionFacts) -> Vec<String> {
    if facts.lines.is_empty() {
        return vec!["The engine has no measurement of this position.".to_owned()];
    }

    let header = if facts.turn == facts.human_color {
        "The engine's best moves for the player, each with its score in pawns, where \
         plus is good for the player:"
    } else {
        "The engine's best moves for the opponent, which the player should expect, each \
         with its score in pawns, where plus is good for the player:"
    };

    once(header.to_owned())
        .chain(
            facts
                .lines
                .iter()
                .map(|line| line_words(line, facts.human_color)),
        )
        .collect()
}

fn facts_text(facts: &PositionFacts) -> Vec<String> {
    let human_color = facts.human_color;
    let moves = if facts.moves.is_empty() {
        "none".to_owned()
    } else {
        numbered_moves(&facts.moves)
    };
    let mut lines = vec![
        format!("Position (FEN): {}", facts.fen),
        format!(
            "The player plays {}. The opponent, {}, plays {}.",
            color_name(human_color),
            facts.opponent_name,
            color_name(human_color.opposite())
        ),
        format!("Moves so far: {moves}"),
    ];

    if let Some(last_move) = facts.moves.last() {
        lines.push(format!(
            "Last move: {last_move}, played by {}.",
            side(facts.turn.opposite(), human_color)
        ));
    }

    lines.push(format!("Material: {}.", material_words(facts)));

    if let Some(outcome) = facts.outcome {
        lines.push(outcome_words(outcome, human_color));
    } else if facts.turn == human_color {
        lines.push("It is the player's turn.".to_owned());
    } else {
        lines.push("It is the opponent's turn: the player waits for its move.".to_owned());
    }

    if facts.in_check {
        lines.push(format!("{} is in check.", side(facts.turn, human_color)));
    }

    lines.extend(engine_lines(facts));

    lines
}

pub fn system_prompt(language: &str, facts: &PositionFacts) -> String {
    let mut text = String::new();

    // Writing into a `String` cannot fail.
    let _ = write!(
        text,
        "{PERSONA}\n\n{GROUND_RULES}\n- Always answer in {}, whatever language the player writes in.\n\nFacts:\n{}\n",
        language_name(language),
        facts_text(facts).join("\n")
    );

    text
}

#[cfg(test)]
mod tests {
    use gb_model::GameEndReason;

    use super::*;

    fn facts() -> PositionFacts {
        PositionFacts {
            fen: "startfen".to_owned(),
            turn: Color::Black,
            human_color: Color::White,
            opponent_name: "Stockfish 19".to_owned(),
            moves: vec!["e4".to_owned(), "e5".to_owned(), "Nf3".to_owned()],
            material_diff: -3,
            in_check: true,
            outcome: None,
            lines: vec![EngineLine {
                moves: vec!["Nc6".to_owned(), "Bb5".to_owned()],
                white_centipawns: Some(25),
                white_mate_in: None,
            }],
        }
    }

    #[test]
    fn the_facts_reach_the_prompt() {
        let prompt = system_prompt("ru", &facts());

        assert!(prompt.contains("The player plays White. The opponent, Stockfish 19, plays Black."));
        assert!(prompt.contains("Moves so far: 1. e4 e5 2. Nf3"));
        assert!(prompt.contains("Last move: Nf3, played by the player."));
        assert!(prompt.contains("the opponent is ahead by 3"));
        assert!(prompt.contains("the opponent is in check."));
        assert!(prompt.contains("It is the opponent's turn"));
        assert!(prompt.contains("for the opponent, which the player should expect"));
        assert!(prompt.contains("- Nc6 (then Bb5): +0.25"));
        assert!(prompt.contains("Always answer in Russian,"));
    }

    #[test]
    fn everything_turns_around_when_the_player_has_black() {
        let black = PositionFacts {
            human_color: Color::Black,
            ..facts()
        };
        let prompt = system_prompt("en", &black);

        assert!(prompt.contains("The player plays Black. The opponent, Stockfish 19, plays White."));
        assert!(prompt.contains("Last move: Nf3, played by the opponent."));
        assert!(prompt.contains("the player is ahead by 3"));
        assert!(prompt.contains("It is the player's turn."));
        assert!(prompt.contains("The engine's best moves for the player,"));
        assert!(prompt.contains("- Nc6 (then Bb5): -0.25"));
    }

    #[test]
    fn a_missing_measurement_is_said_out_loud() {
        let empty = PositionFacts {
            lines: Vec::new(),
            ..facts()
        };

        assert!(system_prompt("en", &empty).contains("no measurement"));
    }

    #[test]
    fn a_mate_score_names_the_mating_side() {
        let line = EngineLine {
            moves: vec!["Qh5".to_owned()],
            white_centipawns: None,
            white_mate_in: Some(-2),
        };

        assert_eq!(score_words(&line, Color::White), "the opponent mates in 2");
        assert_eq!(score_words(&line, Color::Black), "the player mates in 2");
    }

    #[test]
    fn a_finished_game_says_who_won_and_gives_no_turn() {
        let finished = PositionFacts {
            in_check: false,
            human_color: Color::Black,
            outcome: Some(GameOutcome {
                result: GameResult::WhiteWins,
                reason: GameEndReason::Checkmate,
            }),
            ..facts()
        };
        let prompt = system_prompt("en", &finished);

        assert!(prompt.contains("The game is over: the player lost, Checkmate."));
        assert!(!prompt.contains("turn"));
    }
}
