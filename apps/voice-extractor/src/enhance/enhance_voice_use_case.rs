use std::path::Path;

use metilplugins_onnx_device::DeviceChoice;
use metilplugins_plugin_sdk::{Cancel, Failure};

use crate::tables::Tables;

use super::enhance_piece_use_case::enhance_piece;

/// The sound is done in pieces of this many samples (30 s at 48 kHz) so memory does not grow with
/// the length of the video.
const CORE: usize = 30 * 48_000;
/// Each piece is given this much sound before and after it (2 s), to settle in: the network
/// remembers what it heard and so do the features, and they need a moment to catch on. What is
/// made of the margin is thrown away, except where two pieces meet.
const MARGIN: usize = 2 * 48_000;
/// Where two pieces meet, the end of one fades into the start of the next over this many samples
/// (100 ms), half on each side of the joint: two pieces never give exactly the same sound, and a
/// hard switch would click.
const FADE: usize = 4_800;
/// A last piece shorter than this (1 s) is added to the one before it.
const SHORTEST_LAST: usize = 48_000;

/// The voice of a sound (48 kHz, one channel), a piece at a time. A sound of one piece is done in
/// one go and is as `enhance_piece` makes it; a longer one is cut in pieces of 30 s with 2 s of
/// margin on each side, only the middle of each is kept, and the pieces are faded into each other
/// where they meet.
pub fn enhance_voice(
    signal: &[f32],
    tables: &Tables,
    network: &Path,
    device: DeviceChoice,
    cancel: &Cancel,
) -> Result<Vec<f32>, Failure> {
    if signal.is_empty() {
        return Ok(Vec::new());
    }
    let half = FADE / 2;
    let mut voice = vec![0.0f32; signal.len()];
    // The end of the last piece, `FADE` samples round the joint it shares with the next.
    let mut tail: Option<Vec<f32>> = None;
    let mut start = 0;
    while start < signal.len() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!(
            "{} of {} seconds",
            start / 48_000,
            signal.len() / 48_000
        ));
        let mut end = (start + CORE).min(signal.len());
        if signal.len() - end < SHORTEST_LAST {
            end = signal.len();
        }
        let from = start.saturating_sub(MARGIN);
        let to = (end + MARGIN).min(signal.len());
        let piece = enhance_piece(&signal[from..to], tables, network, device)?;

        let lead = if start == 0 { 0 } else { half };
        let trail = if end == signal.len() { 0 } else { half };
        voice[start + lead..end - trail]
            .copy_from_slice(&piece[start + lead - from..end - trail - from]);
        if let Some(previous) = tail.take() {
            for (k, before) in previous.iter().enumerate() {
                let weight = (k as f32 + 0.5) / FADE as f32;
                let after = piece[start - half + k - from];
                voice[start - half + k] = before * (1.0 - weight) + after * weight;
            }
        }
        if trail > 0 {
            tail = Some(piece[end - half - from..end + half - from].to_vec());
        }
        start = end;
    }
    Ok(voice)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::enhance::enhance_piece_use_case::enhance_piece;
    use crate::tables::read_tables;

    #[test]
    fn a_long_sound_done_in_pieces_has_no_more_of_a_jump_where_they_meet_than_one_run_with_the_real_model(
    ) {
        let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) else {
            return;
        };
        let folder = PathBuf::from(home).join(".metil/models/voice-extractor");
        let (Ok(bytes), network) = (
            std::fs::read(folder.join("deepfilter-auxiliary.bin")),
            folder.join("deepfilter.onnx"),
        ) else {
            eprintln!("skipped: the model is not downloaded yet");
            return;
        };
        if !network.is_file() {
            eprintln!("skipped: the model is not downloaded yet");
            return;
        }
        let tables = read_tables(&bytes).unwrap();
        // 70 s (three pieces) of a changing tone with a pulse now and then, without randomness.
        let signal: Vec<f32> = (0..70 * 48_000)
            .map(|n| {
                let t = n as f32 / 48_000.0;
                0.3 * (2.0 * std::f32::consts::PI * (180.0 + 40.0 * (t * 0.7).sin()) * t).sin()
                    + 0.1 * (t * 3_100.0).sin() * if (t * 2.0).fract() < 0.1 { 1.0 } else { 0.0 }
            })
            .collect();

        let device = DeviceChoice::Cpu;
        let voice = enhance_voice(&signal, &tables, &network, device, &Cancel::default()).unwrap();

        assert_eq!(voice.len(), signal.len());
        // The biggest step from one sample to the next, in a stretch of 20 ms round a joint: what
        // the pieces make there is no more of a jump than one run over the whole sound makes (the
        // sound itself has steps: a pulse starts exactly at 60 s).
        let whole = enhance_piece(&signal, &tables, &network, device).unwrap();
        let step = |sound: &[f32], from: usize| {
            sound[from..from + 960]
                .windows(2)
                .map(|pair| (pair[1] - pair[0]).abs())
                .fold(0.0f32, f32::max)
        };
        for joint in [CORE, 2 * CORE] {
            let (in_pieces, in_one_run) = (step(&voice, joint - 480), step(&whole, joint - 480));
            assert!(
                in_pieces < in_one_run * 3.0 + 0.005,
                "a step of {in_pieces} at {joint} where one run has {in_one_run}"
            );
        }
    }
}
