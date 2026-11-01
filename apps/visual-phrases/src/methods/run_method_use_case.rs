use serde_json::Value;
use metilplugins_plugin_sdk::{Cancel, Failure};

use super::find_phrases_use_case::find_phrases;

/// Runs an action of this plugin: `find` (when a phrase describes what a video shows).
pub fn run_method(action: &str, params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    match action {
        "find" => find_phrases(params, cancel),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has find)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_that_names_the_one_it_has() {
        let failure = run_method("count", &json!({}), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(failure.message.contains("it has find"));
    }
}
