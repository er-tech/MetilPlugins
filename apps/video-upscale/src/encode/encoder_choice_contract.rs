/// Which encoder the user asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EncoderChoice {
    /// `libx264` on the processor: the smallest file.
    #[default]
    X264,
    /// The encoder of the graphics card, when the computer has one that works, else `libx264`.
    Hardware,
}

impl EncoderChoice {
    /// Reads `x264` or `hardware`, in any case. `None` for anything else.
    pub fn parse(text: &str) -> Option<EncoderChoice> {
        match text.trim().to_lowercase().as_str() {
            "x264" => Some(EncoderChoice::X264),
            "hardware" => Some(EncoderChoice::Hardware),
            _ => None,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            EncoderChoice::X264 => "x264",
            EncoderChoice::Hardware => "hardware",
        }
    }
}
