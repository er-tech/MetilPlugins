/// One thing found in a sample: which of the things looked for (its number in the plugin's list)
/// and how sure the plugin is, from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Presence {
    pub key: usize,
    pub score: f32,
}

/// What was found in one picture of the video, and when that picture is.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// The place of the picture among those the video gave.
    pub frame: usize,
    pub time_ms: u64,
    pub duration_ms: u64,
    pub present: Vec<Presence>,
}

/// A thing that shows up and stays: from the first sample it is in to the last of the same
/// stretch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    pub key: usize,
    /// The picture where it first shows, and its time in the source.
    pub first_frame: usize,
    pub start_ms: u64,
    /// When the last picture it is in stops being shown.
    pub end_ms: u64,
    /// The surest the plugin was of it in any sample of the stretch.
    pub confidence: f32,
    /// The most of it found together in one sample (how many boxes of the class, for a detector).
    pub peak_count: usize,
    /// In how many samples it was found.
    pub samples: usize,
}
