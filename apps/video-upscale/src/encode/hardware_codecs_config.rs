use super::video_codec_contract::VideoCodec;

/// The hardware H.264 encoders to try on `os` (a value of `std::env::consts::OS`), best first. Only
/// `h264_amf` was measured (RX 7900 XT: same quality as `libx264` medium crf 23 at 3 to 4 times the
/// speed, with 98% less processor time and a file 60% larger, see the research page on the media
/// pipeline in the Loomas repository). The others use settings that are the usual equivalent and
/// were not measured.
pub fn hardware_codecs(os: &str) -> Vec<VideoCodec> {
    let nvenc = VideoCodec::new(
        "h264_nvenc",
        &["-preset", "p5", "-rc", "constqp", "-qp", "22"],
    );
    let amf = VideoCodec::new(
        "h264_amf",
        &[
            "-quality", "balanced", "-rc", "cqp", "-qp_i", "24", "-qp_p", "24", "-qp_b", "24",
        ],
    );
    let qsv = VideoCodec::new("h264_qsv", &["-preset", "medium", "-global_quality", "22"]);
    let videotoolbox = VideoCodec::new("h264_videotoolbox", &["-q:v", "65"]);
    match os {
        "windows" | "linux" => vec![nvenc, amf, qsv],
        "macos" => vec![videotoolbox],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_system_has_its_own_encoders_and_an_unknown_one_has_none() {
        let names =
            |os| -> Vec<&'static str> { hardware_codecs(os).iter().map(|c| c.name).collect() };
        assert_eq!(names("windows"), ["h264_nvenc", "h264_amf", "h264_qsv"]);
        assert_eq!(names("macos"), ["h264_videotoolbox"]);
        assert!(names("plan9").is_empty());
    }
}
