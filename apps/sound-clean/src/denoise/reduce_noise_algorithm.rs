use std::f32::consts::PI;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

/// How hard to clean.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseOptions {
    /// 0 to 1: how much of the noise is taken out (0 takes out what it measured, 1 takes out
    /// five times that).
    pub strength: f32,
    /// The least a frequency is let through as, as a factor (0.1 is -20 dB), so the sound does not
    /// turn into bubbles where everything was taken out.
    pub floor: f32,
}

const FRAME: usize = 512;
const HOP: usize = FRAME / 2;

/// Takes steady noise out of a sound by spectral subtraction: the sound is cut into overlapping
/// frames, the noise is measured as the average spectrum of the quietest frames (the places where
/// only the noise is there), and every frame is turned down, frequency by frequency, by what the
/// noise would add. Frames are windowed with the square root of a Hann window both ways, so the
/// overlap puts the sound back together exactly where nothing was turned down.
///
/// The result has as many samples as the sound. Noise that comes and goes (a door, a dog) is not
/// steady and stays. A sound shorter than a frame is given back as it is.
pub fn reduce_noise(samples: &[f32], options: NoiseOptions) -> Vec<f32> {
    if samples.len() < FRAME {
        return samples.to_vec();
    }
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(FRAME);
    let inverse = planner.plan_fft_inverse(FRAME);
    let window = square_root_hann();

    // Half a frame of silence at the start and a frame at the end, so every sample is covered by
    // two frames.
    let mut padded = vec![0.0; HOP];
    padded.extend_from_slice(samples);
    padded.extend(std::iter::repeat_n(0.0, FRAME));
    let frames = (padded.len() - FRAME) / HOP + 1;

    let spectra: Vec<Vec<Complex<f32>>> = (0..frames)
        .map(|n| transform(&padded[n * HOP..n * HOP + FRAME], &window, forward.as_ref()))
        .collect();
    let noise = noise_profile(&spectra);
    let over_subtraction = 1.0 + 4.0 * options.strength.clamp(0.0, 1.0);

    let mut out = vec![0.0f32; padded.len()];
    let scale = 1.0 / FRAME as f32;
    for (n, spectrum) in spectra.iter().enumerate() {
        let gains = gains(spectrum, &noise, over_subtraction, options.floor);
        let mut bins: Vec<Complex<f32>> = spectrum
            .iter()
            .zip(&gains)
            .map(|(bin, gain)| bin * *gain)
            .collect();
        inverse.process(&mut bins);
        for (i, bin) in bins.iter().enumerate() {
            out[n * HOP + i] += bin.re * scale * window[i];
        }
    }
    out[HOP..HOP + samples.len()].to_vec()
}

fn square_root_hann() -> Vec<f32> {
    (0..FRAME)
        .map(|n| (0.5 - 0.5 * (2.0 * PI * n as f32 / FRAME as f32).cos()).sqrt())
        .collect()
}

fn transform(frame: &[f32], window: &[f32], fft: &dyn Fft<f32>) -> Vec<Complex<f32>> {
    let mut bins: Vec<Complex<f32>> = frame
        .iter()
        .zip(window)
        .map(|(sample, weight)| Complex::new(sample * weight, 0.0))
        .collect();
    fft.process(&mut bins);
    bins
}

/// The average size of every frequency in the quietest frames (a tenth, at least three), which is
/// where only the noise is heard.
fn noise_profile(spectra: &[Vec<Complex<f32>>]) -> Vec<f32> {
    let mut by_energy: Vec<(f32, usize)> = spectra
        .iter()
        .enumerate()
        .map(|(n, spectrum)| (spectrum.iter().map(|bin| bin.norm_sqr()).sum(), n))
        .collect();
    by_energy.sort_by(|a, b| a.0.total_cmp(&b.0));
    let quiet = (spectra.len() / 10).max(3).min(spectra.len());
    let mut profile = vec![0.0f32; FRAME];
    for (_, n) in &by_energy[..quiet] {
        for (sum, bin) in profile.iter_mut().zip(&spectra[*n]) {
            *sum += bin.norm();
        }
    }
    profile.iter().map(|sum| sum / quiet as f32).collect()
}

