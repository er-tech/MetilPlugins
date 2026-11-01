use serde_json::{json, Value};
use metilplugins_plugin_sdk::{number, Cancel, Failure};
use metilplugins_sound_reader::{load_sound_input, write_audio_region, Sound};

use crate::denoise::{level_db, reduce_noise, NoiseOptions};

/// The `reduceNoise` action: the sound with its steady noise (a hum, a fan, hiss) taken out, written
/// for the next plugin to read.
///
/// Params: `audio`, `strength` (0 to 1, default 0.5: how much of the noise is taken out; more also
/// takes a little of the voice), `floorDb` (-60 to -3, default -20: the quietest a frequency is
/// made, so the sound does not turn into bubbles). It gives `{ audio, reducedDb }`: the description
/// of the cleaned sound, and how much quieter the quietest tenth of it became.
pub fn reduce_noise_in_sound(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let strength = number(params, "strength", 0.5, 0.0, 1.0)?;
    let floor_db = number(params, "floorDb", -20.0, -60.0, -3.0)?;
    let folder = cancel.work_dir().ok_or_else(|| {
        Failure::status(
            503,
            "Metil gave this plugin no place to write the cleaned sound (it needs the outputs feature)",
        )
    })?;
    let sound =
        load_sound_input(params).map_err(|error| Failure::status(error.code, error.message))?;
    cancel.progress("cleaning");

    let cleaned = Sound {
        samples: reduce_noise(
            &sound.samples,
            NoiseOptions {
                strength: strength as f32,
                floor: 10f32.powf(floor_db as f32 / 20.0),
            },
        ),
        sample_rate: sound.sample_rate,
        start_ms: sound.start_ms,
    };
    if cancel.is_cancelled() {
        return Err(Failure::named("CANCELLED", "cancelled"));
    }
    let described = write_audio_region(folder, "reduce-noise", &cleaned).map_err(|error| {
        Failure::status(500, format!("cannot write the cleaned sound: {error}"))
    })?;
    Ok(json!({
        "audio": described,
        "reducedDb": (quiet_level(&sound.samples) - quiet_level(&cleaned.samples)).max(0.0),
    }))
}

/// The level of the quietest tenth of the sound, in dB: the noise, when there is some.
fn quiet_level(samples: &[f32]) -> f64 {
    let block = 512;
    let mut levels: Vec<f64> = samples.chunks(block).map(level_db).collect();
    if levels.is_empty() {
        return -120.0;
    }
    levels.sort_by(f64::total_cmp);
    let tenth = (levels.len() / 10).max(1);
    levels[..tenth].iter().sum::<f64>() / tenth as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work_folder(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("sound-clean-{}-{name}", std::process::id()))
    }

    #[test]
    fn without_a_place_to_write_it_is_a_503_that_says_so() {
        let failure =
            reduce_noise_in_sound(&json!({ "audio": "a.wav" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(503)));
        assert!(failure.message.contains("outputs"));
    }

    #[test]
    fn params_out_of_range_are_a_400() {
        let failure =
            reduce_noise_in_sound(&json!({ "strength": 2 }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let _ = work_folder("unused");
    }
}
