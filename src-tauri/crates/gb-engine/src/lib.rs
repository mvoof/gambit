//! Drives a UCI engine process and installs engines from a pinned catalogue.

mod catalog;
mod error;
mod install;
mod protocol;
mod score;
mod uci;

pub use error::EngineError;
pub use install::{
    catalog_entries, first_installed, install_engine, installed_engine_path, remove_engine,
};
pub use protocol::{SearchLimit, Strength};
pub use score::{
    accepts_draw, evaluation_of, lost_win_percent, quality_of, CandidateLine, Score, SearchInfo,
    MIN_BEST_MOVE_DEPTH,
};
pub use uci::{probe_engine, SearchOutcome, UciEngine};
