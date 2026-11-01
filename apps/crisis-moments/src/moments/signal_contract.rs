/// Which input a signal came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    /// The sentences said.
    Text,
    /// The sounds or things found.
    Labels,
    /// The loudness over time.
    Volume,
}

impl Source {
    /// The name written in the result.
    pub fn name(self) -> &'static str {
        match self {
            Source::Text => "text",
            Source::Labels => "labels",
            Source::Volume => "volume",
        }
    }
}

/// One hint from one input that something may be wrong at a time.
#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    /// Start in milliseconds.
    pub start_ms: u64,
    /// End in milliseconds.
    pub end_ms: u64,
    /// The input that gave this hint.
    pub source: Source,
    /// Why it was flagged, in short words such as "shouting".
    pub reason: String,
    /// How strong the hint is, from 0 to 1.
    pub weight: f64,
    /// What was seen, such as the matched word or the sound name.
    pub detail: String,
}

/// Hints that overlap in time, joined into one moment to review.
#[derive(Debug, Clone, PartialEq)]
pub struct Moment {
    /// Start in milliseconds.
    pub start_ms: u64,
    /// End in milliseconds.
    pub end_ms: u64,
    /// The reason of the strongest hint.
    pub reason: String,
    /// How sure the rules are, from 0 to 1. More agreeing inputs give more.
    pub confidence: f64,
    /// The hints behind the moment: words and sounds before loudness, then the strongest first.
    pub signals: Vec<Signal>,
}
