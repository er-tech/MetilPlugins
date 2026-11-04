use std::path::Path;

use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::Failure;

use crate::network::run_network;
use crate::spectrum::{analyse_sound, apply_deep_filter, network_features, synthesise_sound};
use crate::tables::Tables;

/// The voice of a piece of sound (48 kHz, one channel) as the network makes it: the sound is
/// turned into frames of frequencies, the network is shown their features and answers with a mask
/// and a filter, which are applied, and the frames are turned back into a sound of the same length.
pub fn enhance_piece(
    signal: &[f32],
    tables: &Tables,
    network: &Path,
    device: DeviceChoice,
) -> Result<Vec<f32>, Failure> {
    if signal.is_empty() {
        return Ok(Vec::new());
    }
    let spectrogram = analyse_sound(signal, tables);
    let features = network_features(&spectrogram, tables);
    let answer = run_network(network, device, &features, spectrogram.frames)?;
    let enhanced = apply_deep_filter(&spectrogram, &answer, tables);
    Ok(synthesise_sound(&enhanced, signal.len(), tables))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::tables::read_tables;

    /// The model files, when they are already on this computer (the first real run downloads them).
    fn cached_model() -> Option<(PathBuf, Tables)> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()?;
        let folder = PathBuf::from(home).join(".loomas/models/voice-extractor");
        let tables =
            read_tables(&std::fs::read(folder.join("deepfilter-auxiliary.bin")).ok()?).ok()?;
        let network = folder.join("deepfilter.onnx");
        network.is_file().then_some((network, tables))
    }

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
    }

    #[test]
    fn steady_noise_is_turned_down_a_lot_and_the_length_is_kept_with_the_real_model() {
        let Some((network, tables)) = cached_model() else {
            eprintln!("skipped: the model is not downloaded yet");
            return;
        };
        // Two seconds of noise made without randomness: a sum of many unrelated tones.
        let noise: Vec<f32> = (0..96_000)
            .map(|n| {
                (1..40)
                    .map(|k| ((n as f32) * 0.013 * k as f32 * (1.0 + 0.37 * k as f32)).sin() / 40.0)
                    .sum::<f32>()
            })
            .collect();
        let voice = enhance_piece(&noise, &tables, &network, DeviceChoice::Cpu).unwrap();
        assert_eq!(voice.len(), noise.len());
        assert!(
            rms(&voice) < rms(&noise) * 0.3,
            "noise {} became {}",
            rms(&noise),
            rms(&voice)
        );
    }

    /// Compares the plugin's voice with a reference made by another implementation of the same
    /// processing. Set `VOICE_PARITY_INPUT` (raw little-endian 32-bit floats, 48 kHz, one channel)
    /// and `VOICE_PARITY_REFERENCE` (the voice made of it, the same format) to run it.
    #[test]
    fn the_voice_is_the_one_a_reference_implementation_makes() {
        let (Ok(input), Ok(reference)) = (
            std::env::var("VOICE_PARITY_INPUT"),
            std::env::var("VOICE_PARITY_REFERENCE"),
        ) else {
            return;
        };
        let Some((network, tables)) = cached_model() else {
            panic!("the model is not downloaded yet");
        };
        let floats = |path: &str| -> Vec<f32> {
            std::fs::read(path)
                .unwrap()
                .as_chunks::<4>()
                .0
                .iter()
                .map(|four| f32::from_le_bytes(*four))
                .collect()
        };
        let (signal, wanted) = (floats(&input), floats(&reference));
        let voice = enhance_piece(&signal, &tables, &network, DeviceChoice::Cpu).unwrap();
        assert_eq!(voice.len(), wanted.len());
        let error: f32 = voice
            .iter()
            .zip(&wanted)
            .map(|(a, b)| (a - b) * (a - b))
            .sum();
        let energy: f32 = wanted.iter().map(|b| b * b).sum();
        let db = 10.0 * (energy / error.max(1e-20)).log10();
        eprintln!(
            "rms ours {} reference {}; agreement {db:.1} dB",
            rms(&voice),
            rms(&wanted)
        );
        assert!(
            db > 40.0,
            "the voice agrees with the reference to only {db:.1} dB"
        );
    }
}
