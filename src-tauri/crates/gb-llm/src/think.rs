//! Models that reason aloud may put `<think>…</think>` inside the answer, when
//! the server does not split it off. It must not reach the chat or, later, speech.

use std::mem::take;

const OPEN_TAG: &str = "<think>";
const CLOSE_TAG: &str = "</think>";

#[derive(Default)]
pub struct ThinkFilter {
    held: String,
    is_inside: bool,
}

/// How many bytes at the end of `text` could be the start of `tag`.
fn partial_tag_length(text: &str, tag: &str) -> usize {
    (1..tag.len())
        .rev()
        .find(|length| text.ends_with(&tag[..*length]))
        .unwrap_or(0)
}

impl ThinkFilter {
    /// What may be shown of `piece`. A possible tag start is held back.
    pub fn push(&mut self, piece: &str) -> String {
        self.held.push_str(piece);

        let mut shown = String::new();

        loop {
            let tag = if self.is_inside { CLOSE_TAG } else { OPEN_TAG };

            if let Some(start) = self.held.find(tag) {
                if !self.is_inside {
                    shown.push_str(&self.held[..start]);
                }

                self.held.drain(..start + tag.len());
                self.is_inside = !self.is_inside;
            } else {
                let safe = self.held.len() - partial_tag_length(&self.held, tag);

                if !self.is_inside {
                    shown.push_str(&self.held[..safe]);
                }

                self.held.drain(..safe);

                return shown;
            }
        }
    }

    /// The stream ended: a held `<thi` was just text.
    pub fn finish(&mut self) -> String {
        let rest = take(&mut self.held);

        if self.is_inside {
            String::new()
        } else {
            rest
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filtered(pieces: &[&str]) -> String {
        let mut filter = ThinkFilter::default();
        let mut shown: String = pieces.iter().map(|piece| filter.push(piece)).collect();

        shown.push_str(&filter.finish());

        shown
    }

    #[test]
    fn plain_text_passes() {
        assert_eq!(filtered(&["Конь ", "на f3"]), "Конь на f3");
    }

    #[test]
    fn a_thought_is_dropped_even_when_split() {
        assert_eq!(
            filtered(&["<th", "ink>so the user", " wants</th", "ink>\n\nHello"]),
            "\n\nHello"
        );
    }

    #[test]
    fn a_lone_angle_bracket_is_kept() {
        assert_eq!(filtered(&["a <", " b"]), "a < b");
        assert_eq!(filtered(&["ends with <thi"]), "ends with <thi");
    }

    #[test]
    fn an_unfinished_thought_shows_nothing() {
        assert_eq!(filtered(&["<think>still going"]), "");
    }
}
