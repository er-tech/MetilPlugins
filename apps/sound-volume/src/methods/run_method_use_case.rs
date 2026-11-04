use serde_json::Value;
use loomasplugins_plugin_sdk::Failure;

use super::find_silences_use_case::find_silent_stretches;
use super::map_volume_use_case::map_volume;

/// Runs an action of this plugin: `map` (the level over time, by band) or `silences` (the quiet
/// stretches).
pub fn run_method(action: &str, params: &Value) -> Result<Value, Failure> {
    match action {
        "map" => map_volume(params),
        "silences" => find_silent_stretches(params),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has map and silences)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_that_names_those_it_has() {
        let failure = run_method("louder", &json!({})).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(
            failure.message.contains("map and silences"),
            "{}",
            failure.message
        );
    }
}
