/// One item of an input list, with the times every shared timed type has and the field its type adds.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedItem {
    /// Start in milliseconds.
    pub start_ms: u64,
    /// End in milliseconds.
    pub end_ms: u64,
    /// The words, for a `timedText` item.
    pub text: Option<String>,
    /// The name of the sound or thing, for a `timedLabel` item.
    pub label: Option<String>,
    /// How sure the finder was, from 0 to 1, for a `timedLabel` item.
    pub confidence: Option<f64>,
    /// The measured number (here a level in dB), for a `series` item.
    pub value: Option<f64>,
}
