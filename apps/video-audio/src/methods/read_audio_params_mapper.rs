use std::path::PathBuf;

use metilplugins_plugin_sdk::{text, whole_number, Failure};
use serde_json::Value;

use crate::ffmpeg::{AudioFormat, AudioInput, AudioOutput};

/// The file `media` names: a path in text, or a stored file or a video as `media.open` gives it (an
/// object with an absolute `file`). A missing param is a 400; a file that is not there is a 404.
pub fn read_media_file(params: &Value) -> Result<PathBuf, Failure> {
    let path = match params.get("media") {
        Some(Value::String(path)) if !path.is_empty() => PathBuf::from(path),
        Some(Value::Object(object)) => match object.get("file") {
            Some(Value::String(file)) => PathBuf::from(file),
            _ => {
                return Err(Failure::status(
                    400,
                    "media must be a stored file (an object with file) or a path in text",
                ))
            }
        },
        None | Some(Value::Null) => {
            return Err(Failure::status(
                400,
                "media is missing: give a video or a sound",
            ))
        }
        Some(other) => {
            return Err(Failure::status(
                400,
                format!("media must be a stored file or a path in text, not {other}"),
            ))
        }
    };
    if path.is_file() {
        Ok(path)
    } else {
        Err(Failure::status(
            404,
            format!("{}: the file is not there", path.display()),
        ))
    }
}

/// The part of the file to read and which audio track, from `startMs`, `endMs` (the end must come
/// after the start) and `track` (0 to 31, default 0).
pub fn read_input<'a>(
    params: &Value,
    file: &'a std::path::Path,
) -> Result<AudioInput<'a>, Failure> {
    let start_ms = optional_whole(params, "startMs")?;
    let end_ms = optional_whole(params, "endMs")?;
    if let (Some(start), Some(end)) = (start_ms, end_ms) {
        if end <= start {
            return Err(Failure::status(
                400,
                format!("endMs ({end}) must be after startMs ({start})"),
            ));
        }
    }
    Ok(AudioInput {
        file,
        track: whole_number(params, "track", 0, 0, 31)? as u32,
        start_ms,
        end_ms,
    })
}

/// What to write: `format` (default `wav`), `sampleRate` (8000 to 192000) and `channels` (1 or 2).
/// `default_rate` is the sample rate when none is given (none: the source's).
pub fn read_output(params: &Value, default_rate: Option<u32>) -> Result<AudioOutput, Failure> {
    let name = text(params, "format", Some("wav"), 10)?.unwrap_or_default();
    let format = AudioFormat::parse(&name).ok_or_else(|| {
        Failure::status(
            400,
            format!("format must be wav, flac, opus or mp3, not {name:?}"),
        )
    })?;
    let sample_rate = match optional_whole(params, "sampleRate")? {
        Some(rate) if (8_000..=192_000).contains(&rate) => Some(rate as u32),
        Some(rate) => {
            return Err(Failure::status(
                400,
                format!("sampleRate must be from 8000 to 192000, not {rate}"),
            ))
        }
        None => default_rate,
    };
    let channels = match optional_whole(params, "channels")? {
        Some(count @ (1 | 2)) => Some(count as u32),
        Some(count) => {
            return Err(Failure::status(
                400,
                format!("channels must be 1 or 2, not {count}"),
            ))
        }
        None => None,
    };
    Ok(AudioOutput {
        format,
        sample_rate,
        channels,
    })
}

/// A whole number param that may be left out; a text or a fraction is a 400.
fn optional_whole(params: &Value, key: &str) -> Result<Option<u64>, Failure> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number.as_u64().map(Some).ok_or_else(|| {
            Failure::status(
                400,
                format!("{key} must be a whole number of at least 0, not {number}"),
            )
        }),
        Some(other) => Err(Failure::status(
            400,
            format!("{key} must be a whole number, not {other}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_defaults_are_a_wav_of_the_whole_first_track_in_the_rate_and_channels_of_the_source() {
        let output = read_output(&json!({}), None).unwrap();
        assert_eq!(output.format, AudioFormat::Wav);
        assert_eq!((output.sample_rate, output.channels), (None, None));
        let file = std::path::Path::new("a.mp4");
        let input = read_input(&json!({}), file).unwrap();
        assert_eq!((input.track, input.start_ms, input.end_ms), (0, None, None));
        assert_eq!(
            read_output(&json!({}), Some(48_000)).unwrap().sample_rate,
            Some(48_000)
        );
    }

    #[test]
    fn what_is_asked_for_is_read_and_a_wrong_param_is_a_400() {
        let params = json!({ "format": "opus", "sampleRate": 16000, "channels": 1 });
        let output = read_output(&params, None).unwrap();
        assert_eq!(
            (output.format, output.sample_rate, output.channels),
            (AudioFormat::Opus, Some(16_000), Some(1))
        );
        for bad in [
            json!({ "format": "aac" }),
            json!({ "sampleRate": 100 }),
            json!({ "sampleRate": "high" }),
            json!({ "channels": 6 }),
            json!({ "channels": 1.5 }),
        ] {
            let failure = read_output(&bad, None).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)), "{bad}");
        }
        let file = std::path::Path::new("a.mp4");
        for bad in [
            json!({ "startMs": 5000, "endMs": 5000 }),
            json!({ "startMs": -1 }),
            json!({ "track": 32 }),
        ] {
            assert_eq!(
                read_input(&bad, file).unwrap_err().code,
                Some(json!(400)),
                "{bad}"
            );
        }
    }

    #[test]
    fn media_is_a_path_or_an_object_with_a_file_that_is_there() {
        let missing = std::env::temp_dir().join("video-audio-no-such-file.mp4");
        assert_eq!(
            read_media_file(&json!({})).unwrap_err().code,
            Some(json!(400))
        );
        assert_eq!(
            read_media_file(&json!({ "media": 4 })).unwrap_err().code,
            Some(json!(400))
        );
        assert_eq!(
            read_media_file(&json!({ "media": { "name": "x" } }))
                .unwrap_err()
                .code,
            Some(json!(400))
        );
        for media in [json!(missing), json!({ "file": missing })] {
            assert_eq!(
                read_media_file(&json!({ "media": media }))
                    .unwrap_err()
                    .code,
                Some(json!(404))
            );
        }
        let here = std::env::current_exe().unwrap();
        assert_eq!(read_media_file(&json!({ "media": here })).unwrap(), here);
    }
}
