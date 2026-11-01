use serde_json::Value;
use metilplugins_plugin_sdk::{number, whole_number, Failure};

use super::find_options_contract::{FindOptions, DEFAULT_KEYWORDS, DEFAULT_SOUND_NAMES};

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

/// Reads the settings of `find` from the params and fills in the defaults.
///
/// Params: `keywords`, `soundNames` (lists of words), `loudnessJumpDb` (1 to 60, default 15),
/// `mergeGapMs` (default 2000), `minMs` (default 0), `threshold` (0 to 1, default 0.5).
///
/// # Errors
/// A 400 failure that names the param when a value is of the wrong kind or out of range.
pub fn read_find_options(params: &Value) -> Result<FindOptions, Failure> {
    Ok(FindOptions {
        keywords: words_or(params, "keywords", &DEFAULT_KEYWORDS)?,
        sound_names: words_or(params, "soundNames", &DEFAULT_SOUND_NAMES)?,
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
    fn nothing_given_means_the_defaults() {
        let options = read_find_options(&json!({})).unwrap();
        assert_eq!(options.keywords.len(), DEFAULT_KEYWORDS.len());
        assert!((options.loudness_jump_db - 15.0).abs() < 1e-9);
        assert_eq!(options.merge_gap_ms, 2_000);
    }

    #[test]
    fn a_given_list_replaces_the_default_list() {
        let options = read_find_options(&json!({ "keywords": ["socorro"] })).unwrap();
        assert_eq!(options.keywords, vec!["socorro".to_owned()]);
    }

    #[test]
    fn a_wrong_value_is_a_400_that_names_the_param() {
        let failure = read_find_options(&json!({ "threshold": 2 })).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        assert!(failure.message.contains("threshold"), "{}", failure.message);
        let failure = read_find_options(&json!({ "keywords": [3] })).unwrap_err();
        assert!(failure.message.contains("keywords"), "{}", failure.message);
    }
}
