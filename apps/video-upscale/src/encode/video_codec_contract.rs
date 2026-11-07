/// A video encoder of FFmpeg with the arguments that set its quality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoCodec {
    /// The name FFmpeg knows it by, such as `libx264` or `h264_amf`.
    pub name: &'static str,
    /// What follows `-c:v <name>`: the quality settings.
    pub arguments: Vec<String>,
}

impl VideoCodec {
    /// The software encoder: the smallest files, and the default.
    pub fn x264() -> VideoCodec {
        VideoCodec::new("libx264", &["-preset", "medium", "-crf", "18"])
    }

    pub fn new(name: &'static str, arguments: &[&str]) -> VideoCodec {
        VideoCodec {
            name,
            arguments: arguments
                .iter()
                .map(|argument| argument.to_string())
                .collect(),
        }
    }

    /// `-c:v <name>` and the quality arguments, as FFmpeg takes them.
    pub fn to_arguments(&self) -> Vec<String> {
        let mut all = vec!["-c:v".to_string(), self.name.to_string()];
        all.extend(self.arguments.iter().cloned());
        all
    }
}
