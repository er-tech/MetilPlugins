use serde_json::Value;
use metilplugins_plugin_sdk::{Cancel, Failure};

use super::pick_thumbnails_use_case::pick_thumbnails;

/// Runs an action of this plugin: `pick` (the pictures that show a video best).
pub fn run_method(action: &str, params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    match action {
        "pick" => pick_thumbnails(params, cancel),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has pick)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_that_names_the_one_it_has() {
        let failure = run_method("sheet", &json!({}), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(failure.message.contains("it has pick"));
    }
}
