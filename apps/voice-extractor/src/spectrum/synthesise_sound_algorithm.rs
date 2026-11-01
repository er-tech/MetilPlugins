use realfft::num_complex::Complex32;
use realfft::RealFftPlanner;

use crate::tables::{Tables, BINS, FFT_SIZE, HOP};

use super::spectrogram_contract::Spectrogram;

/// The sound a transform stands for, `length` samples of it: each frame is turned back into
/// samples, multiplied by the window again and added in at its place (the frames overlap by half,
/// and the squares of the window sum to one, so the sound comes back whole). The hop of silence
/// that `analyse_sound` put before the sound is taken off.
pub fn synthesise_sound(spectrogram: &Spectrogram, length: usize, tables: &Tables) -> Vec<f32> {
    let fft = RealFftPlanner::<f32>::new().plan_fft_inverse(FFT_SIZE);
    let mut output = vec![0.0f32; spectrogram.frames * HOP + FFT_SIZE];
    let mut bins = vec![Complex32::new(0.0, 0.0); BINS];
    let mut samples = vec![0.0f32; FFT_SIZE];
    for frame in 0..spectrogram.frames {
        bins.copy_from_slice(spectrogram.frame(frame));
        // The first and the last bin of a real sound have no imaginary part.
        bins[0].im = 0.0;
        bins[BINS - 1].im = 0.0;
        fft.process(&mut bins, &mut samples)
            .expect("the sizes are fixed");
        let from = frame * HOP;
        for (at, sample) in samples.iter().enumerate() {
            output[from + at] += sample * tables.window[at];
        }
    }
    output.drain(..HOP);
    output.truncate(length);
    output.resize(length, 0.0);
    output
}

#[cfg(test)]
mod tests {
    use super::super::analyse_sound_algorithm::analyse_sound;
    use super::*;

    fn tables() -> Tables {
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
    fn what_the_analysis_takes_apart_the_synthesis_puts_back() {
        let tables = tables();
        let signal: Vec<f32> = (0..10_000)
            .map(|n| 0.5 * (n as f32 * 0.05).sin() + 0.2 * (n as f32 * 0.31).cos())
            .collect();
        let back = synthesise_sound(&analyse_sound(&signal, &tables), signal.len(), &tables);
        assert_eq!(back.len(), signal.len());
        let worst = signal
            .iter()
            .zip(&back)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        assert!(worst < 1e-4, "the sound came back {worst} off");
    }

    #[test]
    fn an_empty_sound_stays_empty() {
        let tables = tables();
        assert!(synthesise_sound(&analyse_sound(&[], &tables), 0, &tables).is_empty());
    }
}
