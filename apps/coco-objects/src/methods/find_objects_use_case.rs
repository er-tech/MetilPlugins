use std::collections::HashMap;

use serde_json::{json, Value};
use loomasplugins_appearances::{track_appearances, Presence, Sample};
use loomasplugins_frame_reader::{read_video_region, PixelFormat};
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use loomasplugins_work_files::write_png_file;

use crate::classes::COCO_CLASSES;
use crate::detect::{detect_objects, device_report, Detection, DetectionSettings};
use crate::models::locate_model;
use crate::picture::draw_boxes;

use super::read_find_options_mapper::read_find_options;

/// The `find` action: the things a video shows, from the 80 classes of the COCO set.
///
/// Params: `media` (a video; the host decodes colour pictures of it), `classes`, `model`,
/// `minConfidence`, `gapMs` and `minSamples` (see `read_find_options`). It gives
/// `{ appearances, samples, model, durationMs }`. An appearance is
/// `{ class, startMs, endMs, confidence, peakCount, samples, picture }`: the first picture where
/// the class shows up (with its boxes drawn, a stored PNG) and the time of it in the source. A
/// class that stays in the video is one appearance; one that leaves and comes back is two.
pub fn find_objects(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_find_options(params)?;
    // A call that left the device to `auto` runs where the host grants.
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video"))?;
    let video = read_video_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| {
            Failure::status(
                400,
                "media must be a video Loomas decoded for this plugin (it asks for pictures)",
            )
        })?;
    if video.format != PixelFormat::Rgb24 {
        return Err(Failure::status(
            400,
            "this plugin reads colour pictures (rgb24)",
        ));
    }
    let file = locate_model(options.model, cancel)?;

    let settings = DetectionSettings {
        device: options.device,
        min_score: options.min_confidence,
        wanted: options.wanted,
    };
    let mut samples = Vec::with_capacity(video.frames.len());
    // What was found in each picture, to draw the boxes of an appearance on its first one.
    let mut found_in: HashMap<usize, Vec<Detection>> = HashMap::new();
    for (index, frame) in video.frames.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        if index % 20 == 0 {
            cancel.progress(&format!("{index} of {} pictures", video.frames.len()));
        }
        let Some(pixels) = video.pixels(index) else {
            continue;
        };
        let found = detect_objects(
            options.model,
            &file,
            pixels,
            video.width,
            video.height,
            &settings,
        )?;
        samples.push(Sample {
            frame: index,
            time_ms: frame.pts_ms,
            duration_ms: frame.duration_ms,
            present: found
                .iter()
                .map(|found| Presence {
                    key: found.class,
                    score: found.score,
                })
                .collect(),
        });
        found_in.insert(index, found);
    }

    let appearances = track_appearances(&samples, options.gap_ms, options.min_samples);
    let mut described = Vec::with_capacity(appearances.len());
    for appearance in &appearances {
        let work_dir = cancel.work_dir().ok_or_else(|| {
            Failure::status(
                422,
                "Loomas gave this plugin no folder to write pictures in",
            )
        })?;
        let mut picture = video
            .pixels(appearance.first_frame)
            .ok_or_else(|| Failure::status(500, "a picture of the video went missing"))?
            .to_vec();
        let boxes: Vec<Detection> = found_in
            .get(&appearance.first_frame)
            .map(|found| {
                found
                    .iter()
                    .filter(|found| found.class == appearance.key)
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        draw_boxes(&mut picture, video.width, video.height, &boxes, 2);
        described.push(json!({
            "class": COCO_CLASSES[appearance.key],
            "label": COCO_CLASSES[appearance.key],
            "startMs": appearance.start_ms,
            "endMs": appearance.end_ms,
            "confidence": (f64::from(appearance.confidence) * 1000.0).round() / 1000.0,
            "peakCount": appearance.peak_count,
            "samples": appearance.samples,
            "picture": write_png_file("coco-objects", &picture, video.width, video.height, work_dir)
                .map_err(|error| Failure::status(500, error.to_string()))?,
        }));
    }
    Ok(json!({
        "appearances": described,
        "samples": samples.len(),
        "model": options.model.name,
        "device": device_report(options.model, options.device).map(|report| report.to_value()),
        "durationMs": video.end_ms.saturating_sub(video.start_ms),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_in_place_of_a_decoded_video_and_a_missing_media_are_400s() {
        let failure = find_objects(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = find_objects(&json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }

    #[test]
    fn a_wrong_param_is_refused_before_anything_is_read_or_downloaded() {
        let failure = find_objects(
            &json!({ "media": "a.mp4", "classes": ["unicorn"] }),
            &Cancel::default(),
        )
        .unwrap_err();
        assert!(failure.message.contains("unicorn"), "{}", failure.message);
    }
}
