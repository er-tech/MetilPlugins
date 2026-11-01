use std::collections::HashMap;
use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use super::loudness_algorithm::power_to_db;

/// Where the three bands split, in hertz: below `low_mid` is low, from there to `mid_high` is mid,
/// from there up is high.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandEdges {
    pub low_mid: f64,
    pub mid_high: f64,
}

impl Default for BandEdges {
    /// Bass and kick below 250 Hz, voice and most instruments up to 4 kHz, then the hiss and air.
    fn default() -> Self {
        BandEdges {
            low_mid: 250.0,
            mid_high: 4_000.0,
        }
    }
}

/// The level of each band in dB (relative to full scale).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandLevels {
    pub low: f64,
    pub mid: f64,
    pub high: f64,
}

/// Splits sounds into bands to say how loud each is. It keeps the plans of the transforms it has
/// made, since a sound has many windows of the same length.
pub struct BandAnalyser {
    planner: FftPlanner<f64>,
    plans: HashMap<usize, Arc<dyn Fft<f64>>>,
}

impl BandAnalyser {
    pub fn new() -> Self {
        BandAnalyser {
            planner: FftPlanner::new(),
            plans: HashMap::new(),
        }
    }

    /// The level of each band in a window of samples recorded at `sample_rate`.
    ///
    /// The window is shaped with a Hann curve and transformed; the energy of the bins in each band
    /// is added up, so that the three bands together are the level of the whole window.
    pub fn levels(&mut self, samples: &[f32], sample_rate: u32, edges: BandEdges) -> BandLevels {
        let n = samples.len();
        if n < 8 {
            return BandLevels {
                low: power_to_db(0.0),
                mid: power_to_db(0.0),
                high: power_to_db(0.0),
            };
        }
        let fft = Arc::clone(
            self.plans
                .entry(n)
                .or_insert_with(|| self.planner.plan_fft_forward(n)),
        );

        let (mut windowed, mut window_energy) = (Vec::with_capacity(n), 0.0_f64);
        for (index, sample) in samples.iter().enumerate() {
            let hann =
                0.5 - 0.5 * (2.0 * std::f64::consts::PI * index as f64 / (n as f64 - 1.0)).cos();
            window_energy += hann * hann;
            windowed.push(Complex::new(f64::from(*sample) * hann, 0.0));
        }
        fft.process(&mut windowed);

        // Parseval: the mean square of the samples is the sum of |X|^2 over all bins divided by
        // n times the energy of the window. Only the first half is kept, so the others count twice.
        let scale = 1.0 / (n as f64 * window_energy);
        let (mut low, mut mid, mut high) = (0.0_f64, 0.0_f64, 0.0_f64);
        for (bin, value) in windowed.iter().enumerate().take(n / 2 + 1) {
            let twice = if bin == 0 || (n.is_multiple_of(2) && bin == n / 2) {
                1.0
            } else {
                2.0
            };
            let power = value.norm_sqr() * twice * scale;
            let hertz = bin as f64 * f64::from(sample_rate) / n as f64;
            if hertz < edges.low_mid {
                low += power;
            } else if hertz < edges.mid_high {
                mid += power;
            } else {
                high += power;
            }
        }
        BandLevels {
            low: power_to_db(low),
            mid: power_to_db(mid),
            high: power_to_db(high),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(hertz: f32, sample_rate: u32, amplitude: f32, seconds: f32) -> Vec<f32> {
        (0..(sample_rate as f32 * seconds) as usize)
            .map(|n| {
                amplitude
                    * (2.0 * std::f32::consts::PI * hertz * n as f32 / sample_rate as f32).sin()
            })
            .collect()
    }

    #[test]
    fn a_tone_shows_up_in_its_band_at_its_level_and_the_others_stay_quiet() {
        let mut analyser = BandAnalyser::new();
        let edges = BandEdges::default();
        // Amplitude 0.5 is about -9 dB.
        let low = analyser.levels(&sine(100.0, 32_000, 0.5, 1.0), 32_000, edges);
        assert!((low.low + 9.03).abs() < 0.6, "{low:?}");
        assert!(low.mid < -40.0 && low.high < -40.0, "{low:?}");

        let mid = analyser.levels(&sine(1_000.0, 32_000, 0.5, 1.0), 32_000, edges);
        assert!((mid.mid + 9.03).abs() < 0.6, "{mid:?}");
        assert!(mid.low < -40.0 && mid.high < -40.0, "{mid:?}");

        let high = analyser.levels(&sine(8_000.0, 32_000, 0.5, 1.0), 32_000, edges);
        assert!((high.high + 9.03).abs() < 0.6, "{high:?}");
        assert!(high.low < -40.0 && high.mid < -40.0, "{high:?}");
    }

    #[test]
    fn two_tones_in_different_bands_are_each_measured_on_their_own() {
        let mut analyser = BandAnalyser::new();
        let loud = sine(150.0, 16_000, 0.8, 0.5);
        let quiet = sine(2_000.0, 16_000, 0.1, 0.5);
        let both: Vec<f32> = loud.iter().zip(&quiet).map(|(a, b)| a + b).collect();
        let levels = analyser.levels(&both, 16_000, BandEdges::default());
        // 0.8 is about -4.9 dB and 0.1 about -23 dB.
        assert!((levels.low + 4.95).abs() < 0.7, "{levels:?}");
        assert!((levels.mid + 23.0).abs() < 0.7, "{levels:?}");
    }

    #[test]
    fn the_edges_move_the_bands_and_silence_or_a_too_short_window_is_the_floor() {
        let mut analyser = BandAnalyser::new();
        // A 300 Hz tone is mid with the default edges and low when the edge is 500.
        let tone = sine(300.0, 16_000, 0.5, 0.5);
        assert!(analyser.levels(&tone, 16_000, BandEdges::default()).mid > -12.0);
        let moved = analyser.levels(
            &tone,
            16_000,
            BandEdges {
                low_mid: 500.0,
                mid_high: 4_000.0,
            },
        );
        assert!(moved.low > -12.0 && moved.mid < -40.0, "{moved:?}");
        let quiet = analyser.levels(&vec![0.0; 1_000], 16_000, BandEdges::default());
        assert_eq!((quiet.low, quiet.mid, quiet.high), (-100.0, -100.0, -100.0));
        assert_eq!(
            analyser.levels(&[0.5; 3], 16_000, BandEdges::default()).mid,
            -100.0
        );
    }
}
