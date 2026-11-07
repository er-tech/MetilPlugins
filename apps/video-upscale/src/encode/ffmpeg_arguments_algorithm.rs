use std::path::Path;

use super::encoder_settings_contract::EncoderSettings;
use super::video_codec_contract::VideoCodec;

/// The arguments of FFmpeg for a raw `rgb24` stream on its standard input and an H.264 file at
/// `output`, written with `codec`: scaled to the video size with Lanczos, yuv420p so that players
/// accept it.
pub fn ffmpeg_arguments(
    settings: &EncoderSettings,
    codec: &VideoCodec,
    output: &Path,
) -> Vec<String> {
    let text = |value: &str| value.to_string();
    let mut arguments = vec![
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
    ];
    arguments.extend(codec.to_arguments());
    arguments.extend([
        text("-pix_fmt"),
        text("yuv420p"),
        text("-movflags"),
        text("+faststart"),
        output.to_string_lossy().into_owned(),
    ]);
    arguments
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
        let arguments = ffmpeg_arguments(&settings, &VideoCodec::x264(), Path::new("out.mp4"));
        assert!(arguments.contains(&"640x360".to_string()));
        assert!(arguments.contains(&"scale=1280:720:flags=lanczos".to_string()));
        assert_eq!(arguments.last().unwrap(), "out.mp4");
        let input = arguments
            .iter()
            .position(|argument| argument == "-i")
            .unwrap();
        assert_eq!(arguments[input + 1], "-");
    }

    #[test]
    fn the_codec_and_its_quality_settings_are_in_the_arguments() {
        let settings = EncoderSettings {
            in_width: 64,
            in_height: 64,
            out_width: 64,
            out_height: 64,
            frame_rate: 10.0,
        };
        let software = ffmpeg_arguments(&settings, &VideoCodec::x264(), Path::new("a.mp4"));
        let at = software.iter().position(|a| a == "libx264").unwrap();
        assert_eq!(
            &software[at - 1..at + 5],
            ["-c:v", "libx264", "-preset", "medium", "-crf", "18"]
        );
        let card = VideoCodec::new("h264_amf", &["-qp_i", "24"]);
        let hardware = ffmpeg_arguments(&settings, &card, Path::new("a.mp4"));
        assert!(
            hardware.contains(&"h264_amf".to_string())
                && !hardware.contains(&"libx264".to_string())
        );
    }
}
