use std::path::Path;

use super::audio_format_contract::AudioFormat;

/// What to write: the format and, when given, a sample rate and a number of channels (the source's
/// when not).
#[derive(Debug, Clone, Copy)]
pub struct AudioOutput {
    pub format: AudioFormat,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
}

/// A stretch of the source and the audio track in it.
#[derive(Debug, Clone, Copy)]
pub struct AudioInput<'a> {
    pub file: &'a Path,
    /// Which audio track, counting from 0.
    pub track: u32,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
}

/// The FFmpeg arguments that write the audio of `input` to `output`, one step. `filter` is an
/// audio filter to run on it (the second pass of a loudness change), or none.
pub fn extract_arguments(
    input: &AudioInput,
    filter: Option<&str>,
    settings: &AudioOutput,
    output: &Path,
) -> Vec<String> {
    let mut arguments = common(input);
    arguments.extend(["-vn".to_string()]);
    if let Some(filter) = filter {
        arguments.extend(["-af".to_string(), filter.to_string()]);
    }
    if let Some(rate) = settings.sample_rate {
        arguments.extend(["-ar".to_string(), rate.to_string()]);
    }
    if let Some(channels) = settings.channels {
        arguments.extend(["-ac".to_string(), channels.to_string()]);
    }
    arguments.extend(
        settings
            .format
            .encoder_arguments()
            .iter()
            .map(ToString::to_string),
    );
    arguments.push(output.to_string_lossy().into_owned());
    arguments
}

/// The FFmpeg arguments that measure the loudness of `input` with `filter` (the first pass of a
/// loudness change): nothing is written, the filter prints what it found.
pub fn measure_arguments(input: &AudioInput, filter: &str) -> Vec<String> {
    let mut arguments = common(input);
    arguments.extend(
        ["-vn", "-af", filter, "-f", "null", "-"]
            .iter()
            .map(ToString::to_string),
    );
    arguments
}

/// The options both start with: no questions, the stretch, the file and the track. The log level
/// stays at `info` because the loudness filter prints its findings there.
fn common(input: &AudioInput) -> Vec<String> {
    let mut arguments: Vec<String> = ["-nostdin", "-hide_banner", "-nostats", "-y"]
        .iter()
        .map(ToString::to_string)
        .collect();
    if let Some(start) = input.start_ms {
        arguments.extend(["-ss".to_string(), seconds(start)]);
    }
    if let Some(end) = input.end_ms {
        arguments.extend(["-to".to_string(), seconds(end)]);
    }
    arguments.extend([
        "-i".to_string(),
        input.file.to_string_lossy().into_owned(),
        "-map".to_string(),
        format!("0:a:{}", input.track),
    ]);
    arguments
}

/// Milliseconds as FFmpeg's seconds with three decimals.
fn seconds(ms: u64) -> String {
    format!("{}.{:03}", ms / 1000, ms % 1000)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn input(file: &Path) -> AudioInput<'_> {
        AudioInput {
            file,
            track: 1,
            start_ms: Some(1_500),
            end_ms: Some(61_005),
        }
    }

    #[test]
    fn a_stretch_a_track_a_rate_channels_and_an_encoder_make_the_arguments() {
        let file = PathBuf::from("in.mp4");
        let settings = AudioOutput {
            format: AudioFormat::Flac,
            sample_rate: Some(16_000),
            channels: Some(1),
        };
        let arguments = extract_arguments(&input(&file), None, &settings, Path::new("out.flac"));
        let text = arguments.join(" ");
        assert!(
            text.contains("-ss 1.500 -to 61.005 -i in.mp4 -map 0:a:1"),
            "{text}"
        );
        assert!(
            text.contains("-ar 16000 -ac 1 -c:a flac out.flac"),
            "{text}"
        );
        assert!(!text.contains("-af"), "{text}");
    }

    #[test]
    fn nothing_asked_for_leaves_the_rate_and_the_channels_to_the_source() {
        let file = PathBuf::from("in.mp4");
        let whole = AudioInput {
            file: &file,
            track: 0,
            start_ms: None,
            end_ms: None,
        };
        let settings = AudioOutput {
            format: AudioFormat::Wav,
            sample_rate: None,
            channels: None,
        };
        let text = extract_arguments(
            &whole,
            Some("loudnorm=I=-16"),
            &settings,
            Path::new("o.wav"),
        )
        .join(" ");
        assert!(
            !text.contains("-ss") && !text.contains("-ar") && !text.contains("-ac "),
            "{text}"
        );
        assert!(text.contains("-af loudnorm=I=-16"), "{text}");
        assert!(text.contains("-c:a pcm_s16le o.wav"), "{text}");
    }

    #[test]
    fn measuring_writes_nothing_and_keeps_the_filter_output_readable() {
        let file = PathBuf::from("in.mp4");
        let text = measure_arguments(&input(&file), "loudnorm=print_format=json").join(" ");
        assert!(
            text.ends_with("-af loudnorm=print_format=json -f null -"),
            "{text}"
        );
        assert!(!text.contains("-loglevel"), "{text}");
    }
}
