use serde_json::Value;
use metilplugins_onnx_device::DeviceChoice;
use metilplugins_plugin_sdk::{number, text, whole_number, Failure};

use super::find_options_contract::FindOptions;

/// The most phrases one call can look for, and the longest one.
const MAX_PHRASES: usize = 16;
const MAX_PHRASE_CHARS: usize = 200;

/// The params of `find` as options: `phrases` (1 to 16 texts of up to 200 characters), `minConfidence`
/// (0.01 to 1, default 0.1), `gapMs` (0 to ten minutes, default 2000) and `minSamples` (1 to 100,
/// default 1). Anything outside its range is a 400 that says what was wrong.
pub fn read_find_options(params: &Value) -> Result<FindOptions, Failure> {
    let list = params
        .get("phrases")
        .and_then(Value::as_array)
        .filter(|list| !list.is_empty())
        .ok_or_else(|| Failure::status(400, "phrases must be a list with at least one text"))?;
    if list.len() > MAX_PHRASES {
        return Err(Failure::status(
            400,
            format!(
                "phrases has {} texts, and {MAX_PHRASES} is the most",
                list.len()
            ),
        ));
    }
    let mut phrases = Vec::with_capacity(list.len());
    for phrase in list {
        let text = phrase
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty());
        let text = text.ok_or_else(|| {
            Failure::status(
                400,
                format!("{phrase} is not a phrase: give a text with words in it"),
            )
        })?;
        if text.chars().count() > MAX_PHRASE_CHARS {
            return Err(Failure::status(
                400,
                format!("a phrase is at most {MAX_PHRASE_CHARS} characters: {text:?}"),
            ));
        }
        phrases.push(text.to_string());
    }
    let device_text = text(params, "device", Some("cpu"), 12)?.unwrap_or_default();
    let device = DeviceChoice::parse(&device_text).ok_or_else(|| {
        Failure::status(
            400,
            format!(
                "device must be auto, cpu, gpu or gpu:N (an adapter number), not {device_text:?}"
            ),
        )
    })?;
    Ok(FindOptions {
        phrases,
        device,
        min_confidence: number(params, "minConfidence", 0.1, 0.01, 1.0)? as f32,
        gap_ms: whole_number(params, "gapMs", 2_000, 0, 600_000)?,
        min_samples: whole_number(params, "minSamples", 1, 1, 100)? as usize,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_phrases_are_trimmed_and_the_rest_has_defaults() {
        let options =
            read_find_options(&json!({ "phrases": ["  two cats on a couch ", "a car"] })).unwrap();
        assert_eq!(options.phrases, ["two cats on a couch", "a car"]);
        assert_eq!(options.min_confidence, 0.1);
        assert_eq!(options.device, DeviceChoice::Cpu);
        assert_eq!((options.gap_ms, options.min_samples), (2_000, 1));
    }

    #[test]
    fn what_is_wrong_is_a_400_that_says_so() {
        let too_many: Vec<String> = (0..17).map(|n| format!("phrase {n}")).collect();
        for bad in [
            json!({}),
            json!({ "phrases": [] }),
            json!({ "phrases": "a cat" }),
            json!({ "phrases": ["a cat", "  "] }),
            json!({ "phrases": [3] }),
            json!({ "phrases": too_many }),
            json!({ "phrases": ["x".repeat(201)] }),
            json!({ "phrases": ["a cat"], "minConfidence": 0 }),
            json!({ "phrases": ["a cat"], "gapMs": -1 }),
            json!({ "phrases": ["a cat"], "minSamples": 0 }),
        ] {
            let failure = read_find_options(&bad).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)), "{bad}");
        }
    }

    /// The manifest of this plugin, for the tests below.
    fn inputs() -> Value {
        let manifest: Value = serde_json::from_str(include_str!("../../plugin.json")).unwrap();
        manifest["actions"]["find"]["inputs"]["properties"].clone()
    }

    #[test]
    fn every_input_of_the_manifest_has_a_group_for_an_editor() {
        for (name, schema) in inputs().as_object().unwrap() {
            let group = schema["x-ui"]["group"].as_str();
            assert!(
                matches!(group, Some("everyday" | "advanced")),
                "the input {name} says no group"
            );
        }
    }

    #[test]
    fn the_defaults_the_manifest_states_are_the_ones_the_code_uses() {
        let inputs = inputs();
        // `phrases` is required, so the options are made from a call that gives one.
        let options = read_find_options(&json!({ "phrases": ["a cat"] })).unwrap();
        assert_eq!(
            inputs["minConfidence"]["default"].as_f64().unwrap() as f32,
            options.min_confidence
        );
        assert_eq!(inputs["gapMs"]["default"].as_u64().unwrap(), options.gap_ms);
        assert_eq!(
            inputs["minSamples"]["default"].as_u64().unwrap() as usize,
            options.min_samples
        );
    }
}
