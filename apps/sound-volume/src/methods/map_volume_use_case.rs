use serde_json::{json, Value};
use loomasplugins_plugin_sdk::Failure;
use loomasplugins_sound_reader::load_sound_input;

use crate::volume::{measure_volume, summarise_volume, BandEdges, VolumeOptions};

use loomasplugins_plugin_sdk::{flag, number, whole_number};

/// The `map` action: the level of the sound, window by window, with the low, mid and high bands.
///
/// Params: `audio`, `windowMs` (10 to 60000, default 1000), `sampleRate` (to listen at, 4000 to
/// 192000; the highest frequency that can be told is half of it), `bands` (default true),
/// `lowMidHz` (default 250) and `midHighHz` (default 4000).
pub fn map_volume(params: &Value) -> Result<Value, Failure> {
    let window_ms = whole_number(params, "windowMs", 1_000, 10, 60_000)?;
    let bands = if flag(params, "bands", true)? {
        let low_mid = number(params, "lowMidHz", 250.0, 1.0, 96_000.0)?;
        let mid_high = number(params, "midHighHz", 4_000.0, 1.0, 96_000.0)?;
        if low_mid >= mid_high {
            return Err(Failure::status(
                400,
                format!("lowMidHz ({low_mid}) must be below midHighHz ({mid_high})"),
            ));
        }
        Some(BandEdges { low_mid, mid_high })
    } else {
        None
    };
    let sound =
        load_sound_input(params).map_err(|error| Failure::status(error.code, error.message))?;

    let windows = measure_volume(&sound, VolumeOptions { window_ms, bands });
    Ok(json!({
        "sampleRate": sound.sample_rate,
        "durationMs": sound.duration_ms(),
        "windowMs": window_ms,
        "bandsHz": bands.map(|edges| json!({ "lowBelow": edges.low_mid, "highFrom": edges.mid_high })),
        "windows": windows.iter().map(|window| window.to_value()).collect::<Vec<_>>(),
        "summary": summarise_volume(&sound, &windows),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wav file: a second of a 200 Hz tone, then a second of a 2 kHz tone at a tenth of the level.
    fn two_tones(name: &str) -> String {
        let rate = 16_000_u32;
        let tone = |hertz: f32, amplitude: f32| -> Vec<i16> {
            (0..rate)
                .map(|n| {
                    (amplitude
                        * (2.0 * std::f32::consts::PI * hertz * n as f32 / rate as f32).sin()
                        * 32_767.0) as i16
                })
                .collect()
        };
        let samples: Vec<i16> = tone(200.0, 0.5)
            .into_iter()
            .chain(tone(2_000.0, 0.05))
            .collect();
        let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        let mut file = b"RIFF".to_vec();
        file.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        file.extend_from_slice(b"WAVEfmt ");
        for part in [
            16u32.to_le_bytes().as_slice(),
            &1u16.to_le_bytes(),
            &1u16.to_le_bytes(),
            &rate.to_le_bytes(),
            &(rate * 2).to_le_bytes(),
            &2u16.to_le_bytes(),
            &16u16.to_le_bytes(),
        ] {
            file.extend_from_slice(part);
        }
        file.extend_from_slice(b"data");
        file.extend_from_slice(&(data.len() as u32).to_le_bytes());
        file.extend_from_slice(&data);
        let path = std::env::temp_dir().join(format!(
            "sound-volume-{}-{name}-two-tones.wav",
            std::process::id()
        ));
        std::fs::write(&path, file).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn maps_the_level_and_the_bands_of_a_sound_second_by_second() {
        let file = two_tones("t1");
        let result = map_volume(&json!({ "audio": { "file": file }, "windowMs": 1000 })).unwrap();
        assert_eq!(result["sampleRate"], 16_000);
        assert_eq!(result["durationMs"], 2_000);
        let windows = result["windows"].as_array().unwrap();
        assert_eq!(windows.len(), 2);

        // The first second is loud and low, the second is quiet and mid.
        let (first, second) = (&windows[0], &windows[1]);
        assert!(
            (first["rmsDb"].as_f64().unwrap() + 9.0).abs() < 0.3,
            "{first}"
        );
        assert!(
            (second["rmsDb"].as_f64().unwrap() + 29.0).abs() < 0.3,
            "{second}"
        );
        assert!(
            first["bands"]["low"].as_f64().unwrap() > -11.0
                && first["bands"]["mid"].as_f64().unwrap() < -40.0,
            "{first}"
        );
        assert!(
            second["bands"]["mid"].as_f64().unwrap() > -31.0
                && second["bands"]["low"].as_f64().unwrap() < -40.0,
            "{second}"
        );
        assert_eq!(result["summary"]["loudest"]["startMs"], 0);
        assert_eq!(result["summary"]["quietest"]["startMs"], 1_000);
        assert_eq!(
            result["bandsHz"],
            json!({ "lowBelow": 250.0, "highFrom": 4000.0 })
        );
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn can_leave_out_the_bands_listen_at_another_rate_and_move_the_edges() {
        let file = two_tones("t2");
        let plain = map_volume(
            &json!({ "audio": file, "bands": false, "sampleRate": 8000, "windowMs": 500 }),
        )
        .unwrap();
        assert_eq!(plain["sampleRate"], 8_000);
        assert_eq!(plain["windows"].as_array().unwrap().len(), 4);
        assert!(plain["windows"][0].get("bands").is_none());
        assert_eq!(plain["bandsHz"], Value::Null);

        // With the low edge at 500 Hz the 200 Hz tone is still low; with the edge at 100 it is mid.
        let moved =
            map_volume(&json!({ "audio": file, "lowMidHz": 100, "windowMs": 1000 })).unwrap();
        assert!(
            moved["windows"][0]["bands"]["mid"].as_f64().unwrap() > -11.0,
            "{moved}"
        );
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn wrong_params_and_files_that_cannot_be_read_fail_with_the_status_that_fits() {
        let file = two_tones("t3");
        let code = |params: Value| map_volume(&params).unwrap_err().code;
        assert_eq!(
            code(json!({ "audio": file, "windowMs": 5 })),
            Some(json!(400))
        );
        assert_eq!(
            code(json!({ "audio": file, "lowMidHz": 5000 })),
            Some(json!(400))
        );
        assert_eq!(
            code(json!({ "audio": file, "bands": "yes" })),
            Some(json!(400))
        );
        assert_eq!(code(json!({})), Some(json!(400)));
        assert_eq!(
            code(json!({ "audio": "no-such-file.mp3" })),
            Some(json!(404))
        );
        let _ = std::fs::remove_file(file);
    }
}
