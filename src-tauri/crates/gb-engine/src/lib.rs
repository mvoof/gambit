//! Drives a UCI engine process and installs Stockfish.

mod error;
mod install;
mod protocol;
mod score;
mod uci;

pub use error::EngineError;
pub use install::{install_stockfish, stockfish_path};
pub use protocol::{SearchLimit, Strength};
pub use score::{accepts_draw, evaluation_of, Score, SearchInfo};
pub use uci::{probe_engine, SearchOutcome, UciEngine};
