//! The server-sent events of a streaming completion. Network chunks split lines
//! and characters anywhere, so bytes wait here until a whole line has arrived.

use serde::Deserialize;

use crate::error::LlmError;

const DATA_PREFIX: &[u8] = b"data:";
const DONE_MARKER: &str = "[DONE]";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Decoded {
    pub text: String,
    pub is_done: bool,
}

#[derive(Deserialize)]
struct Chunk {
    #[serde(default)]
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Delta,
}

/// `reasoning_content` is deliberately not read: it is the model's scratchpad.
#[derive(Deserialize, Default)]
struct Delta {
    content: Option<String>,
}

#[derive(Default)]
pub struct SseDecoder {
    pending: Vec<u8>,
}

fn read_event(line: &[u8], decoded: &mut Decoded) -> Result<(), LlmError> {
    let Some(payload) = line.strip_prefix(DATA_PREFIX) else {
        return Ok(());
    };
    let payload = String::from_utf8_lossy(payload);
    let payload = payload.trim();

    if payload == DONE_MARKER {
        decoded.is_done = true;

        return Ok(());
    }

    let chunk: Chunk =
        serde_json::from_str(payload).map_err(|error| LlmError::Malformed(error.to_string()))?;

    for choice in chunk.choices {
        decoded.text.extend(choice.delta.content);
    }

    Ok(())
}

impl SseDecoder {
    pub fn push(&mut self, bytes: &[u8]) -> Result<Decoded, LlmError> {
        self.pending.extend_from_slice(bytes);

        let mut decoded = Decoded::default();
        let mut consumed = 0;

        while let Some(length) = self.pending[consumed..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            read_event(&self.pending[consumed..consumed + length], &mut decoded)?;
            consumed += length + 1;
        }

        self.pending.drain(..consumed);

        Ok(decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(content: &str) -> String {
        format!("data: {{\"choices\":[{{\"delta\":{{\"content\":\"{content}\"}}}}]}}\n\n")
    }

    #[test]
    fn reads_text_and_the_end_marker() {
        let stream = format!("{}{}data: [DONE]\n\n", event("Hel"), event("lo"));
        let decoded = SseDecoder::default().push(stream.as_bytes()).unwrap();

        assert_eq!(decoded.text, "Hello");
        assert!(decoded.is_done);
    }

    #[test]
    fn survives_a_chunk_split_inside_a_cyrillic_letter() {
        let stream = event("конь");
        let middle = stream.find('о').unwrap() + 1;
        let mut decoder = SseDecoder::default();
        let first = decoder.push(&stream.as_bytes()[..middle]).unwrap();
        let second = decoder.push(&stream.as_bytes()[middle..]).unwrap();

        assert_eq!(first.text, "");
        assert_eq!(second.text, "конь");
    }

    #[test]
    fn the_scratchpad_and_role_only_events_give_no_text() {
        let stream = concat!(
            "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"hmm\"}}]}\n\n",
            ": keep-alive\n\n",
        );

        assert_eq!(
            SseDecoder::default().push(stream.as_bytes()).unwrap(),
            Decoded::default()
        );
    }

    #[test]
    fn a_broken_event_is_an_error() {
        assert!(SseDecoder::default().push(b"data: {oops\n").is_err());
    }
}
