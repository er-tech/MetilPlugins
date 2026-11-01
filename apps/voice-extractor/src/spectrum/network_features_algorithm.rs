use crate::tables::{Tables, BANDS, FILTER_BINS};

use super::spectrogram_contract::Spectrogram;

/// How much of the last frame stays in what the features are measured against: 0.99 for a time
/// constant of one second at 100 frames a second.
const ALPHA: f32 = 0.99;

/// What the network is given for a sound.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkFeatures {
    /// The loudness of each band in each frame, `BANDS` for each frame, as the network was trained:
    /// in decibels, less its running mean, over 40.
    pub bands: Vec<f32>,
    /// The first `FILTER_BINS` frequencies of each frame, each divided by the square root of its own
    /// running size: the real parts of all the frames, then the imaginary parts.
    pub spectrum: Vec<f32>,
}

/// The features of a transform, worked out frame after frame because each depends on the ones
/// before it (the running mean starts at -60 dB falling to -90 dB over the bands, and the running
/// size at 0.001 falling to 0.0001 over the frequencies).
pub fn network_features(spectrogram: &Spectrogram, tables: &Tables) -> NetworkFeatures {
    let frames = spectrogram.frames;
    let mut bands = vec![0f32; frames * BANDS];
    let mut mean: Vec<f32> = (0..BANDS)
        .map(|band| -60.0 + (-30.0) * band as f32 / (BANDS - 1) as f32)
        .collect();
    let mut size: Vec<f32> = (0..FILTER_BINS)
        .map(|bin| 0.001 + (0.0001 - 0.001) * bin as f32 / (FILTER_BINS - 1) as f32)
        .collect();
    let mut spectrum = vec![0f32; 2 * frames * FILTER_BINS];

    for frame in 0..frames {
        let bins = spectrogram.frame(frame);
        for (band, members) in tables.bands.iter().enumerate() {
            let energy: f32 = members
                .iter()
                .map(|(bin, weight)| {
                    (bins[*bin].re * bins[*bin].re + bins[*bin].im * bins[*bin].im) * weight
                })
                .sum();
            let decibels = 10.0 * (energy + 1e-10).log10();
            mean[band] = decibels * (1.0 - ALPHA) + mean[band] * ALPHA;
            bands[frame * BANDS + band] = (decibels - mean[band]) / 40.0;
        }
        for bin in 0..FILTER_BINS {
            let value = bins[bin];
            size[bin] = value.norm() * (1.0 - ALPHA) + size[bin] * ALPHA;
            let scaled = value / size[bin].sqrt();
            spectrum[frame * FILTER_BINS + bin] = scaled.re;
            spectrum[(frames + frame) * FILTER_BINS + bin] = scaled.im;
        }
    }
    NetworkFeatures { bands, spectrum }
}

#[cfg(test)]
mod tests {
    use realfft::num_complex::Complex32;

    use super::*;
    use crate::tables::BINS;

    fn one_frame_per_band_tables() -> Tables {
        Tables {
            bands: (0..BANDS).map(|band| vec![(band, 1.0)]).collect(),
            spread: (0..BANDS).map(|band| vec![(band, 1.0)]).collect(),
            window: vec![1.0; 960],
        }
    }

    #[test]
    fn a_steady_sound_has_features_that_settle_at_zero_and_a_size_of_the_square_root() {
        let frames = 600;
        let spectrogram = Spectrogram {
            frames,
            values: vec![Complex32::new(4.0, 0.0); frames * BINS],
        };
        let features = network_features(&spectrogram, &one_frame_per_band_tables());
        // Loudness against its own running mean: nothing changes, so nothing is left.
        let last_bands = &features.bands[(frames - 1) * BANDS..];
        assert!(
            last_bands.iter().all(|value| value.abs() < 0.01),
            "{last_bands:?}"
        );
        // The first frames are far from the starting mean, so they are not zero.
        assert!(features.bands[..BANDS]
            .iter()
            .any(|value| value.abs() > 0.1));
        // 4 divided by the square root of its running size (4) is 2, in the real part.
        let last_real = features.spectrum[(frames - 1) * FILTER_BINS];
        assert!((last_real - 2.0).abs() < 0.05, "{last_real}");
        assert_eq!(features.spectrum.len(), 2 * frames * FILTER_BINS);
        assert!(features.spectrum[frames * FILTER_BINS..]
            .iter()
            .all(|im| *im == 0.0));
    }
}
