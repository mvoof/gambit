//! The system prompt: a persona plus the facts the rules code and the engine
//! measured. The rules below keep the model from playing chess in its head.

use std::fmt::Write;

use gb_model::{Color, CompanionRole, GameOutcome};

const CENTIPAWNS_IN_A_PAWN: f64 = 100.0;
const PLIES_IN_A_ROUND: usize = 2;

const GROUND_RULES: &str = "\
You talk with a chess player inside a chess app. You cannot calculate chess and \
you must not try: you lose track of positions and invent moves. The facts below \
were measured by the rules code and a chess engine, and they are all you know.
- Mention only moves that appear in the facts. Never suggest or analyse any other move.
- Never contradict an evaluation. Do not invent evaluations, lines or threats.
- Do not name openings, players or famous games: the facts do not cover them.
- If asked something the facts do not cover, say the engine has not measured it.
- Be brief: two to four sentences, plain text, no lists and no markdown.";

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

fn persona(role: CompanionRole) -> &'static str {
    match role {
        CompanionRole::Coach => {
            "You are the player's coach: warm, patient and honest. Explain the \
             idea behind the engine's moves, at the player's level."
        }
        CompanionRole::Sparring => {
            "You are the player's sparring partner: a blunt, friendly club player \
             who pokes at their choices and asks what they would do next."
        }
        CompanionRole::Commentator => {
            "You are a live chess commentator: vivid and excited, describing the \
             drama of the game to an audience."
        }
        CompanionRole::Opponent => {
            "You are the player's opponent, sitting across the board: confident, \
             playful, a little theatrical. Banter, and keep your plans secret."
        }
    }
}

/// The opponent stays in character only if it cannot give away the best move.
pub fn sees_engine_lines(role: CompanionRole) -> bool {
    role != CompanionRole::Opponent
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

fn material_words(diff: i32) -> String {
    match diff {
        0 => "level".to_owned(),
        ahead if ahead > 0 => format!("White is ahead by {ahead}"),
        behind => format!("Black is ahead by {}", behind.abs()),
    }
}

fn score_words(line: &EngineLine) -> String {
    if let Some(moves) = line.white_mate_in {
        let mating_side = if moves > 0 { "White" } else { "Black" };

        return format!("{mating_side} mates in {}", moves.abs());
    }

    line.white_centipawns.map_or_else(
        || "not measured".to_owned(),
        |centipawns| {
            format!(
                "{:+.2} pawns from White's side",
                f64::from(centipawns) / CENTIPAWNS_IN_A_PAWN
            )
        },
    )
}

fn facts_text(facts: &PositionFacts, role: CompanionRole) -> Vec<String> {
    let moves = if facts.moves.is_empty() {
        "none".to_owned()
    } else {
        numbered_moves(&facts.moves)
    };
    let mut lines = vec![
        format!("Position (FEN): {}", facts.fen),
        format!(
            "The player is {}; their opponent is {}.",
            color_name(facts.human_color),
            facts.opponent_name
        ),
        format!("Moves so far: {moves}"),
        format!("Material: {}.", material_words(facts.material_diff)),
    ];

    if let Some(outcome) = facts.outcome {
        lines.push(format!(
            "The game is over: {:?}, {:?}.",
            outcome.result, outcome.reason
        ));
    } else {
        lines.push(format!("{} is to move.", color_name(facts.turn)));
    }

    if facts.in_check {
        lines.push(format!("{} is in check.", color_name(facts.turn)));
    }

    if !sees_engine_lines(role) {
        return lines;
    }

    if facts.lines.is_empty() {
        lines.push("The engine has no measurement of this position.".to_owned());

        return lines;
    }

    lines.push("Engine's best lines for the side to move:".to_owned());
    lines.extend(
        facts
            .lines
            .iter()
            .map(|line| format!("- {}: {}", line.moves.join(" "), score_words(line))),
    );

    lines
}

pub fn system_prompt(role: CompanionRole, language: &str, facts: &PositionFacts) -> String {
    let mut text = String::new();

    // Writing into a `String` cannot fail.
    let _ = write!(
        text,
        "{}\n\n{GROUND_RULES}\n- Always answer in {}, whatever language the player writes in.\n\nFacts:\n{}\n",
        persona(role),
        language_name(language),
        facts_text(facts, role).join("\n")
    );

    text
}

#[cfg(test)]
mod tests {
    use gb_model::{GameEndReason, GameResult};

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
        let prompt = system_prompt(CompanionRole::Coach, "ru", &facts());

        assert!(prompt.contains("Moves so far: 1. e4 e5 2. Nf3"));
        assert!(prompt.contains("Black is ahead by 3"));
        assert!(prompt.contains("Black is in check."));
        assert!(prompt.contains("- Nc6 Bb5: +0.25 pawns from White's side"));
        assert!(prompt.contains("Always answer in Russian,"));
    }

    #[test]
    fn the_opponent_never_sees_the_engines_lines() {
        let prompt = system_prompt(CompanionRole::Opponent, "en", &facts());

        assert!(!prompt.contains("Nc6"));
        assert!(!prompt.contains("pawns from White's side"));
    }

    #[test]
    fn a_missing_measurement_is_said_out_loud() {
        let empty = PositionFacts {
            lines: Vec::new(),
            ..facts()
        };
        let prompt = system_prompt(CompanionRole::Coach, "en", &empty);

        assert!(prompt.contains("no measurement"));
    }

    #[test]
    fn a_mate_score_names_the_mating_side() {
        let line = EngineLine {
            moves: vec!["Qh5".to_owned()],
            white_centipawns: None,
            white_mate_in: Some(-2),
        };

        assert_eq!(score_words(&line), "Black mates in 2");
    }

    #[test]
    fn a_finished_game_is_not_given_a_side_to_move() {
        let finished = PositionFacts {
            in_check: false,
            outcome: Some(GameOutcome {
                result: GameResult::WhiteWins,
                reason: GameEndReason::Checkmate,
            }),
            ..facts()
        };
        let prompt = system_prompt(CompanionRole::Commentator, "en", &finished);

        assert!(prompt.contains("The game is over: WhiteWins, Checkmate."));
        assert!(!prompt.contains("to move."));
    }
}
