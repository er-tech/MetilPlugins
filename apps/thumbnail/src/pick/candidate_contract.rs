use serde_json::{json, Value};

/// A picture of the video and how good it would be as a thumbnail.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    /// When the picture is shown, in milliseconds from the start of the source.
    pub time_ms: u64,
    /// 0 to 1: the sharpness and the exposure together.
    pub score: f64,
    /// How much detail the picture has (the variance of its edges): a blur or a fade has little.
    pub sharpness: f64,
    /// 0 (black) to 1 (white).
    pub brightness: f64,
}

impl Candidate {
    /// What the action gives for a pick; the numbers are rounded to three decimals.
    pub fn to_value(self) -> Value {
        let round = |number: f64| (number * 1000.0).round() / 1000.0;
        json!({
            "timeMs": self.time_ms,
            "score": round(self.score),
            "sharpness": round(self.sharpness),
            "brightness": round(self.brightness),
        })
    }
}
