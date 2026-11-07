use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{number, text, whole_number, Failure};
use serde_json::Value;

use crate::models::{model_named, MODELS};
use crate::motion::MotionSettings;

use super::find_options_contract::FindOptions;

/// The pictures Loomas decodes for this plugin each second (see `wants` in the manifest).
const DECODED_PER_SECOND: f64 = 5.0;

/// The params of `find` as options: `model` (`rtmpose-small` by default), `device` (`auto`, `cpu`,
/// `gpu` or `gpu:N`, default `auto`), `picturesPerSecond` (1, 2.5 or 5, default 5), `armSpeed`
/// (default 4) and `legSpeed` (default 5) in torso lengths a second, `minConfidence` (default 0.3),
/// `gapMs` (default 1000), `minSamples` (default 1), `minPersonScore` (default 0.5) and
/// `maxPeople` (default 8). Anything outside its range is a 400 that says what was wrong.
pub fn read_find_options(params: &Value) -> Result<FindOptions, Failure> {
    let name = text(params, "model", Some("rtmpose-small"), 20)?.unwrap_or_default();
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
    let rate = number(
        params,
        "picturesPerSecond",
        DECODED_PER_SECOND,
        0.5,
        DECODED_PER_SECOND,
    )?;
    let stride = match rate {
        r if (r - 5.0).abs() < 1e-9 => 1,
        r if (r - 2.5).abs() < 1e-9 => 2,
        r if (r - 1.0).abs() < 1e-9 => 5,
        other => {
            return Err(Failure::status(
                400,
                format!("picturesPerSecond must be 1, 2.5 or 5, not {other}"),
            ))
        }
    };
    Ok(FindOptions {
        model,
        device,
        stride,
        motion: MotionSettings {
            arm_speed: number(params, "armSpeed", 4.0, 0.5, 50.0)? as f32,
            leg_speed: number(params, "legSpeed", 5.0, 0.5, 50.0)? as f32,
        },
        min_person_score: number(params, "minPersonScore", 0.5, 0.05, 1.0)? as f32,
        most_people: whole_number(params, "maxPeople", 8, 1, 30)? as usize,
        min_confidence: number(params, "minConfidence", 0.3, 0.0, 1.0)? as f32,
        gap_ms: whole_number(params, "gapMs", 1000, 0, 60_000)? as u64,
        min_samples: whole_number(params, "minSamples", 1, 1, 100)? as usize,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_means_the_small_model_on_the_grant_with_every_picture() {
        let options = read_find_options(&json!({})).unwrap();
        assert_eq!(options.model.name, "rtmpose-small");
        assert_eq!(options.device, DeviceChoice::Auto);
        assert_eq!(
            (options.stride, options.min_samples, options.gap_ms),
            (1, 1, 1000)
        );
        assert_eq!(options.motion.arm_speed, 4.0);
    }

    #[test]
    fn what_is_given_is_used_and_what_is_wrong_is_a_400_that_says_so() {
        let options = read_find_options(&json!({ "model": "movenet", "device": "cpu", "picturesPerSecond": 2.5, "armSpeed": 6 })).unwrap();
        assert_eq!(
            (options.model.name, options.stride, options.motion.arm_speed),
            ("movenet", 2, 6.0)
        );
        assert_eq!(
            read_find_options(&json!({ "picturesPerSecond": 1 }))
                .unwrap()
                .stride,
            5
        );
        for bad in [
            json!({ "model": "huge" }),
            json!({ "device": "cuda" }),
            json!({ "picturesPerSecond": 3 }),
            json!({ "picturesPerSecond": 10 }),
            json!({ "armSpeed": 0 }),
            json!({ "maxPeople": 0 }),
            json!({ "minConfidence": 2 }),
        ] {
            assert_eq!(
                read_find_options(&bad).unwrap_err().code,
                Some(json!(400)),
                "{bad}"
            );
        }
    }
}
