use serde_json::{json, Value};
use metilplugins_plugin_sdk::Failure;
use metilplugins_sound_reader::load_sound_input;

use crate::volume::{find_silences, measure_volume, VolumeOptions};

use metilplugins_plugin_sdk::{number, whole_number};

/// The `silences` action: the stretches where the sound stays below a level.
///
/// Params: `audio`, `thresholdDb` (-100 to 0, default -50), `minMs` (default 500), `windowMs`
/// (10 to 10000, default 100: how finely the sound is looked at), `sampleRate`.
pub fn find_silent_stretches(params: &Value) -> Result<Value, Failure> {
    let threshold_db = number(params, "thresholdDb", -50.0, -100.0, 0.0)?;
    let min_ms = whole_number(params, "minMs", 500, 0, 3_600_000)?;
    let window_ms = whole_number(params, "windowMs", 100, 10, 10_000)?;
    let sound =
        load_sound_input(params).map_err(|error| Failure::status(error.code, error.message))?;

    let windows = measure_volume(
        &sound,
        VolumeOptions {
            window_ms,
            bands: None,
        },
    );
    let silences = find_silences(&windows, threshold_db, min_ms);
    let total: u64 = silences
        .iter()
        .map(|silence| silence.end_ms - silence.start_ms)
        .sum();
    Ok(json!({
        "durationMs": sound.duration_ms(),
        "thresholdDb": threshold_db,
        "minMs": min_ms,
        "silences": silences.iter().map(|silence| json!({
            "startMs": silence.start_ms,
            "endMs": silence.end_ms,
            "durationMs": silence.end_ms - silence.start_ms,
        })).collect::<Vec<_>>(),
        "totalSilenceMs": total,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wav file: 0.5 s of a tone, 1 s of silence, 0.5 s of a tone, 0.2 s of silence, 0.3 s of a tone.
    fn with_gaps(name: &str) -> String {
        let rate = 8_000_u32;
        let part = |seconds: f32, amplitude: f32| -> Vec<i16> {
            (0..(rate as f32 * seconds) as usize)
                .map(|n| {
                    (amplitude
                        * (2.0 * std::f32::consts::PI * 440.0 * n as f32 / rate as f32).sin()
                        * 32_767.0) as i16
                })
                .collect()
        };
        let samples: Vec<i16> = [
            part(0.5, 0.5),
            part(1.0, 0.0),
            part(0.5, 0.5),
            part(0.2, 0.0),
            part(0.3, 0.5),
        ]
        .concat();
        let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        let mut file = b"RIFF".to_vec();
        file.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        file.extend_from_slice(b"WAVEfmt ");
        for chunk in [
            16u32.to_le_bytes().as_slice(),
            &1u16.to_le_bytes(),
            &1u16.to_le_bytes(),
            &rate.to_le_bytes(),
            &(rate * 2).to_le_bytes(),
            &2u16.to_le_bytes(),
            &16u16.to_le_bytes(),
        ] {
            file.extend_from_slice(chunk);
        }
        file.extend_from_slice(b"data");
        file.extend_from_slice(&(data.len() as u32).to_le_bytes());
        file.extend_from_slice(&data);
        let path = std::env::temp_dir().join(format!(
            "sound-volume-{}-{name}-gaps.wav",
            std::process::id()
        ));
        std::fs::write(&path, file).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn finds_the_quiet_stretches_that_last_long_enough() {
        let file = with_gaps("t1");
        // The 1 s gap counts; the 0.2 s gap is under the default minimum of 500 ms.
        let found = find_silent_stretches(&json!({ "audio": file })).unwrap();
        let silences = found["silences"].as_array().unwrap();
        assert_eq!(silences.len(), 1, "{found}");
        assert_eq!(
            (
                silences[0]["startMs"].as_u64().unwrap(),
                silences[0]["endMs"].as_u64().unwrap()
            ),
            (500, 1_500)
        );
        assert_eq!(found["totalSilenceMs"], 1_000);
        assert_eq!(found["durationMs"], 2_500);

        // With a shorter minimum both are found.
        let both = find_silent_stretches(&json!({ "audio": file, "minMs": 100 })).unwrap();
        assert_eq!(both["silences"].as_array().unwrap().len(), 2);
        assert_eq!(both["silences"][1]["startMs"], 2_000);
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn the_threshold_decides_what_is_quiet_and_wrong_params_are_a_400() {
        let file = with_gaps("t2");
        // Nothing is quieter than -100 dB here except real silence; at 0 dB everything is "quiet".
        let everything =
            find_silent_stretches(&json!({ "audio": file, "thresholdDb": 0, "minMs": 0 })).unwrap();
        assert_eq!(everything["totalSilenceMs"], 2_500);
        assert_eq!(
            find_silent_stretches(&json!({ "audio": file, "thresholdDb": 5 }))
                .unwrap_err()
                .code,
            Some(json!(400))
        );
        assert_eq!(
            find_silent_stretches(&json!({ "audio": file, "windowMs": 1 }))
                .unwrap_err()
                .code,
            Some(json!(400))
        );
        let _ = std::fs::remove_file(file);
    }
}
