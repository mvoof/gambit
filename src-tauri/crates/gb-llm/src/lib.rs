//! Talks to a local language model over the OpenAI-compatible API of LM Studio.
//! The model only puts measured facts into words: nothing here plays or judges.

mod client;
mod error;
mod prompt;
mod sse;
mod think;

pub use client::{chat_models, is_model_loaded, stream_chat, ChatMessage, ChatRequest, Speaker};
pub use error::LlmError;
pub use prompt::{sees_engine_lines, system_prompt, EngineLine, PositionFacts};
