use realfft::num_complex::Complex32;
use realfft::RealFftPlanner;

use crate::tables::{Tables, BINS, FFT_SIZE, HOP};

use super::spectrogram_contract::Spectrogram;

/// The transform of a sound: frames of `FFT_SIZE` samples, a new one every `HOP`, each multiplied
/// by the window and divided by `FFT_SIZE` (so that the synthesis, which does not divide, gives back
/// what went in). A hop of silence is put before the sound and a frame after it, so every sample
/// is in two frames and the end is not cut.
pub fn analyse_sound(signal: &[f32], tables: &Tables) -> Spectrogram {
    let mut padded = vec![0.0; HOP];
    padded.extend_from_slice(signal);
    padded.extend(std::iter::repeat_n(0.0, FFT_SIZE));
    let frames = (padded.len() - FFT_SIZE) / HOP + 1;

    let fft = RealFftPlanner::<f32>::new().plan_fft_forward(FFT_SIZE);
    let mut input = vec![0.0f32; FFT_SIZE];
    let mut output = vec![Complex32::new(0.0, 0.0); BINS];
    let mut values = Vec::with_capacity(frames * BINS);
    let scale = 1.0 / FFT_SIZE as f32;
    for frame in 0..frames {
        let from = frame * HOP;
        for (at, sample) in input.iter_mut().enumerate() {
            *sample = padded[from + at] * tables.window[at];
        }
        fft.process(&mut input, &mut output)
            .expect("the sizes are fixed");
        values.extend(output.iter().map(|bin| bin * scale));
    }
    Spectrogram { frames, values }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A window that is 1 where the frames overlap perfectly: sine squared and cosine squared
    /// sum to one, as the real window's squares do.
    pub(crate) fn test_tables() -> Tables {
        let window = (0..FFT_SIZE)
            .map(|i| {
                let sine =
                    (0.5 * std::f64::consts::PI * (i as f64 + 0.5) / (FFT_SIZE / 2) as f64).sin();
                (0.5 * std::f64::consts::PI * sine * sine).sin() as f32
            })
            .collect();
        Tables {
            bands: vec![vec![(0, 1.0)]],
            spread: vec![vec![(0, 1.0)]],
            window,
        }
    }

    #[test]
    fn a_second_of_sound_makes_the_frames_it_should_and_a_tone_shows_in_its_bin() {
        let tables = test_tables();
        // 48000 samples: 100 hops of sound, then padding: (480 + 48000 + 960 - 960) / 480 + 1.
        let tone: Vec<f32> = (0..48_000)
            .map(|n| (2.0 * std::f32::consts::PI * 4_800.0 * n as f32 / 48_000.0).sin())
            .collect();
        let spectrogram = analyse_sound(&tone, &tables);
        assert_eq!(spectrogram.frames, 102);
        // 4800 Hz is bin 4800 / (48000 / 960) = 96, and nothing else is as strong.
        let frame = spectrogram.frame(50);
        let strongest = (0..BINS)
            .max_by(|a, b| frame[*a].norm().total_cmp(&frame[*b].norm()))
            .unwrap();
        assert_eq!(strongest, 96);
    }
}
