use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{text, Failure};
use serde_json::Value;

use crate::models::{model_named, MODELS};

use super::analyze_options_contract::AnalyzeOptions;

/// The params of `analyze` as options: `model` (`standard` by default, or `full`) and `device`
/// (`auto`, `cpu`, `gpu` or `gpu:N`; the default is `cpu`, because the model is small and the
/// texts are short). Anything else is a 400 that says what was wrong.
pub fn read_analyze_options(params: &Value) -> Result<AnalyzeOptions, Failure> {
    let name = text(params, "model", Some("standard"), 20)?.unwrap_or_default();
    let model = model_named(&name).ok_or_else(|| {
        let known: Vec<&str> = MODELS.iter().map(|model| model.name).collect();
        Failure::status(
            400,
            format!("model must be one of {}, not {name:?}", known.join(", ")),
        )
    })?;
    let device_text = text(params, "device", Some("cpu"), 12)?.unwrap_or_default();
    let device = DeviceChoice::parse(&device_text).ok_or_else(|| {
        Failure::status(
            400,
            format!(
                "device must be auto, cpu, gpu or gpu:N (an adapter number), not {device_text:?}"
            ),
        )
    })?;
    Ok(AnalyzeOptions { model, device })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_means_the_small_model_on_the_processor() {
        let options = read_analyze_options(&json!({})).unwrap();
        assert_eq!(options.model.name, "standard");
        assert_eq!(options.device, DeviceChoice::Cpu);
    }

    #[test]
    fn what_is_given_is_used_and_what_is_wrong_is_a_400() {
        let options = read_analyze_options(&json!({ "model": "full", "device": "auto" })).unwrap();
        assert_eq!(
            (options.model.name, options.device),
            ("full", DeviceChoice::Auto)
        );
        for bad in [json!({ "model": "huge" }), json!({ "device": "cuda" })] {
            assert_eq!(
                read_analyze_options(&bad).unwrap_err().code,
                Some(json!(400)),
                "{bad}"
            );
        }
    }
}
