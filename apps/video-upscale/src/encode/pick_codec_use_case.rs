use super::encoder_choice_contract::EncoderChoice;
use super::video_codec_contract::VideoCodec;

/// The encoder to use, and what the result says about the choice.
#[derive(Debug, Clone, PartialEq)]
pub struct CodecPick {
    pub codec: VideoCodec,
    /// Why the encoder asked for was not used, when that is so.
    pub fallback: Option<String>,
    pub requested: EncoderChoice,
}

impl CodecPick {
    /// The `encoder` field of a result: `{ used, requested, fallback }`.
    pub fn to_value(&self) -> serde_json::Value {
        serde_json::json!({
            "used": self.codec.name,
            "requested": self.requested.word(),
            "fallback": self.fallback,
        })
    }
}

/// Chooses the encoder. `x264` is `libx264`. `hardware` is the first of `candidates` that
/// `works` says really starts here, and `libx264` with the reason when none does, or when the
/// graphics card was turned off for the run (`card_allowed` false).
pub fn pick_codec(
    choice: EncoderChoice,
    card_allowed: bool,
    candidates: Vec<VideoCodec>,
    works: impl Fn(&VideoCodec) -> bool,
) -> CodecPick {
    let pick = |codec, fallback| CodecPick {
        codec,
        fallback,
        requested: choice,
    };
    if choice == EncoderChoice::X264 {
        return pick(VideoCodec::x264(), None);
    }
    if !card_allowed {
        return pick(
            VideoCodec::x264(),
            Some(
                "the graphics card is turned off for this run (device cpu); libx264 is used".into(),
            ),
        );
    }
    match candidates.into_iter().find(|codec| works(codec)) {
        Some(codec) => pick(codec, None),
        None => pick(
            VideoCodec::x264(),
            Some("no graphics card encoder starts on this computer; libx264 is used".into()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates() -> Vec<VideoCodec> {
        vec![
            VideoCodec::new("one", &[]),
            VideoCodec::new("two", &["-x", "1"]),
        ]
    }

    #[test]
    fn x264_is_the_default_and_asks_nothing_of_the_computer() {
        let pick = pick_codec(EncoderChoice::X264, true, candidates(), |_| {
            panic!("not asked")
        });
        assert_eq!((pick.codec.name, pick.fallback), ("libx264", None));
    }

    #[test]
    fn hardware_takes_the_first_encoder_that_works() {
        let pick = pick_codec(EncoderChoice::Hardware, true, candidates(), |codec| {
            codec.name == "two"
        });
        assert_eq!(pick.codec.name, "two");
        assert_eq!(pick.to_value()["requested"], "hardware");
        assert!(pick.fallback.is_none());
    }

    #[test]
    fn no_working_encoder_or_a_card_turned_off_is_libx264_with_the_reason() {
        let none = pick_codec(EncoderChoice::Hardware, true, candidates(), |_| false);
        assert_eq!(none.codec.name, "libx264");
        assert!(none.fallback.unwrap().contains("no graphics card encoder"));
        let off = pick_codec(EncoderChoice::Hardware, false, candidates(), |_| true);
        assert!(off.fallback.unwrap().contains("turned off"));
    }
}
