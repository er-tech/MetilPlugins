use serde_json::{json, Value};
use metilplugins_onnx_device::DeviceChoice;
use metilplugins_plugin_sdk::{Cancel, Failure};
use metilplugins_sound_reader::read_audio_region;
use metilplugins_work_files::write_wav_file;

use crate::classes::AUDIOSET_CLASSES;
use crate::events::{event_times, merge_frames};
use crate::listen::{device_report, score_frames, CLASSES, HOP, WINDOW};
use crate::models::locate_model;

use super::read_find_options_mapper::read_find_options;

/// The rate the tagger listens at, and the one the manifest asks Metil to decode to.
const RATE: u32 = 16_000;
/// How far apart two frames start, in milliseconds.
const HOP_MS: u64 = (HOP as u64) * 1000 / (RATE as u64);
/// How long a frame is, in milliseconds.
const WINDOW_MS: u64 = (WINDOW as u64) * 1000 / (RATE as u64);

/// The `find` action: the sounds a video has, from the 521 classes of AudioSet.
///
/// Params: `media` (the sound of a video, decoded by the host to 16 kHz, one channel), `labels`,
/// `minConfidence`, `gapMs`, `minFrames`, `clips`, `clipPadMs` and `maxClipMs` (see
/// `read_find_options`). It gives `{ events, frames, durationMs }`. An event is
/// `{ label, startMs, endMs, confidence, frames, clip }`: when a sound starts and stops, in the time
/// of the source, how sure the tagger is, and (unless `clips` is false) a stored WAV of it. A sound
/// that goes on is one event; one that stops for longer than `gapMs` and comes back is two.
pub fn find_sounds(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_find_options(params)?;
    // A call that asked for `auto` runs where the host grants; the default stays the processor.
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video"))?;
    let sound = read_audio_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| {
            Failure::status(
                400,
                "media must be sound Metil decoded for this plugin (it asks for 16 kHz sound)",
            )
        })?;
    if sound.sample_rate != RATE {
        return Err(Failure::status(
            400,
            format!(
                "this plugin listens at {RATE} Hz, not {} Hz",
                sound.sample_rate
            ),
        ));
    }
    let file = locate_model(cancel)?;
    let scores = score_frames(&file, options.device, &sound.samples, cancel)?;
    let events = merge_frames(
        &scores,
        CLASSES,
        &options.wanted,
        options.min_confidence,
        (options.gap_ms / HOP_MS) as usize,
        options.min_frames,
    );

    let duration_ms = sound.samples.len() as u64 * 1000 / u64::from(RATE);
    let mut described = Vec::with_capacity(events.len());
    for event in &events {
        let (start, end) = event_times(
            event.first_frame,
            event.last_frame,
            HOP_MS,
            WINDOW_MS,
            duration_ms,
        );
        let mut found = json!({
            "label": AUDIOSET_CLASSES[event.label],
            "startMs": sound.start_ms + start,
            "endMs": sound.start_ms + end,
            "confidence": (f64::from(event.confidence) * 1000.0).round() / 1000.0,
            "frames": event.frames,
        });
        if options.clips {
            let work_dir = cancel.work_dir().ok_or_else(|| {
                Failure::status(
                    422,
                    "Metil gave this plugin no folder to write clips in",
                )
            })?;
            let from = start.saturating_sub(options.clip_pad_ms);
            let to = (end + options.clip_pad_ms)
                .min(duration_ms)
                .min(from + options.max_clip_ms);
            let samples =
                &sound.samples[ms_to_sample(from)..ms_to_sample(to).min(sound.samples.len())];
            found["clip"] = write_wav_file("sound-events", samples, RATE, work_dir)
                .map_err(|error| Failure::status(500, error.to_string()))?;
        }
        described.push(found);
    }
    Ok(json!({
        "events": described,
        "frames": scores.len() / CLASSES,
        "device": device_report().map(|report| report.to_value()),
        "durationMs": duration_ms,
    }))
}

fn ms_to_sample(ms: u64) -> usize {
    (ms * u64::from(RATE) / 1000) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_in_place_of_decoded_sound_and_a_missing_media_are_400s() {
        let failure = find_sounds(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = find_sounds(&json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }

    #[test]
    fn a_wrong_param_is_refused_before_anything_is_read_or_downloaded() {
        let failure = find_sounds(
            &json!({ "media": "a.mp4", "labels": ["Unicorn"] }),
            &Cancel::default(),
        )
        .unwrap_err();
        assert!(failure.message.contains("Unicorn"), "{}", failure.message);
    }

    #[test]
    fn the_frame_times_are_what_the_model_uses() {
        assert_eq!((HOP_MS, WINDOW_MS), (480, 975));
        assert_eq!(ms_to_sample(1000), 16_000);
    }
}
