use serde_json::{json, Value};
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use loomasplugins_sound_reader::{read_audio_region, resample, write_audio_region, Sound};
use loomasplugins_work_files::write_wav_file;

use crate::enhance::{enhance_voice, level_db, split_voice};
use crate::models::locate_models;
use crate::network::device_report;
use crate::tables::read_tables;

use super::read_extract_options_mapper::read_extract_options;

/// The rate the model works at, and the one the manifest asks Loomas to decode to.
const MODEL_RATE: u32 = 48_000;

/// The `extract` action: the voice of a video and the rest, as two tracks.
///
/// Params: `media` (the sound of a video, decoded by the host to 48 kHz, one channel), `keep`,
/// `sampleRate`, `rest` and `files` (see `read_extract_options`). It gives
/// `{ voice, rest, voiceDb, restDb, durationMs }`: `voice` and `rest` are the tracks described as
/// Loomas describes decoded sound, so the next plugin (speech, sounds) is given one as it is
/// (good for this run only); `voiceDb` and `restDb` say how loud they are. With `files` each track
/// also comes as a WAV in the result, as `voiceFile` and `restFile`.
pub fn extract_voice(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_extract_options(params)?;
    // A call that asked for `auto` runs where the host grants; the default stays the processor.
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video"))?;
    let sound = read_audio_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| {
            Failure::status(
                400,
                "media must be sound Loomas decoded for this plugin (it asks for 48 kHz sound)",
            )
        })?;
    if sound.sample_rate != MODEL_RATE {
        return Err(Failure::status(
            400,
            format!(
                "this plugin works at {MODEL_RATE} Hz, not {} Hz",
                sound.sample_rate
            ),
        ));
    }
    let folder = cancel.work_dir().ok_or_else(|| {
        Failure::status(
            503,
            "Loomas gave this plugin no place to write the tracks (it needs the outputs feature)",
        )
    })?;

    let paths = locate_models(cancel)?;
    let table_bytes = std::fs::read(&paths.tables).map_err(|error| {
        Failure::status(
            422,
            format!(
                "the tables cannot be read from {}: {error}",
                paths.tables.display()
            ),
        )
    })?;
    let tables = read_tables(&table_bytes).map_err(|message| Failure::status(422, message))?;

    let enhanced = enhance_voice(
        &sound.samples,
        &tables,
        &paths.network,
        options.device,
        cancel,
    )?;
    let (voice, rest) = split_voice(&sound.samples, &enhanced, options.keep);

    let track = |samples: Vec<f32>| Sound {
        samples: resample(&samples, MODEL_RATE, options.sample_rate),
        sample_rate: options.sample_rate,
        start_ms: sound.start_ms,
    };
    let (voice_db, rest_db) = (level_db(&voice), level_db(&rest));
    let (voice, rest) = (track(voice), track(rest));
    let write = |name: &str, sound: &Sound| {
        write_audio_region(folder, name, sound).map_err(|error| {
            Failure::status(500, format!("cannot write the {name} track: {error}"))
        })
    };
    let mut result = json!({
        "voice": write("voice", &voice)?,
        "voiceDb": round(voice_db),
        "restDb": round(rest_db),
        "durationMs": sound.duration_ms(),
        "device": device_report().map(|report| report.to_value()),
    });
    if options.rest {
        result["rest"] = write("rest", &rest)?;
    }
    if options.files {
        let wav = |prefix: &str, sound: &Sound| {
            write_wav_file(prefix, &sound.samples, sound.sample_rate, folder)
                .map_err(|error| Failure::status(500, error.to_string()))
        };
        result["voiceFile"] = wav("voice", &voice)?;
        if options.rest {
            result["restFile"] = wav("rest", &rest)?;
        }
    }
    Ok(result)
}

fn round(decibels: f64) -> f64 {
    (decibels * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_in_place_of_decoded_sound_and_a_missing_media_are_400s() {
        let failure = extract_voice(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = extract_voice(&json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }

    #[test]
    fn a_wrong_param_is_refused_before_anything_is_read_or_downloaded() {
        let failure = extract_voice(
            &json!({ "media": "a.mp4", "sampleRate": 44100 }),
            &Cancel::default(),
        )
        .unwrap_err();
        assert!(
            failure.message.contains("sampleRate"),
            "{}",
            failure.message
        );
    }
}
