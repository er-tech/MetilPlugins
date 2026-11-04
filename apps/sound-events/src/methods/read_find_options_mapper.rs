use serde_json::Value;
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{flag, number, text, whole_number, Failure};

use crate::classes::select_labels;

use super::find_options_contract::FindOptions;

/// The params of `find` as options: `labels` (names; all when left out), `minConfidence` (0.05 to
/// 1, default 0.5), `gapMs` (0 to ten minutes, default 1000), `minFrames` (1 to 100, default 1),
/// `clips` (default true), `clipPadMs` (0 to 5000, default 250), `maxClipMs` (1000 to a minute,
/// default 10000) and `device` (`auto`, `cpu`, `gpu` or `gpu:N`, default `cpu`). Anything outside
/// its range is a 400 that says what was wrong.
pub fn read_find_options(params: &Value) -> Result<FindOptions, Failure> {
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
        device,
        wanted: select_labels(params)?,
        min_confidence: number(params, "minConfidence", 0.5, 0.05, 1.0)? as f32,
        gap_ms: whole_number(params, "gapMs", 1_000, 0, 600_000)?,
        min_frames: whole_number(params, "minFrames", 1, 1, 100)? as usize,
        clips: flag(params, "clips", true)?,
        clip_pad_ms: whole_number(params, "clipPadMs", 250, 0, 5_000)?,
        max_clip_ms: whole_number(params, "maxClipMs", 10_000, 1_000, 60_000)?,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_means_the_defaults() {
        let options = read_find_options(&json!({})).unwrap();
        assert!(options.wanted.iter().all(|wanted| *wanted));
        assert_eq!(options.min_confidence, 0.5);
        assert_eq!((options.gap_ms, options.min_frames), (1_000, 1));
        assert!(options.clips);
        assert_eq!((options.clip_pad_ms, options.max_clip_ms), (250, 10_000));
    }

    #[test]
    fn what_is_given_is_used_and_what_is_wrong_is_a_400_that_says_so() {
        let options = read_find_options(
            &json!({ "labels": ["Meow"], "minConfidence": 0.3, "clips": false, "gapMs": 0 }),
        )
        .unwrap();
        assert_eq!(options.wanted.iter().filter(|wanted| **wanted).count(), 1);
        assert!(!options.clips);

        for bad in [
            json!({ "minConfidence": 2 }),
            json!({ "gapMs": -1 }),
            json!({ "minFrames": 0 }),
            json!({ "clips": "yes" }),
            json!({ "maxClipMs": 10 }),
            json!({ "labels": ["Unicorn"] }),
            json!({ "device": "cuda" }),
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
        let options = read_find_options(&json!({})).unwrap();
        assert_eq!(options.device, DeviceChoice::Cpu);
        assert_eq!(inputs["device"]["default"], "cpu");
        assert_eq!(
            inputs["minConfidence"]["default"].as_f64().unwrap() as f32,
            options.min_confidence
        );
        assert_eq!(inputs["gapMs"]["default"].as_u64().unwrap(), options.gap_ms);
        assert_eq!(
            inputs["minFrames"]["default"].as_u64().unwrap() as usize,
            options.min_frames
        );
        assert_eq!(inputs["clips"]["default"].as_bool().unwrap(), options.clips);
        assert_eq!(
            inputs["clipPadMs"]["default"].as_u64().unwrap(),
            options.clip_pad_ms
        );
        assert_eq!(
            inputs["maxClipMs"]["default"].as_u64().unwrap(),
            options.max_clip_ms
        );
    }
}
