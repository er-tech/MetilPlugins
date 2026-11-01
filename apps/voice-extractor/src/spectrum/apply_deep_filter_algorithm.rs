use realfft::num_complex::Complex32;

use crate::tables::{Tables, BANDS, BINS, FILTER_BINS, FILTER_LOOKAHEAD, FILTER_ORDER};

use super::spectrogram_contract::Spectrogram;

/// What the network answered for a sound.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkAnswer {
    /// A gain from 0 to 1 for each band of each frame, `BANDS` for each frame.
    pub mask: Vec<f32>,
    /// For each of `FILTER_ORDER` taps, each frame and each of the `FILTER_BINS` lowest
    /// frequencies, a complex number (real, imaginary).
    pub coefficients: Vec<f32>,
}

/// The spectrum of the voice: the noisy one with each band turned down by its gain, except in the
/// `FILTER_BINS` lowest frequencies (where the voice is) that are rebuilt by the deep filter: each
/// is a sum of the same frequency in `FILTER_ORDER` frames of the *noisy* spectrum, from two before
/// the frame to two after it, each multiplied by its own complex coefficient.
pub fn apply_deep_filter(
    noisy: &Spectrogram,
    answer: &NetworkAnswer,
    tables: &Tables,
) -> Spectrogram {
    let frames = noisy.frames;
    let mut values = Vec::with_capacity(frames * BINS);
    let mut gains = vec![0f32; BINS];
    for frame in 0..frames {
        gains.fill(0.0);
        for (band, members) in tables.spread.iter().enumerate() {
            let mask = answer.mask[frame * BANDS + band];
            for (bin, weight) in members {
                gains[*bin] += mask * weight;
            }
        }
        let start = values.len();
        values.extend(
            noisy
                .frame(frame)
                .iter()
                .zip(&gains)
                .map(|(bin, gain)| bin * gain),
        );
        for bin in 0..FILTER_BINS {
            let mut sum = Complex32::new(0.0, 0.0);
            for tap in 0..FILTER_ORDER {
                // The frame this tap reads: tap 0 is two before, tap 4 two after.
                let Some(source) = (frame + tap).checked_sub(FILTER_LOOKAHEAD) else {
                    continue;
                };
                if source >= frames {
                    continue;
                }
                let at = ((tap * frames + frame) * FILTER_BINS + bin) * 2;
                let coefficient =
                    Complex32::new(answer.coefficients[at], answer.coefficients[at + 1]);
                sum += coefficient * noisy.frame(source)[bin];
            }
            values[start + bin] = sum;
        }
    }
    Spectrogram { frames, values }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables() -> Tables {
        Tables {
            bands: (0..BANDS).map(|band| vec![(band, 1.0)]).collect(),
            // Each band gives its gain to 15 frequencies, the last one to the 16 that remain.
            spread: (0..BANDS)
                .map(|band| {
                    let end = if band == BANDS - 1 {
                        BINS
                    } else {
                        band * 15 + 15
                    };
                    (band * 15..end).map(|bin| (bin, 1.0)).collect()
                })
                .collect(),
            window: vec![1.0; 960],
        }
    }

    fn noisy(frames: usize) -> Spectrogram {
        Spectrogram {
            frames,
            values: (0..frames * BINS)
                .map(|n| Complex32::new((n % 7) as f32, (n % 3) as f32))
                .collect(),
        }
    }

    fn identity_answer(frames: usize, mask: f32) -> NetworkAnswer {
        let mut coefficients = vec![0f32; FILTER_ORDER * frames * FILTER_BINS * 2];
        // The middle tap (the frame itself) is 1 + 0i, the others 0.
        for frame in 0..frames {
            for bin in 0..FILTER_BINS {
                coefficients[((FILTER_LOOKAHEAD * frames + frame) * FILTER_BINS + bin) * 2] = 1.0;
            }
        }
        NetworkAnswer {
            mask: vec![mask; frames * BANDS],
            coefficients,
        }
    }

    #[test]
    fn a_filter_that_only_keeps_the_frame_and_a_full_mask_give_back_the_noisy_spectrum() {
        let noisy = noisy(6);
        let enhanced = apply_deep_filter(&noisy, &identity_answer(6, 1.0), &tables());
        assert_eq!(enhanced, noisy);
    }

    #[test]
    fn a_closed_mask_silences_the_bins_above_the_filter_and_not_the_ones_the_filter_rebuilds() {
        let noisy = noisy(6);
        let enhanced = apply_deep_filter(&noisy, &identity_answer(6, 0.0), &tables());
        let frame = enhanced.frame(3);
        assert!(frame[FILTER_BINS..BANDS * 15]
            .iter()
            .all(|bin| bin.norm() == 0.0));
        assert_eq!(frame[..FILTER_BINS], noisy.frame(3)[..FILTER_BINS]);
    }

    #[test]
    fn a_tap_reads_the_frame_it_is_named_for_and_nothing_outside_the_sound() {
        let noisy = noisy(5);
        let mut answer = identity_answer(5, 1.0);
        // Make the filter read only the tap that looks two frames ahead (tap 4).
        for frame in 0..5 {
            for bin in 0..FILTER_BINS {
                answer.coefficients[((FILTER_LOOKAHEAD * 5 + frame) * FILTER_BINS + bin) * 2] = 0.0;
                answer.coefficients[((4 * 5 + frame) * FILTER_BINS + bin) * 2] = 1.0;
            }
        }
        let enhanced = apply_deep_filter(&noisy, &answer, &tables());
        assert_eq!(enhanced.frame(1)[10], noisy.frame(3)[10]);
        assert_eq!(
            enhanced.frame(4)[10],
            Complex32::new(0.0, 0.0),
            "frame 6 does not exist"
        );
    }
}
