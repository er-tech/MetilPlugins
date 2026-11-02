use serde_json::Value;

/// What the first pass of FFmpeg's `loudnorm` found in a sound.
#[derive(Debug, Clone, PartialEq)]
pub struct Loudness {
    /// The integrated loudness, in LUFS.
    pub input_lufs: f64,
    /// The highest peak, in dB.
    pub input_true_peak_db: f64,
    /// The loudness range, in LU.
    pub input_range: f64,
    /// The gate `loudnorm` found, in LUFS.
    pub input_threshold: f64,
    /// The offset `loudnorm` suggests, in LU.
    pub target_offset: f64,
}

/// The `loudnorm` filter with the targets: integrated loudness in LUFS, true peak in dB and
/// loudness range in LU. With `measured` it is the second pass, which changes the sound by a
/// straight gain; without, the first pass, which only prints what it found as JSON.
pub fn loudnorm_filter(
    target_lufs: f64,
    true_peak_db: f64,
    range: f64,
    measured: Option<&Loudness>,
) -> String {
    let targets = format!("loudnorm=I={target_lufs}:TP={true_peak_db}:LRA={range}");
    match measured {
        None => format!("{targets}:print_format=json"),
        Some(found) => format!(
            "{targets}:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true",
            found.input_lufs,
            found.input_true_peak_db,
            found.input_range,
            found.input_threshold,
            found.target_offset
        ),
    }
}

/// Reads what the first pass printed: the last JSON object in FFmpeg's output. A sound with no
/// audio, or a sound so quiet that the filter prints `-inf`, is an error that says so.
pub fn parse_loudness(output: &str) -> Result<Loudness, String> {
    let end = output
        .rfind('}')
        .ok_or("FFmpeg printed no loudness: the file may have no audio")?;
    let start = output[..end]
        .rfind('{')
        .ok_or("FFmpeg printed no loudness: the file may have no audio")?;
    let found: Value = serde_json::from_str(&output[start..=end])
        .map_err(|error| format!("the loudness FFmpeg printed cannot be read: {error}"))?;
    let read = |key: &str| -> Result<f64, String> {
        let number: f64 = found
            .get(key)
            .and_then(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| value.as_f64().map(|n| n.to_string()))
            })
            .and_then(|text| text.trim().parse().ok())
            .ok_or_else(|| format!("FFmpeg printed no {key}"))?;
        if number.is_finite() {
            Ok(number)
        } else {
            Err("the sound is silent, so its loudness cannot be changed".to_string())
        }
    };
    Ok(Loudness {
        input_lufs: read("input_i")?,
        input_true_peak_db: read("input_tp")?,
        input_range: read("input_lra")?,
        input_threshold: read("input_thresh")?,
        target_offset: read("target_offset")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUTPUT: &str = r#"Input #0, mov,mp4 ...
[Parsed_loudnorm_0 @ 0x55]
{
	"input_i" : "-23.54",
	"input_tp" : "-4.21",
	"input_lra" : "7.80",
	"input_thresh" : "-34.17",
	"output_i" : "-16.45",
	"output_tp" : "-1.50",
	"output_lra" : "5.60",
	"output_thresh" : "-26.80",
	"normalization_type" : "dynamic",
	"target_offset" : "0.45"
}
"#;

    #[test]
    fn the_last_json_block_of_the_output_is_the_finding() {
        let found = parse_loudness(OUTPUT).unwrap();
        assert_eq!(
            found,
            Loudness {
                input_lufs: -23.54,
                input_true_peak_db: -4.21,
                input_range: 7.8,
                input_threshold: -34.17,
                target_offset: 0.45
            }
        );
    }

    #[test]
    fn no_json_and_a_silent_sound_are_errors_that_say_what_is_wrong() {
        assert!(parse_loudness("nothing here")
            .unwrap_err()
            .contains("no audio"));
        let silent = OUTPUT.replace("-23.54", "-inf");
        assert!(parse_loudness(&silent).unwrap_err().contains("silent"));
    }

    #[test]
    fn the_first_pass_prints_json_and_the_second_carries_what_was_measured() {
        let first = loudnorm_filter(-16.0, -1.5, 11.0, None);
        assert_eq!(first, "loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json");
        let second = loudnorm_filter(-16.0, -1.5, 11.0, Some(&parse_loudness(OUTPUT).unwrap()));
        assert!(
            second.contains("measured_I=-23.54:measured_TP=-4.21"),
            "{second}"
        );
        assert!(second.ends_with("offset=0.45:linear=true"), "{second}");
    }
}
