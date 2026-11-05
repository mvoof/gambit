use std::fmt;
use std::io;

#[derive(Debug)]
pub enum EngineError {
    Spawn(String),
    Io(io::Error),
    /// The process closed its output.
    Closed,
    Timeout(&'static str),
    Download(String),
    /// The download is not the file the spec pins.
    Corrupt(String),
    Archive(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(detail) => write!(formatter, "cannot start the engine: {detail}"),
            Self::Io(error) => write!(formatter, "engine i/o failed: {error}"),
            Self::Closed => write!(formatter, "the engine closed unexpectedly"),
            Self::Timeout(stage) => write!(formatter, "the engine did not answer: {stage}"),
            Self::Download(detail) => write!(formatter, "download failed: {detail}"),
            Self::Corrupt(detail) => write!(formatter, "corrupt download: {detail}"),
            Self::Archive(detail) => write!(formatter, "cannot unpack the engine: {detail}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<io::Error> for EngineError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
