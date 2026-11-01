use serde_json::{json, Value};

use super::loudness_algorithm::{peak_db, rms_db};
use super::volume_window_contract::{round_db, VolumeWindow};
use metilplugins_sound_reader::Sound;

/// The sound in a few numbers: how loud it is overall, its loudest instant, and the loudest and
/// quietest windows. `null` for the windows when there are none.
pub fn summarise_volume(sound: &Sound, windows: &[VolumeWindow]) -> Value {
    let by_level = |a: &&VolumeWindow, b: &&VolumeWindow| a.rms_db.total_cmp(&b.rms_db);
    let describe = |window: Option<&VolumeWindow>| {
        window.map_or(Value::Null, |window| {
            json!({ "startMs": window.start_ms, "endMs": window.end_ms, "rmsDb": round_db(window.rms_db) })
        })
    };
    json!({
        "averageRmsDb": round_db(rms_db(&sound.samples)),
        "peakDb": round_db(peak_db(&sound.samples)),
        "loudest": describe(windows.iter().max_by(by_level)),
        "quietest": describe(windows.iter().min_by(by_level)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(start: u64, db: f64) -> VolumeWindow {
        VolumeWindow {
            start_ms: start,
            end_ms: start + 1_000,
            rms_db: db,
            peak_db: db,
            bands: None,
        }
    }

    #[test]
    fn gives_the_overall_levels_and_picks_the_loudest_and_the_quietest_windows() {
        let sound = Sound {
            samples: vec![0.5, -0.5, 0.5, -0.5],
            sample_rate: 8_000,
            start_ms: 0,
        };
        let summary = summarise_volume(
            &sound,
            &[window(0, -30.0), window(1_000, -12.0), window(2_000, -50.0)],
        );
        assert_eq!(summary["averageRmsDb"], -6.0);
        assert_eq!(summary["peakDb"], -6.0);
        assert_eq!(
            summary["loudest"],
            json!({ "startMs": 1000, "endMs": 2000, "rmsDb": -12.0 })
        );
        assert_eq!(summary["quietest"]["startMs"], 2_000);
    }

    #[test]
    fn without_windows_there_is_no_loudest_or_quietest() {
        let summary = summarise_volume(
            &Sound {
                samples: vec![0.0],
                sample_rate: 8_000,
                start_ms: 0,
            },
            &[],
        );
        assert_eq!(summary["loudest"], Value::Null);
        assert_eq!(summary["averageRmsDb"], -100.0);
    }
}
