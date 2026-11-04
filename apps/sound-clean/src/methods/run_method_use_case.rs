use serde_json::Value;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::reduce_noise_use_case::reduce_noise_in_sound;

/// Runs an action of this plugin: `reduceNoise` (a sound with its steady noise taken out).
pub fn run_method(action: &str, params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    match action {
        "reduceNoise" => reduce_noise_in_sound(params, cancel),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has reduceNoise)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_that_names_the_one_it_has() {
        let failure = run_method("denoise", &json!({}), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(failure.message.contains("it has reduceNoise"));
    }
}
