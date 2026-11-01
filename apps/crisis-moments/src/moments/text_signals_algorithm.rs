use super::signal_contract::{Signal, Source};
use super::timed_item_contract::TimedItem;

/// The reason written for a sentence that holds a keyword.
const REASON: &str = "words of alarm";
/// How strong a keyword hint is. Words alone are weak: they need another input to agree.
const WEIGHT: f64 = 0.6;

/// Splits a text into lower case words. An apostrophe stays inside a word, so "don't" is one word.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Flags each sentence that holds one of the keywords as whole words, in the same order.
///
/// A keyword may be a phrase such as "let go". Items without `text` are skipped. At most one
/// signal is made per sentence, for the first keyword in the list that matches.
pub fn text_signals(sentences: &[TimedItem], keywords: &[String]) -> Vec<Signal> {
    let keyword_words: Vec<(&String, Vec<String>)> = keywords
        .iter()
        .map(|keyword| (keyword, words(keyword)))
        .filter(|(_, parts)| !parts.is_empty())
        .collect();

    sentences
        .iter()
        .filter_map(|sentence| {
            let spoken = words(sentence.text.as_deref()?);
            keyword_words
                .iter()
                .find(|(_, parts)| {
                    spoken
                        .windows(parts.len())
                        .any(|window| window == parts.as_slice())
                })
                .map(|(keyword, _)| Signal {
                    start_ms: sentence.start_ms,
                    end_ms: sentence.end_ms,
                    source: Source::Text,
                    reason: REASON.to_owned(),
                    weight: WEIGHT,
                    detail: format!("said \"{keyword}\""),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sentence(start_ms: u64, text: &str) -> TimedItem {
        TimedItem {
            start_ms,
            end_ms: start_ms + 2_000,
            text: Some(text.to_owned()),
            label: None,
            confidence: None,
            value: None,
        }
    }

    fn keywords(list: &[&str]) -> Vec<String> {
        list.iter().map(|word| (*word).to_owned()).collect()
    }

    #[test]
    fn a_keyword_in_a_sentence_makes_one_signal_with_the_times_of_the_sentence() {
        let signals = text_signals(&[sentence(1_000, "Please, HELP me!")], &keywords(&["help"]));
        assert_eq!(signals.len(), 1);
        assert_eq!((signals[0].start_ms, signals[0].end_ms), (1_000, 3_000));
        assert_eq!(signals[0].detail, "said \"help\"");
    }

    #[test]
    fn only_whole_words_match() {
        let signals = text_signals(
            &[sentence(0, "The helpful helper waved")],
            &keywords(&["help"]),
        );
        assert!(signals.is_empty());
    }

    #[test]
    fn a_phrase_matches_only_in_order() {
        let list = keywords(&["let go"]);
        assert_eq!(
            text_signals(&[sentence(0, "Please let go of me")], &list).len(),
            1
        );
        assert!(text_signals(&[sentence(0, "go and let it be")], &list).is_empty());
    }

    #[test]
    fn a_sentence_without_text_is_skipped() {
        let item = TimedItem {
            text: None,
            ..sentence(0, "")
        };
        assert!(text_signals(&[item], &keywords(&["help"])).is_empty());
    }
}