/// How much of each frequency to let through: what is left of its size after the noise is taken
/// away, never under the floor, and smoothed over three neighbours so single frequencies do not
/// flicker on and off.
fn gains(spectrum: &[Complex<f32>], noise: &[f32], over_subtraction: f32, floor: f32) -> Vec<f32> {
    let raw: Vec<f32> = spectrum
        .iter()
        .zip(noise)
        .map(|(bin, noise)| {
            let size = bin.norm();
            if size <= f32::EPSILON {
                floor
            } else {
                ((size - over_subtraction * noise) / size).clamp(floor, 1.0)
            }
        })
        .collect();
    (0..raw.len())
        .map(|k| {
            let before = raw[(k + raw.len() - 1) % raw.len()];
            let after = raw[(k + 1) % raw.len()];
            (before + 2.0 * raw[k] + after) / 4.0
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::denoise::level_db;

    /// White noise from a small generator, so the test is the same every time.
    fn noise(length: usize, amplitude: f32, seed: u32) -> Vec<f32> {
        let mut state = seed;
        (0..length)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((state >> 8) as f32 / (1 << 24) as f32 * 2.0 - 1.0) * amplitude
            })
            .collect()
    }

    fn tone(length: usize, hertz: f32, amplitude: f32) -> Vec<f32> {
        (0..length)
            .map(|n| amplitude * (2.0 * PI * hertz * n as f32 / 16_000.0).sin())
            .collect()
    }

    const OPTIONS: NoiseOptions = NoiseOptions {
        strength: 0.5,
        floor: 0.1,
    };

    #[test]
    fn noise_alone_is_turned_down_a_lot() {
        let hiss = noise(32_000, 0.05, 1);
        let cleaned = reduce_noise(&hiss, OPTIONS);
        assert_eq!(cleaned.len(), hiss.len());
        let (before, after) = (level_db(&hiss), level_db(&cleaned));
        assert!(after < before - 10.0, "{before} dB became {after} dB");
    }

    #[test]
    fn a_tone_over_noise_keeps_its_level_while_the_noise_around_it_goes() {
        // A second of noise alone (so the noise is measured), then the same noise under a tone.
        let hiss = noise(32_000, 0.05, 7);
        let mut mixed = hiss[..16_000].to_vec();
        mixed.extend(
            hiss[16_000..]
                .iter()
                .zip(tone(16_000, 1_000.0, 0.5))
                .map(|(n, t)| n + t),
        );
        let cleaned = reduce_noise(&mixed, OPTIONS);
        // The tone is as loud as before, within 1.5 dB...
        let (before, after) = (
            level_db(&mixed[20_000..28_000]),
            level_db(&cleaned[20_000..28_000]),
        );
        assert!(
            (before - after).abs() < 1.5,
            "{before} dB became {after} dB"
        );
        // ...and the noise alone is far quieter.
        let (hiss_before, hiss_after) = (
            level_db(&mixed[2_000..14_000]),
            level_db(&cleaned[2_000..14_000]),
        );
        assert!(
            hiss_after < hiss_before - 10.0,
            "{hiss_before} -> {hiss_after}"
        );
    }

    #[test]
    fn silence_stays_silent_and_a_short_sound_is_given_back() {
        assert!(reduce_noise(&[0.0; 4_000], OPTIONS)
            .iter()
            .all(|s| s.abs() < 1e-6));
        let short = vec![0.3; 100];
        assert_eq!(reduce_noise(&short, OPTIONS), short);
        assert!(reduce_noise(&[], OPTIONS).is_empty());
    }

    #[test]
    fn with_nothing_turned_down_the_sound_comes_back_as_it_was() {
        // Strength 0 and a floor of 1 let everything through: the overlap must rebuild the sound.
        let original = tone(8_000, 440.0, 0.4);
        let same = reduce_noise(
            &original,
            NoiseOptions {
                strength: 0.0,
                floor: 1.0,
            },
        );
        let worst = original
            .iter()
            .zip(&same)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 1e-3, "worst difference {worst}");
    }
}
