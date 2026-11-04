use loomasplugins_plugin_sdk::{Cancel, Failure};
use serde_json::Value;

use super::extract_audio_use_case::extract_audio;
use super::normalize_audio_use_case::normalize_audio;

/// Runs an action of this plugin: `extract` (the sound of a video as an audio file) or `normalize`
/// (the same at a given loudness).
pub fn run_method(action: &str, params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    match action {
        "extract" => extract_audio(params, cancel),
        "normalize" => normalize_audio(params, cancel),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has extract and normalize)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_that_names_the_ones_it_has() {
        let failure = run_method("dance", &json!({}), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(failure.message.contains("extract and normalize"));
    }
}
