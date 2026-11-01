use serde_json::Value;
use metilplugins_onnx_device::DeviceChoice;
use metilplugins_plugin_sdk::{text, whole_number, Failure};

use crate::models::{model_named, MODELS};

use super::upscale_options_contract::UpscaleOptions;

/// The params of `upscale` as options: `model` (`general-x4v3` by default), `device` (`auto`,
/// `cpu`, `gpu` or `gpu:N`, default `auto`), `tile` (64 to 1024, default 256) and `scale` (2 or 4,
/// default 4). Anything outside its range is a 400 that says what was wrong.
pub fn read_upscale_options(params: &Value) -> Result<UpscaleOptions, Failure> {
    let name = text(params, "model", Some("general-x4v3"), 20)?.unwrap_or_default();
    let model = model_named(&name).ok_or_else(|| {
        let known: Vec<&str> = MODELS.iter().map(|model| model.name).collect();
        Failure::status(
            400,
            format!("model must be one of {}, not {name:?}", known.join(", ")),
        )
    })?;
    let device_text = text(params, "device", Some("auto"), 12)?.unwrap_or_default();
    let device = DeviceChoice::parse(&device_text).ok_or_else(|| {
        Failure::status(
            400,
            format!(
                "device must be auto, cpu, gpu or gpu:N (an adapter number), not {device_text:?}"
            ),
        )
    })?;
    let scale = whole_number(params, "scale", 4, 2, 4)? as usize;
    if scale != 2 && scale != 4 {
        return Err(Failure::status(
            400,
            format!("scale must be 2 or 4, not {scale}"),
        ));
    }
    Ok(UpscaleOptions {
        model,
        device,
        tile: whole_number(params, "tile", 256, 64, 1024)? as usize,
        scale,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_means_the_defaults() {
        let options = read_upscale_options(&json!({})).unwrap();
        assert_eq!(options.model.name, "general-x4v3");
        assert_eq!(options.device, DeviceChoice::Auto);
        assert_eq!((options.tile, options.scale), (256, 4));
    }

    #[test]
    fn what_is_given_is_used_and_what_is_wrong_is_a_400_that_says_so() {
        let options = read_upscale_options(
            &json!({ "model": "x4plus", "device": "cpu", "tile": 128, "scale": 2 }),
        )
        .unwrap();
        assert_eq!(options.model.fixed_tile, Some(128));
        assert_eq!(options.device, DeviceChoice::Cpu);
        assert_eq!((options.tile, options.scale), (128, 2));

        for bad in [
            json!({ "model": "huge" }),
            json!({ "device": "cuda" }),
            json!({ "tile": 32 }),
            json!({ "tile": 2000 }),
            json!({ "scale": 3 }),
            json!({ "scale": 8 }),
        ] {
            let failure = read_upscale_options(&bad).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)), "{bad}");
        }
    }

    /// The manifest of this plugin, for the tests below.
    fn inputs() -> Value {
        let manifest: Value = serde_json::from_str(include_str!("../../plugin.json")).unwrap();
        manifest["actions"]["upscale"]["inputs"]["properties"].clone()
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
    fn the_defaults_and_the_model_names_the_manifest_states_are_the_ones_the_code_uses() {
        let inputs = inputs();
        let options = read_upscale_options(&json!({})).unwrap();
        assert_eq!(inputs["model"]["default"], options.model.name);
        assert_eq!(inputs["device"]["default"], "auto");
        assert_eq!(
            inputs["tile"]["default"].as_u64().unwrap() as usize,
            options.tile
        );
        assert_eq!(
            inputs["scale"]["default"].as_u64().unwrap() as usize,
            options.scale
        );
        let listed: Vec<&str> = inputs["model"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| name.as_str().unwrap())
            .collect();
        let known: Vec<&str> = MODELS.iter().map(|model| model.name).collect();
        assert_eq!(listed, known);
    }
}
