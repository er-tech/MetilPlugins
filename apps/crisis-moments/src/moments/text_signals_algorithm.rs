use super::signal_contract::{Signal, Source};
use super::timed_item_contract::TimedItem;

/// The reason written for a sentence that holds a keyword.
const REASON: &str = "words of alarm";
/// How strong a hint is when a sentence holds one keyword. Words alone are weak: one word such as
/// "help" is often only a word, so it stays under the default threshold and needs another input
/// to agree.
const ONE_WORD: f64 = 0.45;
/// How much each further keyword in the same sentence adds, up to `MOST`.
const EACH_MORE: f64 = 0.15;
/// The strongest a hint of words can be.
const MOST: f64 = 0.9;

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
/// signal is made per sentence. Its strength grows with the number of different keywords in the
/// sentence: one gives 0.45, two 0.6, three 0.75, four or more 0.9.
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
            let mut found: Vec<&String> = Vec::new();
            for (keyword, parts) in &keyword_words {
                let present = spoken
                    .windows(parts.len())
                    .any(|window| window == parts.as_slice());
                if present && !found.contains(keyword) {
                    found.push(keyword);
                }
            }
            if found.is_empty() {
                return None;
            }
            let weight = (ONE_WORD + EACH_MORE * (found.len() - 1) as f64).min(MOST);
            let said: Vec<String> = found
                .iter()
                .map(|keyword| format!("\"{keyword}\""))
                .collect();
            Some(Signal {
                start_ms: sentence.start_ms,
                end_ms: sentence.end_ms,
                source: Source::Text,
                reason: REASON.to_owned(),
                weight,
                detail: format!("said {}", said.join(", ")),
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
    fn the_more_different_keywords_a_sentence_holds_the_stronger_the_signal() {
        let list = keywords(&["help", "police", "fire", "bleeding", "knife"]);
        let weight = |text: &str| text_signals(&[sentence(0, text)], &list)[0].weight;
        assert!((weight("Can you help me?") - 0.45).abs() < 1e-9);
        assert!((weight("Help, call the police") - 0.6).abs() < 1e-9);
        assert!((weight("Help, the police, a fire") - 0.75).abs() < 1e-9);
        assert!((weight("Help, police, fire, bleeding, a knife") - 0.9).abs() < 1e-9);
        let said = &text_signals(&[sentence(0, "Help, call the police")], &list)[0].detail;
        assert_eq!(said, "said \"help\", \"police\"");
    }

    #[test]
    fn the_same_keyword_said_twice_counts_once() {
        let signals = text_signals(&[sentence(0, "Help help help")], &keywords(&["help"]));
        assert!((signals[0].weight - 0.45).abs() < 1e-9);
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
