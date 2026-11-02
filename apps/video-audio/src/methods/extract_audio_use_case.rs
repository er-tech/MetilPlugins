use std::path::Path;

use metilplugins_plugin_sdk::{Cancel, Failure};
use metilplugins_work_files::unique_work_file_name;
use serde_json::{json, Value};

use super::read_audio_params_mapper::{read_input, read_media_file, read_output};
use crate::ffmpeg::{extract_arguments, ffmpeg_program, run_ffmpeg};

/// The `extract` action: the sound of a video, or a stretch of it, as an audio file.
///
/// Params: `media` (a video or a sound), `format` (`wav`, `flac`, `opus` or `mp3`, default `wav`),
/// `sampleRate` (8000 to 192000) and `channels` (1 or 2) when the source's do not do (16 kHz mono
/// is what speech models read), `startMs` and `endMs` for a stretch, `track` (default 0) for a
/// video with more than one sound. It gives `{ audio, format, sampleRate?, channels?, track }`
/// where `audio` is the stored file.
pub fn extract_audio(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let file = read_media_file(params)?;
    let input = read_input(params, &file)?;
    let settings = read_output(params, None)?;
    let work_dir = cancel
        .work_dir()
        .ok_or_else(|| Failure::status(500, "Metil gave no work folder to write the audio in"))?;
    let output = work_dir.join(unique_work_file_name("video-audio", settings.format.name()));

    let program = ffmpeg_program(std::env::var_os("METIL_FFMPEG"));
    run_ffmpeg(
        &program,
        &extract_arguments(&input, None, &settings, &output),
        cancel,
    )?;
    describe(&output, &settings, input.track)
}

/// What an action answers with once the file is written: the stored file and how it was made.
pub(super) fn describe(
    output: &Path,
    settings: &crate::ffmpeg::AudioOutput,
    track: u32,
) -> Result<Value, Failure> {
    if !output.is_file() {
        return Err(Failure::status(
            422,
            "the media has no audio to write (FFmpeg made no file)",
        ));
    }
    Ok(json!({
        "audio": { "workFile": output.to_string_lossy(), "mime": settings.format.mime() },
        "format": settings.format.name(),
        "sampleRate": settings.sample_rate,
        "channels": settings.channels,
        "track": track,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether FFmpeg can be started here; the tests that need it say nothing when it cannot.
    fn has_ffmpeg() -> bool {
        std::process::Command::new(ffmpeg_program(std::env::var_os("METIL_FFMPEG")))
            .arg("-version")
            .output()
            .is_ok()
    }

    fn cancel_in(name: &str) -> (Cancel, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("video-audio-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        (Cancel::with_work_dir(dir.clone()), dir)
    }

    /// Makes a two second sound of a tone with FFmpeg itself, so no media file is kept in the tests.
    fn tone(dir: &Path) -> std::path::PathBuf {
        let file = dir.join("tone.wav");
        let status = std::process::Command::new(ffmpeg_program(std::env::var_os("METIL_FFMPEG")))
            .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y"])
            .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=2"])
            .arg(&file)
            .status()
            .unwrap();
        assert!(status.success());
        file
    }

    #[test]
    fn a_stretch_is_written_in_the_format_rate_and_channels_asked_for() {
        if !has_ffmpeg() {
            return;
        }
        let (cancel, dir) = cancel_in("extract");
        let source = tone(&dir);
        let answer = extract_audio(
            &json!({ "media": source, "format": "flac", "sampleRate": 16000, "channels": 1,
                     "startMs": 500, "endMs": 1500 }),
            &cancel,
        )
        .unwrap();
        assert_eq!(answer["audio"]["mime"], "audio/flac");
        assert_eq!(answer["format"], "flac");
        let written = answer["audio"]["workFile"].as_str().unwrap();
        assert!(written.ends_with(".flac"));
        assert!(std::fs::metadata(written).unwrap().len() > 0);
    }

    #[test]
    fn a_track_the_media_does_not_have_is_a_422_with_what_ffmpeg_said() {
        if !has_ffmpeg() {
            return;
        }
        let (cancel, dir) = cancel_in("track");
        let source = tone(&dir);
        let failure = extract_audio(&json!({ "media": source, "track": 3 }), &cancel).unwrap_err();
        assert_eq!(failure.code, Some(json!(422)), "{failure:?}");
    }

    #[test]
    fn without_a_work_folder_and_with_bad_params_it_fails_before_starting_anything() {
        let here = std::env::current_exe().unwrap();
        let failure = extract_audio(&json!({ "media": here }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(500)));
        let failure = extract_audio(
            &json!({ "media": here, "format": "aac" }),
            &Cancel::default(),
        )
        .unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
    }
}
