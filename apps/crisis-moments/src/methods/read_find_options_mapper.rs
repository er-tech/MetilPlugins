use loomasplugins_plugin_sdk::{number, whole_number, Failure};
use serde_json::Value;

use super::find_options_contract::{FindOptions, DEFAULT_SOUND_NAMES};
use super::keyword_lists_config::{word_lists_for, WordLists, LANGUAGES};

/// A list of words from a param, or the defaults when the param is missing or `null`.
fn words_or(params: &Value, key: &str, defaults: &[&str]) -> Result<Vec<String>, Failure> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(defaults.iter().map(|word| (*word).to_owned()).collect()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str().map(str::to_owned).ok_or_else(|| {
                    Failure::status(400, format!("{key} must be a list of words, not {item}"))
                })
            })
            .collect(),
        Some(other) => Err(Failure::status(
            400,
            format!("{key} must be a list of words, not {other}"),
        )),
    }
}

/// The built-in lists of the `languages` asked for (default English).
fn read_languages(params: &Value) -> Result<Vec<WordLists>, Failure> {
    words_or(params, "languages", &["en"])?
        .iter()
        .map(|code| {
            word_lists_for(code).ok_or_else(|| {
                Failure::status(
                    400,
                    format!(
                        "languages holds {code:?}, which has no list; the languages are {}",
                        LANGUAGES.join(", ")
                    ),
                )
            })
        })
        .collect()
}

/// One kind of words: the list the user gives in `key` (it replaces every built-in list of that kind), or
/// else the built-in lists of all the languages asked for, each word once.
fn read_words(
    params: &Value,
    key: &str,
    languages: &[WordLists],
    pick: fn(&WordLists) -> &'static [&'static str],
) -> Result<Vec<String>, Failure> {
    if !matches!(params.get(key), None | Some(Value::Null)) {
        return words_or(params, key, &[]);
    }
    let mut words: Vec<String> = Vec::new();
    for lists in languages {
        for word in pick(lists) {
            if !words.iter().any(|known| known == word) {
                words.push((*word).to_owned());
            }
        }
    }
    Ok(words)
}

/// Reads the settings of `find` from the params and fills in the defaults.
///
/// Params: `languages` (codes whose built-in word lists are used, default `["en"]`),
/// `distressWords`, `selfHarmWords`, `aggressionWords` (each replaces the built-in lists of its
/// kind), `soundNames` (a list of words), `wordsWindowMs` (1000 to 600000, default 10000: words said
/// this close together count together), `loudnessJumpDb` (1 to 60, default 15), `mergeGapMs`
/// (default 2000), `minMs` (default 0), `threshold` (0 to 1, default 0.5).
///
/// # Errors
/// A 400 failure that names the param when a value is of the wrong kind or out of range.
pub fn read_find_options(params: &Value) -> Result<FindOptions, Failure> {
    let languages = read_languages(params)?;
    Ok(FindOptions {
        distress_words: read_words(params, "distressWords", &languages, |lists| lists.distress)?,
        self_harm_words: read_words(params, "selfHarmWords", &languages, |lists| lists.self_harm)?,
        aggression_words: read_words(params, "aggressionWords", &languages, |lists| {
            lists.aggression
        })?,
        sound_names: words_or(params, "soundNames", &DEFAULT_SOUND_NAMES)?,
        words_window_ms: whole_number(params, "wordsWindowMs", 10_000, 1_000, 600_000)?,
        loudness_jump_db: number(params, "loudnessJumpDb", 15.0, 1.0, 60.0)?,
        merge_gap_ms: whole_number(params, "mergeGapMs", 2_000, 0, 3_600_000)?,
        min_ms: whole_number(params, "minMs", 0, 0, 3_600_000)?,
        threshold: number(params, "threshold", 0.5, 0.0, 1.0)?,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_means_english_lists_and_the_default_settings() {
        let options = read_find_options(&json!({})).unwrap();
        let english = word_lists_for("en").unwrap();
        assert_eq!(options.distress_words.len(), english.distress.len());
        assert_eq!(options.self_harm_words.len(), english.self_harm.len());
        assert_eq!(options.aggression_words.len(), english.aggression.len());
        assert_eq!(options.sound_names.len(), DEFAULT_SOUND_NAMES.len());
        assert!((options.loudness_jump_db - 15.0).abs() < 1e-9);
        assert_eq!(options.merge_gap_ms, 2_000);
        assert!((options.threshold - 0.5).abs() < 1e-9);
    }

    #[test]
    fn the_languages_asked_for_make_the_lists_each_word_once() {
        let options = read_find_options(&json!({ "languages": ["pt", "es", "pt"] })).unwrap();
        assert!(options.distress_words.contains(&"socorro".to_owned()));
        assert!(options.distress_words.contains(&"tengo miedo".to_owned()));
        assert!(options.aggression_words.contains(&"cala a boca".to_owned()));
        assert_eq!(
            options
                .distress_words
                .iter()
                .filter(|word| *word == "socorro")
                .count(),
            1,
            "a word two languages share is listed once"
        );
        assert!(
            !options.distress_words.contains(&"help".to_owned()),
            "English is only there when asked for"
        );
    }

    #[test]
    fn a_list_the_user_gives_replaces_the_built_in_lists_of_its_kind_only() {
        let options = read_find_options(&json!({
            "languages": ["fr"], "aggressionWords": ["x"], "distressWords": []
        }))
        .unwrap();
        assert_eq!(options.aggression_words, vec!["x".to_owned()]);
        assert!(options.distress_words.is_empty());
        assert!(options
            .self_harm_words
            .contains(&"je veux mourir".to_owned()));
    }

    #[test]
    fn a_language_with_no_list_is_a_400_that_lists_the_ones_there_are() {
        let failure = read_find_options(&json!({ "languages": ["xx"] })).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        assert!(
            failure.message.contains("en, pt, es, fr, de"),
            "{}",
            failure.message
        );
    }

    #[test]
    fn a_wrong_value_is_a_400_that_names_the_param() {
        let failure = read_find_options(&json!({ "threshold": 2 })).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        assert!(failure.message.contains("threshold"), "{}", failure.message);
        let failure = read_find_options(&json!({ "aggressionWords": [3] })).unwrap_err();
        assert!(
            failure.message.contains("aggressionWords"),
            "{}",
            failure.message
        );
    }
}
