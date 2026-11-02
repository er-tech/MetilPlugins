use super::signal_contract::{Signal, Source};
use super::timed_item_contract::TimedItem;

/// The strongest a hint of words can be.
const MOST: f64 = 0.9;

/// One kind of words to look for in the sentences, and how strong a hint of it is.
#[derive(Debug, Clone)]
pub struct WordList<'a> {
    /// Why a sentence with these words is flagged, in short words such as "aggressive words".
    pub reason: &'a str,
    /// The words or phrases. Whole words, ignoring case; a phrase must come in order.
    pub words: &'a [String],
    /// The strength of a hint when the sentence holds one of the words.
    pub one_word: f64,
    /// How much each further different word of the list in the same sentence adds.
    pub each_more: f64,
}

/// Splits a text into lower case words. An apostrophe stays inside a word, so "don't" is one word.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// One word of a list found in one sentence.
struct Found<'a> {
    start_ms: u64,
    end_ms: u64,
    word: &'a String,
}

/// Flags the words of each list that are said close together.
///
/// A word may be a phrase such as "let go". Items without `text` are skipped. For each list the
/// words found are taken in time order, and the ones that start within `window_ms` of the first
/// one make one signal: it covers them, and its strength starts at the list's `one_word` and grows
/// by `each_more` for each further *different* word, up to 0.9. So "Help! I can't breathe! I'm
/// going to die!", which a speech recogniser may split into three sentences, counts as three words,
/// and the same word said again counts once.
pub fn text_signals(sentences: &[TimedItem], lists: &[WordList], window_ms: u64) -> Vec<Signal> {
    let mut signals = Vec::new();
    for list in lists {
        let candidates: Vec<(&String, Vec<String>)> = list
            .words
            .iter()
            .map(|word| (word, words(word)))
            .filter(|(_, parts)| !parts.is_empty())
            .collect();
        let mut found: Vec<Found> = Vec::new();
        for sentence in sentences {
            let Some(text) = sentence.text.as_deref() else {
                continue;
            };
            let spoken = words(text);
            for (word, parts) in &candidates {
                if spoken
                    .windows(parts.len())
                    .any(|window| window == parts.as_slice())
                {
                    found.push(Found {
                        start_ms: sentence.start_ms,
                        end_ms: sentence.end_ms,
                        word,
                    });
                }
            }
        }
        found.sort_by_key(|item| item.start_ms);

        let mut next = 0;
        while next < found.len() {
            let limit = found[next].start_ms.saturating_add(window_ms);
            let run: Vec<&Found> = found[next..]
                .iter()
                .take_while(|item| item.start_ms <= limit)
                .collect();
            next += run.len();
            let mut different: Vec<&String> = Vec::new();
            for item in &run {
                if !different.contains(&item.word) {
                    different.push(item.word);
                }
            }
            let weight = (list.one_word + list.each_more * (different.len() - 1) as f64).min(MOST);
            let said: Vec<String> = different.iter().map(|word| format!("\"{word}\"")).collect();
            signals.push(Signal {
                start_ms: run.iter().map(|item| item.start_ms).min().unwrap_or(0),
                end_ms: run.iter().map(|item| item.end_ms).max().unwrap_or(0),
                source: Source::Text,
                reason: list.reason.to_owned(),
                weight,
                detail: format!("said {}", said.join(", ")),
            });
        }
    }
    signals.sort_by_key(|signal| (signal.start_ms, signal.end_ms));
    signals
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

    /// The signals for sentences, with the window of ten seconds the action uses by default.
    fn run(sentences: &[TimedItem], lists: &[WordList]) -> Vec<Signal> {
        text_signals(sentences, lists, 10_000)
    }

    fn words_of(list: &[&str]) -> Vec<String> {
        list.iter().map(|word| (*word).to_owned()).collect()
    }

    fn list<'a>(reason: &'a str, words: &'a [String]) -> WordList<'a> {
        WordList {
            reason,
            words,
            one_word: 0.45,
            each_more: 0.15,
        }
    }

    #[test]
    fn a_word_in_a_sentence_makes_one_signal_with_the_times_and_the_reason_of_the_list() {
        let words = words_of(&["help"]);
        let signals = run(
            &[sentence(1_000, "Please, HELP me!")],
            &[list("words of distress", &words)],
        );
        assert_eq!(signals.len(), 1);
        assert_eq!((signals[0].start_ms, signals[0].end_ms), (1_000, 3_000));
        assert_eq!(signals[0].reason, "words of distress");
        assert_eq!(signals[0].detail, "said \"help\"");
    }

    #[test]
    fn the_more_different_words_of_a_list_a_sentence_holds_the_stronger_the_signal() {
        let words = words_of(&["help", "stop it", "let go", "get off me", "leave me alone"]);
        let lists = [list("words of distress", &words)];
        let weight = |text: &str| run(&[sentence(0, text)], &lists)[0].weight;
        assert!((weight("Can you help me?") - 0.45).abs() < 1e-9);
        assert!((weight("Help, stop it") - 0.6).abs() < 1e-9);
        assert!((weight("Help, stop it, let go") - 0.75).abs() < 1e-9);
        assert!((weight("Help, stop it, let go, get off me, leave me alone") - 0.9).abs() < 1e-9);
        let said = &run(&[sentence(0, "Help, stop it")], &lists)[0].detail;
        assert_eq!(said, "said \"help\", \"stop it\"");
    }

    #[test]
    fn words_in_different_sentences_close_together_count_as_one_run_of_words() {
        // A speech recogniser splits "Help! I can't breathe! I'm going to die!" into sentences.
        let words = words_of(&["help", "can't breathe", "i'm going to die"]);
        let sentences = [
            sentence(1_000, "Help!"),
            sentence(3_000, "I can't breathe!"),
            sentence(5_000, "I'm going to die!"),
        ];
        let signals = run(&sentences, &[list("words of distress", &words)]);
        assert_eq!(signals.len(), 1);
        assert!((signals[0].weight - 0.75).abs() < 1e-9);
        assert_eq!((signals[0].start_ms, signals[0].end_ms), (1_000, 7_000));
        assert_eq!(
            signals[0].detail,
            "said \"help\", \"can't breathe\", \"i'm going to die\""
        );
    }

    #[test]
    fn words_further_apart_than_the_window_are_separate_signals() {
        let words = words_of(&["help", "stop it"]);
        let sentences = [sentence(0, "Help"), sentence(30_000, "Stop it")];
        let signals = text_signals(&sentences, &[list("words of distress", &words)], 10_000);
        assert_eq!(signals.len(), 2);
        assert!(signals
            .iter()
            .all(|signal| (signal.weight - 0.45).abs() < 1e-9));
        let together = text_signals(&sentences, &[list("words of distress", &words)], 60_000);
        assert_eq!(together.len(), 1);
        assert!((together[0].weight - 0.6).abs() < 1e-9);
    }

    #[test]
    fn the_same_word_said_twice_counts_once() {
        let words = words_of(&["help"]);
        let signals = run(
            &[sentence(0, "Help help help")],
            &[list("words of distress", &words)],
        );
        assert!((signals[0].weight - 0.45).abs() < 1e-9);
    }

    #[test]
    fn each_list_makes_its_own_signal_for_the_same_sentence() {
        let distress = words_of(&["help"]);
        let aggression = words_of(&["shut up"]);
        let lists = [
            list("words of distress", &distress),
            list("aggressive words", &aggression),
        ];
        let signals = run(&[sentence(0, "Help! Shut up!")], &lists);
        let reasons: Vec<&str> = signals.iter().map(|s| s.reason.as_str()).collect();
        assert_eq!(reasons, ["words of distress", "aggressive words"]);
    }

    #[test]
    fn a_list_can_start_stronger_for_words_that_say_more() {
        let words = words_of(&["want to die"]);
        let strong = WordList {
            reason: "words of distress",
            words: &words,
            one_word: 0.7,
            each_more: 0.1,
        };
        let signals = run(&[sentence(0, "I just want to die")], &[strong]);
        assert!((signals[0].weight - 0.7).abs() < 1e-9);
    }

    #[test]
    fn only_whole_words_match() {
        let words = words_of(&["help"]);
        let signals = run(
            &[sentence(0, "The helpful helper waved")],
            &[list("words of distress", &words)],
        );
        assert!(signals.is_empty());
    }

    #[test]
    fn a_phrase_matches_only_in_order() {
        let words = words_of(&["let go"]);
        let lists = [list("words of distress", &words)];
        assert_eq!(run(&[sentence(0, "Please let go of me")], &lists).len(), 1);
        assert!(run(&[sentence(0, "go and let it be")], &lists).is_empty());
    }

    #[test]
    fn a_sentence_without_text_is_skipped() {
        let item = TimedItem {
            text: None,
            ..sentence(0, "")
        };
        let words = words_of(&["help"]);
        assert!(run(&[item], &[list("words of distress", &words)]).is_empty());
    }
}
