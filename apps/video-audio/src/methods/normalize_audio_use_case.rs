use loomasplugins_plugin_sdk::{number, Cancel, Failure};
use loomasplugins_work_files::unique_work_file_name;
use serde_json::{json, Value};

use super::extract_audio_use_case::describe;
use super::read_audio_params_mapper::{read_input, read_media_file, read_output};
use crate::ffmpeg::{
    extract_arguments, ffmpeg_program, loudnorm_filter, measure_arguments, parse_loudness,
    run_ffmpeg,
};

/// The sample rate of a normalised file when none is asked for: the loudness filter would write
/// 192 kHz without it.
const DEFAULT_RATE: u32 = 48_000;

/// The `normalize` action: the sound of a video, or a stretch of it, at a given loudness, so
/// videos can be compared and heard at one level.
///
/// Params: those of `extract` (`sampleRate` defaults to 48000 here), and `targetLufs` (-70 to -5,
/// default -16: the loudness of streaming and podcasts), `truePeakDb` (-9 to 0, default -1.5) and
/// `loudnessRange` (1 to 20, default 11). It measures the sound first and then changes it by a
/// straight gain, so the dynamics stay. It gives what `extract` gives and `targetLufs`,
/// `inputLufs`, `inputTruePeakDb` and `inputLoudnessRange`: how loud the source was.
pub fn normalize_audio(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let file = read_media_file(params)?;
    let input = read_input(params, &file)?;
    let settings = read_output(params, Some(DEFAULT_RATE))?;
    let target = number(params, "targetLufs", -16.0, -70.0, -5.0)?;
    let peak = number(params, "truePeakDb", -1.5, -9.0, 0.0)?;
    let range = number(params, "loudnessRange", 11.0, 1.0, 20.0)?;
    let work_dir = cancel
        .work_dir()
        .ok_or_else(|| Failure::status(500, "Loomas gave no work folder to write the audio in"))?;
    let program = ffmpeg_program(std::env::var_os("LOOMAS_FFMPEG"));

    cancel.progress("measuring the loudness");
    let measure = measure_arguments(&input, &loudnorm_filter(target, peak, range, None));
    let log = run_ffmpeg(&program, &measure, cancel)?;
    let found = parse_loudness(&log).map_err(|message| Failure::status(422, message))?;

    cancel.progress("changing the loudness");
    let output = work_dir.join(unique_work_file_name("video-audio", settings.format.name()));
    let filter = loudnorm_filter(target, peak, range, Some(&found));
    run_ffmpeg(
        &program,
        &extract_arguments(&input, Some(&filter), &settings, &output),
        cancel,
    )?;

    let mut answer = describe(&output, &settings, input.track)?;
    answer["targetLufs"] = json!(target);
    answer["inputLufs"] = json!(found.input_lufs);
    answer["inputTruePeakDb"] = json!(found.input_true_peak_db);
    answer["inputLoudnessRange"] = json!(found.input_range);
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_ffmpeg() -> bool {
        std::process::Command::new(ffmpeg_program(std::env::var_os("LOOMAS_FFMPEG")))
            .arg("-version")
            .output()
            .is_ok()
    }

    #[test]
    fn a_quiet_tone_is_measured_and_written_at_the_target() {
        if !has_ffmpeg() {
            return;
        }
        let dir = std::env::temp_dir().join(format!("video-audio-norm-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("noise.wav");
        let made = std::process::Command::new(ffmpeg_program(std::env::var_os("LOOMAS_FFMPEG")))
            .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y"])
            .args([
                "-f",
                "lavfi",
                "-i",
                "anoisesrc=duration=6:amplitude=0.05:seed=7",
            ])
            .arg(&source)
            .status()
            .unwrap();
        assert!(made.success());
        let answer = normalize_audio(
            &json!({ "media": source, "targetLufs": -20, "format": "wav" }),
            &Cancel::with_work_dir(dir.clone()),
        )
        .unwrap();
        let before = answer["inputLufs"].as_f64().unwrap();
        assert!(before < -26.0, "a quiet source, measured at {before}");
        assert_eq!(answer["targetLufs"], json!(-20.0));
        assert_eq!(answer["sampleRate"], json!(48_000));

        // The written file measures close to the target.
        let written = answer["audio"]["workFile"].as_str().unwrap();
        let log = run_ffmpeg(
            &ffmpeg_program(std::env::var_os("LOOMAS_FFMPEG")),
            &measure_arguments(
                &crate::ffmpeg::AudioInput {
                    file: std::path::Path::new(written),
                    track: 0,
                    start_ms: None,
                    end_ms: None,
                },
                &loudnorm_filter(-20.0, -1.5, 11.0, None),
            ),
            &Cancel::default(),
        )
        .unwrap();
        let after = parse_loudness(&log).unwrap().input_lufs;
        assert!(
            (after + 20.0).abs() < 1.5,
            "measured {after} after normalising to -20"
        );
    }

    #[test]
    fn a_target_out_of_range_is_a_400() {
        let here = std::env::current_exe().unwrap();
        for bad in [
            json!({ "targetLufs": 3 }),
            json!({ "truePeakDb": 2 }),
            json!({ "loudnessRange": 0 }),
        ] {
            let mut params = bad;
            params["media"] = json!(here);
            let failure = normalize_audio(&params, &Cancel::default()).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)), "{params}");
        }
    }
}
