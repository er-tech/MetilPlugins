use metilplugins_sound_reader::Sound;

use super::band_levels_algorithm::{BandAnalyser, BandEdges};
use super::loudness_algorithm::{peak_db, rms_db};
use super::volume_window_contract::VolumeWindow;

/// What to measure: how long each window is and whether to split it into bands.
#[derive(Debug, Clone, Copy)]
pub struct VolumeOptions {
    pub window_ms: u64,
    /// `Some` to give the level of the low, mid and high bands too.
    pub bands: Option<BandEdges>,
}

/// Measures the sound window by window, one after the other with none missing or overlapping.
/// Times are in the source: a stretch the host cut out starts at its `start_ms`. A
/// last window shorter than a quarter of the others is left out: so little sound says little.
pub fn measure_volume(sound: &Sound, options: VolumeOptions) -> Vec<VolumeWindow> {
    let rate = u64::from(sound.sample_rate);
    let window_samples = ((rate * options.window_ms) / 1000).max(1) as usize;
    let mut analyser = BandAnalyser::new();
    let mut windows = Vec::new();

    for (index, samples) in sound.samples.chunks(window_samples).enumerate() {
        if samples.len() * 4 < window_samples && index > 0 {
            break;
        }
        let first = index * window_samples;
        let bands = options
            .bands
            .map(|edges| analyser.levels(samples, sound.sample_rate, edges));
        windows.push(VolumeWindow {
            start_ms: sound.start_ms + first as u64 * 1000 / rate,
            end_ms: sound.start_ms + (first + samples.len()) as u64 * 1000 / rate,
            rms_db: rms_db(samples),
            peak_db: peak_db(samples),
            bands,
        });
    }
    windows
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A second of 440 Hz at the first amplitude, then a second of the second.
    fn two_levels(first: f32, second: f32) -> Sound {
        let tone = |amplitude: f32| {
            (0..16_000).map(move |n| {
                amplitude * (2.0 * std::f32::consts::PI * 440.0 * n as f32 / 16_000.0).sin()
            })
        };
        Sound {
            samples: tone(first).chain(tone(second)).collect(),
            sample_rate: 16_000,
            start_ms: 0,
        }
    }

    #[test]
    fn follows_the_loudness_second_by_second() {
        let windows = measure_volume(
            &two_levels(0.5, 0.05),
            VolumeOptions {
                window_ms: 1_000,
                bands: None,
            },
        );
        assert_eq!(windows.len(), 2);
        assert_eq!((windows[0].start_ms, windows[0].end_ms), (0, 1_000));
        assert_eq!((windows[1].start_ms, windows[1].end_ms), (1_000, 2_000));
        assert!(
            (windows[0].rms_db + 9.0).abs() < 0.2,
            "{}",
            windows[0].rms_db
        );
        assert!(
            (windows[1].rms_db + 29.0).abs() < 0.2,
            "{}",
            windows[1].rms_db
        );
        assert!(windows[0].bands.is_none());
    }

    #[test]
    fn splits_each_window_into_bands_when_asked_and_the_tone_sits_in_the_mid_band() {
        let options = VolumeOptions {
            window_ms: 500,
            bands: Some(BandEdges::default()),
        };
        let windows = measure_volume(&two_levels(0.5, 0.5), options);
        assert_eq!(windows.len(), 4);
        let bands = windows[0].bands.expect("bands were asked for");
        assert!((bands.mid + 9.0).abs() < 0.8, "{bands:?}");
        assert!(bands.low < -40.0 && bands.high < -40.0, "{bands:?}");
    }

    #[test]
    fn a_short_remainder_is_left_out_and_a_longer_one_is_kept_with_its_real_end() {
        let sound = |seconds: f32| Sound {
            samples: vec![0.1; (8_000.0 * seconds) as usize],
            sample_rate: 8_000,
            start_ms: 0,
        };
        let options = VolumeOptions {
            window_ms: 1_000,
            bands: None,
        };
        // 2.1 s: the 0.1 s left over is a tenth of a window.
        assert_eq!(measure_volume(&sound(2.1), options).len(), 2);
        // 2.5 s: half a window is kept and ends where the sound ends.
        let windows = measure_volume(&sound(2.5), options);
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[2].end_ms, 2_500);
        // A stretch cut out of a longer sound keeps the time of the source.
        let stretch = Sound {
            start_ms: 4_000,
            ..sound(2.0)
        };
        let windows = measure_volume(&stretch, options);
        assert_eq!((windows[0].start_ms, windows[1].end_ms), (4_000, 6_000));
        // A sound shorter than a window still gives one.
        assert_eq!(measure_volume(&sound(0.3), options).len(), 1);
        assert!(measure_volume(
            &Sound {
                samples: vec![],
                sample_rate: 8_000,
                start_ms: 0
            },
            options
        )
        .is_empty());
    }
}
