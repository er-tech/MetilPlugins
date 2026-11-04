use serde_json::Value;
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{flag, number, text, whole_number, Failure};

use super::extract_options_contract::ExtractOptions;

/// The params of `extract` as options: `keep` (0 to 1, default 0), `sampleRate` (16000 by default,
/// or 48000), `rest` (default true), `files` (default false) and `device` (`auto`, `cpu`, `gpu` or
/// `gpu:N`, default `cpu`). Anything else is a 400 that says what was wrong.
pub fn read_extract_options(params: &Value) -> Result<ExtractOptions, Failure> {
    let device_text = text(params, "device", Some("cpu"), 12)?.unwrap_or_default();
    let device = DeviceChoice::parse(&device_text).ok_or_else(|| {
        Failure::status(
            400,
            format!(
                "device must be auto, cpu, gpu or gpu:N (an adapter number), not {device_text:?}"
            ),
        )
    })?;
    let sample_rate = whole_number(params, "sampleRate", 16_000, 16_000, 48_000)? as u32;
    if sample_rate != 16_000 && sample_rate != 48_000 {
        return Err(Failure::status(
            400,
            format!("sampleRate must be 16000 or 48000, not {sample_rate}"),
        ));
    }
    Ok(ExtractOptions {
        device,
        keep: number(params, "keep", 0.0, 0.0, 1.0)? as f32,
        sample_rate,
        rest: flag(params, "rest", true)?,
        files: flag(params, "files", false)?,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_means_the_defaults() {
        let options = read_extract_options(&json!({})).unwrap();
        assert_eq!(options.keep, 0.0);
        assert_eq!(options.sample_rate, 16_000);
        assert!(options.rest && !options.files);
    }

    #[test]
    fn what_is_given_is_used_and_what_is_wrong_is_a_400_that_says_so() {
        let options = read_extract_options(
            &json!({ "keep": 0.15, "sampleRate": 48000, "rest": false, "files": true }),
        )
        .unwrap();
        assert_eq!((options.keep, options.sample_rate), (0.15, 48_000));
        assert!(!options.rest && options.files);
        for bad in [
            json!({ "keep": 2 }),
            json!({ "sampleRate": 44100 }),
            json!({ "sampleRate": 8000 }),
            json!({ "rest": "no" }),
            json!({ "files": 1 }),
            json!({ "device": "cuda" }),
        ] {
            let failure = read_extract_options(&bad).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)), "{bad}");
        }
    }

    /// The manifest of this plugin, for the tests below.
    fn inputs() -> Value {
        let manifest: Value = serde_json::from_str(include_str!("../../plugin.json")).unwrap();
        manifest["actions"]["extract"]["inputs"]["properties"].clone()
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
        let options = read_extract_options(&json!({})).unwrap();
        assert_eq!(options.device, DeviceChoice::Cpu);
        assert_eq!(inputs["device"]["default"], "cpu");
        assert_eq!(
            inputs["keep"]["default"].as_f64().unwrap() as f32,
            options.keep
        );
        assert_eq!(
            inputs["sampleRate"]["default"].as_u64().unwrap() as u32,
            options.sample_rate
        );
        assert_eq!(inputs["rest"]["default"].as_bool().unwrap(), options.rest);
        assert_eq!(inputs["files"]["default"].as_bool().unwrap(), options.files);
    }
}
