use serde_json::Value;
use loomasplugins_plugin_sdk::Failure;

use super::find_moments_use_case::find_moments;

/// Runs an action of this plugin: `find` (the moments to review).
///
/// # Errors
/// A 404 failure for any other action name, and whatever failure the action gives.
pub fn run_method(action: &str, params: &Value) -> Result<Value, Failure> {
    match action {
        "find" => find_moments(params),
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
        let failure = run_method("judge", &json!({})).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(failure.message.contains("find"), "{}", failure.message);
    }
}
