use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{number, text, whole_number, Failure};
use serde_json::Value;

use crate::models::{detector_named, language_named, DETECTORS, LANGUAGES};

use super::find_options_contract::FindOptions;

/// The params of `find` as options: `language` (`latin` by default), `detector` (`fast` by default,
/// or `accurate`), `device` (`auto`, `cpu`, `gpu` or `gpu:N`, default `auto`), `minConfidence`
/// (default 0.6), `minChars` (default 2) and `gapMs` (default 2500). Anything outside its range is a
/// 400 that says what was wrong.
pub fn read_find_options(params: &Value) -> Result<FindOptions, Failure> {
    let language_name = text(params, "language", Some("latin"), 20)?.unwrap_or_default();
    let language = language_named(&language_name).ok_or_else(|| {
        let known: Vec<&str> = LANGUAGES.iter().map(|language| language.name).collect();
        Failure::status(
            400,
            format!(
                "language must be one of {}, not {language_name:?}",
                known.join(", ")
            ),
        )
    })?;
    let detector_name = text(params, "detector", Some("fast"), 20)?.unwrap_or_default();
    let detector = detector_named(&detector_name).ok_or_else(|| {
        let known: Vec<&str> = DETECTORS.iter().map(|detector| detector.name).collect();
        Failure::status(
            400,
            format!(
                "detector must be one of {}, not {detector_name:?}",
                known.join(", ")
            ),
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
    Ok(FindOptions {
        language,
        detector,
        device,
        min_confidence: number(params, "minConfidence", 0.6, 0.0, 1.0)? as f32,
        min_chars: whole_number(params, "minChars", 2, 1, 100)? as usize,
        gap_ms: whole_number(params, "gapMs", 2500, 0, 600_000)? as u64,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_means_latin_letters_and_the_fast_detector() {
        let options = read_find_options(&json!({})).unwrap();
        assert_eq!(
            (options.language.name, options.detector.name),
            ("latin", "fast")
        );
        assert_eq!(
            (options.min_confidence, options.min_chars, options.gap_ms),
            (0.6, 2, 2500)
        );
        assert_eq!(options.device, DeviceChoice::Auto);
    }

    #[test]
    fn what_is_given_is_used_and_what_is_wrong_is_a_400_that_says_so() {
        let options = read_find_options(
            &json!({ "language": "korean", "detector": "accurate", "device": "cpu", "gapMs": 0 }),
        )
        .unwrap();
        assert_eq!(
            (options.language.name, options.detector.name, options.gap_ms),
            ("korean", "accurate", 0)
        );
        for bad in [
            json!({ "language": "klingon" }),
            json!({ "detector": "huge" }),
            json!({ "device": "cuda" }),
            json!({ "minConfidence": 2 }),
            json!({ "minChars": 0 }),
        ] {
            assert_eq!(
                read_find_options(&bad).unwrap_err().code,
                Some(json!(400)),
                "{bad}"
            );
        }
    }
}
