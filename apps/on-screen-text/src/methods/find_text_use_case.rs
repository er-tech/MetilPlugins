use loomasplugins_frame_reader::{read_video_region, PixelFormat};
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use serde_json::{json, Value};

use crate::appearances::{merge_appearances, ReadLine, Reading};
use crate::models::locate_models;
use crate::reading::{load_characters, read_picture};

use super::read_find_options_mapper::read_find_options;

/// The `find` action: the text a video shows.
///
/// Params: `media` (a video; Loomas decodes a colour picture of it each second) and the options of
/// `read_find_options`. It gives `{ texts, samples, durationMs, language, device }`. A text is
/// `{ startMs, endMs, text, confidence, samples, box }`: what is written, when it is on the screen,
/// how sure the reading is (0 to 1), in how many pictures it was read and where it is, as shares of
/// the picture (`x`, `y`, `width`, `height`). A video with no text is a success with no texts.
pub fn find_text(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_find_options(params)?;
    // A call that asked for `auto` runs where the host grants.
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
    let paths = locate_models(options.detector, options.language, cancel)?;
    let dictionary = std::fs::read_to_string(&paths.dictionary).map_err(|error| {
        Failure::status(
            422,
            format!(
                "the dictionary cannot be read from {}: {error}",
                paths.dictionary.display()
            ),
        )
    })?;
    let characters = load_characters(&dictionary);
    let size = (video.width, video.height);
    let mut readings = Vec::with_capacity(video.frames.len());
    let mut report = None;
    for (index, frame) in video.frames.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!("{index} of {} pictures", video.frames.len()));
        let pixels = video
            .pixels(index)
            .ok_or_else(|| Failure::status(500, "a picture of the video went missing"))?;
        let (lines, used) = read_picture(&paths, &characters, options.device, pixels, size)?;
        report = used.or(report);
        let (width, height) = (size.0 as f32, size.1 as f32);
        readings.push(Reading {
            time_ms: frame.pts_ms,
            end_ms: frame.pts_ms + frame.duration_ms.max(1),
            lines: lines
                .into_iter()
                .map(|line| ReadLine {
                    text: line.text,
                    confidence: line.confidence,
                    bbox: [
                        line.bbox[0] / width,
                        line.bbox[1] / height,
                        line.bbox[2] / width,
                        line.bbox[3] / height,
                    ],
                })
                .collect(),
        });
    }
    let appearances = merge_appearances(
        &readings,
        options.gap_ms,
        options.min_confidence,
        options.min_chars,
    );
    let round = |value: f32| (value * 1000.0).round() / 1000.0;
    Ok(json!({
        "texts": appearances.iter().map(|appearance| json!({
            "startMs": appearance.start_ms,
            "endMs": appearance.end_ms,
            "text": appearance.text,
            "confidence": round(appearance.confidence),
            "samples": appearance.samples,
            "box": {
                "x": round(appearance.bbox[0]),
                "y": round(appearance.bbox[1]),
                "width": round(appearance.bbox[2] - appearance.bbox[0]),
                "height": round(appearance.bbox[3] - appearance.bbox[1]),
            },
        })).collect::<Vec<_>>(),
        "samples": video.frames.len(),
        "durationMs": video.end_ms.saturating_sub(video.start_ms),
        "language": options.language.name,
        "device": report.map(|report| report.to_value()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_in_place_of_a_decoded_video_and_a_missing_media_are_400s() {
        let failure = find_text(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = find_text(&json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }

    #[test]
    fn a_wrong_param_is_refused_before_anything_is_read_or_downloaded() {
        let failure = find_text(
            &json!({ "media": "a.mp4", "language": "klingon" }),
            &Cancel::default(),
        )
        .unwrap_err();
        assert!(failure.message.contains("klingon"), "{}", failure.message);
    }
}
