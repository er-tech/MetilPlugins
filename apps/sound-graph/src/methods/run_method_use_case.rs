use serde_json::Value;
use metilplugins_plugin_sdk::Failure;

use super::draw_waveform_use_case::draw_waveform;

/// Runs an action of this plugin: `waveform` (an SVG picture of the waves of a sound).
pub fn run_method(action: &str, params: &Value) -> Result<Value, Failure> {
    match action {
        "waveform" => draw_waveform(params),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has waveform)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_that_names_the_one_it_has() {
        let failure = run_method("spectrogram", &json!({})).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(failure.message.contains("it has waveform"));
    }
}
