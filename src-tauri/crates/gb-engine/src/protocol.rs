//! The text of the UCI protocol: no process in here, so all of it is testable.

use gb_model::EloRange;

const FALLBACK_BASE_ELO: u32 = 800;
const FALLBACK_ELO_PER_DEPTH: u32 = 200;
const FALLBACK_MIN_DEPTH: u32 = 1;
const FALLBACK_MAX_DEPTH: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strength {
    Full,
    Elo(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchLimit {
    Clock {
        white_ms: u32,
        black_ms: u32,
        white_increment_ms: u32,
        black_increment_ms: u32,
    },
    MoveTime(u32),
    Infinite,
}

/// What the `uci` reply said.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineIdentity {
    pub name: Option<String>,
    pub elo_range: Option<EloRange>,
    pub has_limit_strength: bool,
}

impl EngineIdentity {
    /// Folds one line of the `uci` reply in.
    pub fn read_line(&mut self, line: &str) {
        if let Some(name) = line.strip_prefix("id name ") {
            self.name = Some(name.trim().to_owned());
        } else if line.starts_with("option name UCI_LimitStrength ") {
            self.has_limit_strength = true;
        } else if line.starts_with("option name UCI_Elo ") {
            self.elo_range = parse_spin_range(line);
        }
    }

    /// The Elo range only counts with the switch that turns it on.
    pub fn usable_elo_range(&self) -> Option<EloRange> {
        self.elo_range.filter(|_| self.has_limit_strength)
    }
}

fn parse_spin_range(line: &str) -> Option<EloRange> {
    let mut tokens = line.split_whitespace();
    let mut min = None;
    let mut max = None;

    while let Some(token) = tokens.next() {
        match token {
            "min" => min = tokens.next().and_then(|value| value.parse().ok()),
            "max" => max = tokens.next().and_then(|value| value.parse().ok()),
            _ => {}
        }
    }

    Some(EloRange {
        min: min?,
        max: max?,
    })
}

/// The `setoption` lines for a strength. Empty when the engine has no switch:
/// such an engine is limited by depth instead.
pub fn strength_commands(strength: Strength, identity: &EngineIdentity) -> Vec<String> {
    let Some(range) = identity.usable_elo_range() else {
        return Vec::new();
    };

    match strength {
        Strength::Full => vec!["setoption name UCI_LimitStrength value false".to_owned()],
        Strength::Elo(elo) => vec![
            "setoption name UCI_LimitStrength value true".to_owned(),
            format!(
                "setoption name UCI_Elo value {}",
                elo.clamp(range.min, range.max)
            ),
        ],
    }
}

/// A rough depth for a wanted Elo, for an engine that cannot limit itself.
pub fn fallback_depth(strength: Strength, identity: &EngineIdentity) -> Option<u32> {
    let Strength::Elo(elo) = strength else {
        return None;
    };

    if identity.usable_elo_range().is_some() {
        return None;
    }

    let depth = elo.saturating_sub(FALLBACK_BASE_ELO) / FALLBACK_ELO_PER_DEPTH;

    Some(depth.clamp(FALLBACK_MIN_DEPTH, FALLBACK_MAX_DEPTH))
}

/// The start position plus the moves, so the engine sees repetitions.
pub fn position_command(start_fen: Option<&str>, moves: &[String]) -> String {
    let start = start_fen.map_or_else(|| "startpos".to_owned(), |fen| format!("fen {fen}"));

    if moves.is_empty() {
        format!("position {start}")
    } else {
        format!("position {start} moves {}", moves.join(" "))
    }
}

pub fn go_command(limit: SearchLimit, depth: Option<u32>) -> String {
    let depth_part = depth.map_or_else(String::new, |value| format!(" depth {value}"));

    match limit {
        SearchLimit::Clock {
            white_ms,
            black_ms,
            white_increment_ms,
            black_increment_ms,
        } => format!(
            "go wtime {white_ms} btime {black_ms} winc {white_increment_ms} binc {black_increment_ms}{depth_part}"
        ),
        SearchLimit::MoveTime(milliseconds) => format!("go movetime {milliseconds}{depth_part}"),
        SearchLimit::Infinite => "go infinite".to_owned(),
    }
}

/// `bestmove e2e4 ponder e7e5`. `None` for a position with no move.
pub fn parse_best_move(line: &str) -> Option<Option<String>> {
    let mut tokens = line.split_whitespace();

    if tokens.next() != Some("bestmove") {
        return None;
    }

    Some(
        tokens
            .next()
            .filter(|chosen| !matches!(*chosen, "(none)" | "0000"))
            .map(str::to_owned),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stockfish() -> EngineIdentity {
        let mut identity = EngineIdentity::default();

        for line in [
            "id name Stockfish 19",
            "id author the Stockfish developers",
            "option name Threads type spin default 1 min 1 max 1024",
            "option name UCI_LimitStrength type check default false",
            "option name UCI_Elo type spin default 1320 min 1320 max 3190",
            "uciok",
        ] {
            identity.read_line(line);
        }

        identity
    }

    #[test]
    fn reads_the_name_and_the_elo_range() {
        let identity = stockfish();

        assert_eq!(identity.name.as_deref(), Some("Stockfish 19"));
        assert_eq!(
            identity.usable_elo_range(),
            Some(EloRange {
                min: 1320,
                max: 3190,
            })
        );
    }

    #[test]
    fn an_elo_option_without_the_switch_does_not_count() {
        let mut identity = EngineIdentity::default();

        identity.read_line("option name UCI_Elo type spin default 1500 min 1000 max 2000");

        assert_eq!(identity.usable_elo_range(), None);
    }

    #[test]
    fn strength_is_clamped_to_the_engine_range() {
        let commands = strength_commands(Strength::Elo(500), &stockfish());

        assert_eq!(commands[1], "setoption name UCI_Elo value 1320");
    }

    #[test]
    fn full_strength_turns_the_limit_off() {
        assert_eq!(
            strength_commands(Strength::Full, &stockfish()),
            vec!["setoption name UCI_LimitStrength value false"]
        );
    }

    #[test]
    fn an_engine_without_the_switch_gets_a_depth_limit() {
        let plain = EngineIdentity::default();

        assert!(strength_commands(Strength::Elo(1600), &plain).is_empty());
        assert_eq!(fallback_depth(Strength::Elo(1600), &plain), Some(4));
        assert_eq!(fallback_depth(Strength::Elo(100), &plain), Some(1));
        assert_eq!(fallback_depth(Strength::Full, &plain), None);
        assert_eq!(fallback_depth(Strength::Elo(1600), &stockfish()), None);
    }

    #[test]
    fn position_keeps_the_moves() {
        assert_eq!(position_command(None, &[]), "position startpos");
        assert_eq!(
            position_command(Some("8/8/8/8/8/8/8/K1k5 w - - 0 1"), &["a1a2".to_owned()]),
            "position fen 8/8/8/8/8/8/8/K1k5 w - - 0 1 moves a1a2"
        );
    }

    #[test]
    fn go_carries_the_real_clock() {
        let limit = SearchLimit::Clock {
            white_ms: 1,
            black_ms: 2,
            white_increment_ms: 3,
            black_increment_ms: 4,
        };

        assert_eq!(go_command(limit, None), "go wtime 1 btime 2 winc 3 binc 4");
        assert_eq!(
            go_command(SearchLimit::MoveTime(500), Some(6)),
            "go movetime 500 depth 6"
        );
    }

    #[test]
    fn reads_the_best_move() {
        assert_eq!(
            parse_best_move("bestmove e2e4 ponder e7e5"),
            Some(Some("e2e4".to_owned()))
        );
        assert_eq!(parse_best_move("bestmove (none)"), Some(None));
        assert_eq!(parse_best_move("info depth 1"), None);
    }
}
