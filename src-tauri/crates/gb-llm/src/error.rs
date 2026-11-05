use std::fmt;

#[derive(Debug)]
pub enum LlmError {
    /// Nothing answers at the address, or the connection broke.
    Unreachable(String),
    /// The server answered with a failure status.
    Rejected(String),
    /// The server answered, but not with what the API promises.
    Malformed(String),
    Timeout,
}

impl fmt::Display for LlmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(detail) => write!(formatter, "cannot reach the model: {detail}"),
            Self::Rejected(detail) => write!(formatter, "the model server refused: {detail}"),
            Self::Malformed(detail) => write!(formatter, "unexpected model reply: {detail}"),
            Self::Timeout => write!(formatter, "the model stopped answering"),
        }
    }
}

impl std::error::Error for LlmError {}
