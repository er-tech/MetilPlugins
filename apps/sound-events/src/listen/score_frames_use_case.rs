use std::path::Path;

use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::yamnet_store::{run_yamnet, CLASSES};

/// A frame starts every 7680 samples (480 ms at 16 kHz) ...
pub const HOP: usize = 7680;
/// ... and is 15600 samples (975 ms) long.
pub const WINDOW: usize = 15600;
/// How many frames are scored in one go: about five minutes of sound, so the model's memory does
/// not grow with the length of the video.
const CHUNK_FRAMES: usize = 600;

/// The scores of every frame of `samples` (mono, 16 kHz), one frame after another with `CLASSES`
/// values each. Frame `n` starts at sample `n * HOP`.
pub fn score_frames(
    file: &Path,
    device: DeviceChoice,
    samples: &[f32],
    cancel: &Cancel,
) -> Result<Vec<f32>, Failure> {
    score_in_chunks(file, device, samples, CHUNK_FRAMES, cancel)
}

/// The same, a chunk of `chunk_frames` frames at a time. Chunks start on a multiple of `HOP` and
/// reach far enough to finish their last frame, so a frame has the same score whatever the chunking.
pub(super) fn score_in_chunks(
    file: &Path,
    device: DeviceChoice,
    samples: &[f32],
    chunk_frames: usize,
    cancel: &Cancel,
) -> Result<Vec<f32>, Failure> {
    let step = chunk_frames * HOP;
    let mut scores = Vec::new();
    let mut start = 0;
    while start < samples.len() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!(
            "{} of {} seconds",
            start / 16_000,
            samples.len() / 16_000
        ));
        let last = start + step >= samples.len();
        let end = (start + step + WINDOW - HOP).min(samples.len());
        let frames = run_yamnet(file, device, &samples[start..end])?;
        let keep = if last {
            frames.len()
        } else {
            frames.len().min(chunk_frames * CLASSES)
        };
        scores.extend_from_slice(&frames[..keep]);
        start += step;
    }
    Ok(scores)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The model, when it is already on this computer (it is downloaded by the first real run).
    fn cached_model() -> Option<PathBuf> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()?;
        let file = PathBuf::from(home).join(".loomas/models/sound-events/yamnet.onnx");
        file.is_file().then_some(file)
    }

    #[test]
    fn a_frame_has_the_same_scores_whether_the_sound_is_scored_whole_or_in_chunks() {
        let Some(file) = cached_model() else {
            eprintln!("skipped: the model is not downloaded yet");
            return;
        };
        // Ten seconds of a rising tone with a click now and then, made without randomness.
        let samples: Vec<f32> = (0..160_000)
            .map(|n| {
                let t = n as f32 / 16_000.0;
                0.4 * (2.0 * std::f32::consts::PI * (200.0 + 100.0 * t) * t).sin()
                    + if n % 20_000 < 200 { 0.5 } else { 0.0 }
            })
            .collect();
        let cancel = Cancel::default();

        let device = DeviceChoice::Cpu;
        let whole = score_in_chunks(&file, device, &samples, 10_000, &cancel).unwrap();
        let chunked = score_in_chunks(&file, device, &samples, 5, &cancel).unwrap();

        // The chunked run may score one more frame at the end: the last stretch of sound, which no
        // whole frame reaches, with silence to fill it. Every frame both score must agree.
        assert!(whole.len().abs_diff(chunked.len()) <= CLASSES);
        let worst = whole
            .iter()
            .zip(&chunked)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        assert!(worst < 1e-3, "the scores differ by {worst}");
    }

    #[test]
    fn no_sound_gives_no_frames() {
        let scores = score_in_chunks(
            Path::new("no-model"),
            DeviceChoice::Cpu,
            &[],
            5,
            &Cancel::default(),
        )
        .unwrap();
        assert!(scores.is_empty());
    }
}
