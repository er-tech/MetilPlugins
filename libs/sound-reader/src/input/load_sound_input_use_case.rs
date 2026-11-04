use std::path::PathBuf;

use serde_json::Value;

use crate::read::{read_sound, ReadSoundError, Sound};
use crate::region::read_audio_region;
use crate::resample::resample;

use super::sound_input_error::SoundInputError;

/// The lowest and highest sample rate an action may ask for.
const SAMPLE_RATES: std::ops::RangeInclusive<u64> = 4_000..=192_000;

/// Reads the sound an action was given.
///
/// `params.audio` may also be a region Loomas decoded for the plugin (see `read_audio_region`).
///
/// `params.audio` is a stored file as Loomas passes it (an object with the absolute path in
/// `file`), the sound of a video as `media.open` describes it (also an object with `file`), or just
/// a path in text. `params.sampleRate`, when there is one, is the rate to listen at: the sound is
/// resampled to it before anything is measured.
pub fn load_sound_input(params: &Value) -> Result<Sound, SoundInputError> {
    let sound = match params
        .get("audio")
        .map(read_audio_region)
        .transpose()?
        .flatten()
    {
        Some(decoded) => decoded,
        None => {
            let path = audio_path(params)?;
            read_sound(&path).map_err(|error| match error {
                ReadSoundError::Open { .. } => SoundInputError::new(404, error.to_string()),
                ReadSoundError::Unsupported(_) | ReadSoundError::Decode(_) => {
                    SoundInputError::new(422, error.to_string())
                }
            })?
        }
    };

    match params.get("sampleRate") {
        None | Some(Value::Null) => Ok(sound),
        Some(wanted) => {
            let rate = wanted
                .as_u64()
                .filter(|rate| SAMPLE_RATES.contains(rate))
                .ok_or_else(|| {
                    SoundInputError::new(
                        400,
                        format!(
                            "sampleRate must be a whole number from {} to {}, not {wanted}",
                            SAMPLE_RATES.start(),
                            SAMPLE_RATES.end()
                        ),
                    )
                })? as u32;
            Ok(Sound {
                samples: resample(&sound.samples, sound.sample_rate, rate),
                sample_rate: rate,
                start_ms: sound.start_ms,
            })
        }
    }
}

fn audio_path(params: &Value) -> Result<PathBuf, SoundInputError> {
    let audio = params.get("audio").ok_or_else(|| {
        SoundInputError::new(
            400,
            "audio is missing: give a stored file (an object with file) or a path",
        )
    })?;
    match audio {
        Value::String(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        Value::Object(fields) => fields
            .get("file")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| {
                SoundInputError::new(400, "audio has no file, so there is nothing to open")
            }),
        other => Err(SoundInputError::new(
            400,
            format!("audio must be a stored file or a path in text, not {other}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn wav(sample_rate: u32, seconds: u32) -> Vec<u8> {
        let frames = (sample_rate * seconds) as usize;
        let data: Vec<u8> = (0..frames)
            .flat_map(|n| (((n % 100) as i16 - 50) * 300).to_le_bytes())
            .collect();
        let mut file = b"RIFF".to_vec();
        file.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        file.extend_from_slice(b"WAVEfmt ");
        for part in [
            16u32.to_le_bytes().as_slice(),
            &1u16.to_le_bytes(),
            &1u16.to_le_bytes(),
            &sample_rate.to_le_bytes(),
            &(sample_rate * 2).to_le_bytes(),
            &2u16.to_le_bytes(),
            &16u16.to_le_bytes(),
        ] {
            file.extend_from_slice(part);
        }
        file.extend_from_slice(b"data");
        file.extend_from_slice(&(data.len() as u32).to_le_bytes());
        file.extend_from_slice(&data);
        file
    }

    fn sound_file(name: &str) -> String {
        let path =
            std::env::temp_dir().join(format!("sound-input-{}-{name}.wav", std::process::id()));
        std::fs::write(&path, wav(16_000, 1)).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn a_stored_file_a_path_and_a_resampled_sound_all_load() {
        let file = sound_file("ok");
        let stored = load_sound_input(
            &json!({ "audio": { "blob": "sha256:a", "path": "blobs/a.wav", "file": file } }),
        )
        .unwrap();
        assert_eq!((stored.sample_rate, stored.samples.len()), (16_000, 16_000));
        assert_eq!(load_sound_input(&json!({ "audio": file })).unwrap(), stored);

        let slower = load_sound_input(&json!({ "audio": file, "sampleRate": 8_000 })).unwrap();
        assert_eq!((slower.sample_rate, slower.samples.len()), (8_000, 8_000));
        assert_eq!(slower.duration_ms(), 1_000);
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn each_kind_of_mistake_has_its_own_status_code() {
        let code = |params: Value| load_sound_input(&params).unwrap_err().code;
        assert_eq!(code(json!({})), 400);
        assert_eq!(code(json!({ "audio": 5 })), 400);
        assert_eq!(code(json!({ "audio": {} })), 400);
        assert_eq!(code(json!({ "audio": "" })), 400);
        let file = sound_file("bad-rate");
        assert_eq!(code(json!({ "audio": file, "sampleRate": 100 })), 400);
        assert_eq!(code(json!({ "audio": file, "sampleRate": "fast" })), 400);
        assert_eq!(code(json!({ "audio": "no-such-file.mp3" })), 404);
        let text =
            std::env::temp_dir().join(format!("sound-input-{}-words.txt", std::process::id()));
        std::fs::write(&text, "not a sound, only words").unwrap();
        assert_eq!(code(json!({ "audio": text.to_string_lossy() })), 422);
        let _ = std::fs::remove_file(file);
        let _ = std::fs::remove_file(text);
    }
}
