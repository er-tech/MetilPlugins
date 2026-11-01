use std::path::Path;

use super::encoder_settings_contract::EncoderSettings;

/// The arguments of FFmpeg for a raw `rgb24` stream on its standard input and an H.264 file at
/// `output`: scaled to the video size with Lanczos, yuv420p so that players accept it.
pub fn ffmpeg_arguments(settings: &EncoderSettings, output: &Path) -> Vec<String> {
    let text = |value: &str| value.to_string();
    vec![
        text("-y"),
        text("-hide_banner"),
        text("-loglevel"),
        text("error"),
        text("-f"),
        text("rawvideo"),
        text("-pix_fmt"),
        text("rgb24"),
        text("-s"),
        format!("{}x{}", settings.in_width, settings.in_height),
        text("-framerate"),
        format!("{:.6}", settings.frame_rate),
        text("-i"),
        text("-"),
        text("-vf"),
        format!(
            "scale={}:{}:flags=lanczos",
            settings.out_width, settings.out_height
        ),
        text("-c:v"),
        text("libx264"),
        text("-preset"),
        text("medium"),
        text("-crf"),
        text("18"),
        text("-pix_fmt"),
        text("yuv420p"),
        text("-movflags"),
        text("+faststart"),
        output.to_string_lossy().into_owned(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stream_size_the_scale_and_the_output_are_in_the_arguments() {
        let settings = EncoderSettings {
            in_width: 640,
            in_height: 360,
            out_width: 1280,
            out_height: 720,
            frame_rate: 25.0,
        };
        let arguments = ffmpeg_arguments(&settings, Path::new("out.mp4"));
        assert!(arguments.contains(&"640x360".to_string()));
        assert!(arguments.contains(&"scale=1280:720:flags=lanczos".to_string()));
        assert_eq!(arguments.last().unwrap(), "out.mp4");
        let input = arguments
            .iter()
            .position(|argument| argument == "-i")
            .unwrap();
        assert_eq!(arguments[input + 1], "-");
    }
}
