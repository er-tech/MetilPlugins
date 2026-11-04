use serde_json::{json, Value};
use loomasplugins_plugin_sdk::{flag, text, whole_number, Failure};
use loomasplugins_sound_reader::load_sound_input;

use crate::waveform::{compute_columns, draw_waveform_svg, read_regions, SvgOptions};

/// The `waveform` action: the waves of a sound as an SVG picture.
///
/// Params: `audio`, `width` (100 to 10000 pixels, default 1200), `height` (40 to 2000, default
/// 200), `color` (default `#2b6cb0`), `background` (default `#ffffff`, or `none`), `normalize`
/// (stretch the waves to the edge, default false), `showTime` (the time axis, default true),
/// `title`, `regions` (stretches to mark: `{ startMs, endMs, label?, color? }`, and the sentences
/// of a transcript work as they are) and `sampleRate`.
///
/// It gives `{ svg, width, height, durationMs, sampleRate }`.
pub fn draw_waveform(params: &Value) -> Result<Value, Failure> {
    let width = whole_number(params, "width", 1_200, 100, 10_000)? as usize;
    let height = whole_number(params, "height", 200, 40, 2_000)? as usize;
    let options = SvgOptions {
        width,
        height,
        color: text(params, "color", Some("#2b6cb0"), 64)?.unwrap_or_default(),
        background: text(params, "background", Some("#ffffff"), 64)?.unwrap_or_default(),
        show_time: flag(params, "showTime", true)?,
        title: text(params, "title", None, 200)?,
        normalize: flag(params, "normalize", false)?,
    };
    let regions = read_regions(params)?;
    let sound =
        load_sound_input(params).map_err(|error| Failure::status(error.code, error.message))?;

    let columns = compute_columns(&sound.samples, width);
    let svg = draw_waveform_svg(&columns, sound.duration_ms(), &regions, &options);
    Ok(json!({
        "svg": svg,
        "width": width,
        "height": height,
        "durationMs": sound.duration_ms(),
        "sampleRate": sound.sample_rate,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wav file of `seconds` with a tone in the first half and silence in the second.
    fn half_tone(name: &str, seconds: u32) -> String {
        let rate = 8_000_u32;
        let samples: Vec<i16> = (0..rate * seconds)
            .map(|n| {
                if n < rate * seconds / 2 {
                    (0.6 * (2.0 * std::f32::consts::PI * 300.0 * n as f32 / rate as f32).sin()
                        * 32_767.0) as i16
                } else {
                    0
                }
            })
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
        let path =
            std::env::temp_dir().join(format!("sound-graph-{}-{name}.wav", std::process::id()));
        std::fs::write(&path, file).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn draws_the_waves_of_a_sound_with_regions_and_an_axis() {
        let file = half_tone("draw", 4);
        let result = draw_waveform(&json!({
            "audio": { "file": file },
            "width": 400,
            "height": 120,
            "title": "Take 1",
            "regions": [{ "startMs": 0, "endMs": 1_500, "text": "Hello & welcome." }]
        }))
        .unwrap();
        assert_eq!(
            (result["width"].as_u64(), result["height"].as_u64()),
            (Some(400), Some(120))
        );
        assert_eq!(result["durationMs"], 4_000);
        assert_eq!(result["sampleRate"], 8_000);
        let svg = result["svg"].as_str().unwrap();
        assert!(svg.starts_with("<svg ") && svg.trim_end().ends_with("</svg>"));
        assert!(
            svg.contains("viewBox=\"0 0 400 120\"")
                && svg.contains("<title id=\"title\">Take 1</title>")
        );
        assert!(svg.contains("Hello &amp; welcome.</text>"), "{svg}");
        assert!(
            svg.contains(">0:00</text>") && svg.contains(">0:04</text>")
                || svg.contains(">0:03</text>"),
            "{svg}"
        );
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn the_loud_half_has_tall_waves_and_the_silent_half_is_flat() {
        let file = half_tone("shape", 2);
        let result = draw_waveform(
            &json!({ "audio": file, "width": 200, "height": 120, "showTime": false }),
        )
        .unwrap();
        let svg = result["svg"].as_str().unwrap();
        let path = svg
            .split("<path d=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let ys: Vec<f32> = path
            .trim_end_matches('Z')
            .split_whitespace()
            .map(|point| point[1..].split_once(',').unwrap().1.parse().unwrap())
            .collect();
        // 200 points along the top; the plot is 120 high, so the middle is 60.
        assert!(
            ys[..90].iter().any(|y| *y < 40.0),
            "the loud part should rise well above the middle"
        );
        assert!(
            ys[110..200].iter().all(|y| (y - 60.0).abs() < 0.2),
            "the silent part should lie on the middle"
        );
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn wrong_params_are_a_400_and_a_file_that_is_not_there_is_a_404() {
        let file = half_tone("bad", 1);
        let code = |params: Value| draw_waveform(&params).unwrap_err().code;
        assert_eq!(
            code(json!({ "audio": file, "width": 10 })),
            Some(json!(400))
        );
        assert_eq!(
            code(json!({ "audio": file, "height": 5000 })),
            Some(json!(400))
        );
        assert_eq!(
            code(json!({ "audio": file, "normalize": "yes" })),
            Some(json!(400))
        );
        assert_eq!(
            code(json!({ "audio": file, "regions": [{ "startMs": 5, "endMs": 1 }] })),
            Some(json!(400))
        );
        assert_eq!(code(json!({ "audio": file, "color": 3 })), Some(json!(400)));
        assert_eq!(
            code(json!({ "audio": "no-such-file.mp3" })),
            Some(json!(404))
        );
        let _ = std::fs::remove_file(file);
    }
}
