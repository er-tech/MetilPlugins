use loomasplugins_frame_reader::{read_video_region, PixelFormat};
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use serde_json::{json, Value};

use crate::models::locate_models;
use crate::motion::{find_hints, join_hints};
use crate::people::Person;
use crate::pose::{find_people, FindSettings};
use crate::tracking::track_people;

use super::read_find_options_mapper::read_find_options;

/// The `find` action: the movements of the people in a video, as a list.
///
/// Params: `media` (a video; Loomas decodes five colour pictures a second of it) and the options of
/// `read_find_options`. It gives `{ movements, people, samples, durationMs, model, device }`. A
/// movement is `{ startMs, endMs, label, confidence, person, detail }`: what was seen, written as
/// what moved and never as what it means. `person` is a number for this run only, never an
/// identity. A video with nobody in it is a success with no movements.
pub fn find_movements(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
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
    let spec = options.model;
    let paths = locate_models(spec, cancel)?;
    let settings = FindSettings {
        device: options.device,
        min_score: options.min_person_score,
        most: options.most_people,
    };
    let size = (video.width, video.height);
    let looked_at: Vec<usize> = (0..video.frames.len()).step_by(options.stride).collect();
    let mut frames: Vec<(u64, Vec<Person>)> = Vec::with_capacity(looked_at.len());
    let mut report = None;
    for (done, &index) in looked_at.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!("{done} of {} pictures", looked_at.len()));
        let pixels = video
            .pixels(index)
            .ok_or_else(|| Failure::status(500, "a picture of the video went missing"))?;
        let (people, used) = find_people(spec, &paths, pixels, size, &settings)?;
        report = used.or(report);
        frames.push((video.frames[index].pts_ms, people));
    }
    let tracks = track_people(&frames);
    let hints = find_hints(&tracks, &options.motion);
    let movements = join_hints(
        &hints,
        options.gap_ms,
        options.min_samples,
        options.min_confidence,
    );
    let followed = tracks
        .iter()
        .filter(|track| track.samples.len() >= 2)
        .count();
    Ok(json!({
        "movements": movements.iter().map(|movement| json!({
            "startMs": movement.start_ms,
            "endMs": movement.end_ms,
            "label": movement.label,
            "confidence": movement.confidence,
            "person": movement.person,
            "detail": movement.detail,
        })).collect::<Vec<_>>(),
        "people": followed,
        "samples": looked_at.len(),
        "durationMs": video.end_ms.saturating_sub(video.start_ms),
        "model": spec.name,
        "device": report.map(|report| report.to_value()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_in_place_of_a_decoded_video_and_a_missing_media_are_400s() {
        let failure = find_movements(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = find_movements(&json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }

    #[test]
    fn a_wrong_param_is_refused_before_anything_is_read_or_downloaded() {
        let failure = find_movements(
            &json!({ "media": "a.mp4", "model": "unicorn" }),
            &Cancel::default(),
        )
        .unwrap_err();
        assert!(failure.message.contains("unicorn"), "{}", failure.message);
    }
}
