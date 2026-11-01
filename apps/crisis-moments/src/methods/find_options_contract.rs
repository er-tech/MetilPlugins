/// The settings of the `find` action, after the defaults are filled in.
#[derive(Debug, Clone, PartialEq)]
pub struct FindOptions {
    /// Words or phrases that flag a sentence. Whole words, ignoring case.
    pub keywords: Vec<String>,
    /// Parts of sound names that flag a found sound, ignoring case.
    pub sound_names: Vec<String>,
    /// How many dB above the usual level a window must be to count as sudden loud sound.
    pub loudness_jump_db: f64,
    /// Hints at most this far apart in time make one moment.
    pub merge_gap_ms: u64,
    /// Moments shorter than this are dropped.
    pub min_ms: u64,
    /// Moments less sure than this, from 0 to 1, are dropped.
    pub threshold: f64,
}

/// The words that flag a sentence when the user gives no list.
pub const DEFAULT_KEYWORDS: [&str; 13] = [
    "help",
    "emergency",
    "police",
    "ambulance",
    "fire",
    "hurt",
    "bleeding",
    "dying",
    "let go",
    "stop it",
    "gun",
    "knife",
    "don't hurt",
];

/// The parts of sound names that flag a found sound when the user gives no list.
pub const DEFAULT_SOUND_NAMES: [&str; 13] = [
    "scream",
    "shout",
    "yell",
    "cry",
    "sob",
    "siren",
    "alarm",
    "glass",
    "shatter",
    "explosion",
    "gunshot",
    "gunfire",
    "bang",
];
