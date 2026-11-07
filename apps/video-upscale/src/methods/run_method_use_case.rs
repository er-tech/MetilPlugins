use loomasplugins_plugin_sdk::{Cancel, Failure};
use serde_json::Value;

use super::upscale_video_use_case::upscale_video;

/// Runs an action of this plugin: `upscale` (a larger video).
pub fn run_method(action: &str, params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    match action {
        "upscale" => upscale_video(params, cancel),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has upscale)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_that_names_the_one_it_has() {
        let failure = run_method("shrink", &json!({}), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(failure.message.contains("it has upscale"));
    }
}
