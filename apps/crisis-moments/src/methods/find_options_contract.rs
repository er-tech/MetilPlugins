/// The settings of the `find` action, after the defaults are filled in.
#[derive(Debug, Clone, PartialEq)]
pub struct FindOptions {
    /// Words or phrases of distress: a cry for help, not being able to breathe, fear, "stop".
    pub distress_words: Vec<String>,
    /// Words or phrases about ending one's own life or hurting oneself.
    pub self_harm_words: Vec<String>,
    /// Threats and insults.
    pub aggression_words: Vec<String>,
    /// Parts of sound names that flag a found sound, ignoring case.
    pub sound_names: Vec<String>,
    /// Words of one kind said within this many milliseconds of the first one count together.
    pub words_window_ms: u64,
    /// How many dB above the usual level a window must be to count as a loud voice.
    pub loudness_jump_db: f64,
    /// Hints at most this far apart in time make one moment.
    pub merge_gap_ms: u64,
    /// Moments shorter than this are dropped.
    pub min_ms: u64,
    /// Moments less sure than this, from 0 to 1, are dropped.
    pub threshold: f64,
}

/// The parts of sound names that flag a found sound when the user gives no list: the sounds of a
/// person screaming, shouting, crying, struggling to breathe, or being hit.
pub const DEFAULT_SOUND_NAMES: [&str; 21] = [
    "scream", "shout", "yell", "bellow", "cry", "sob", "whimper", "wail", "moan", "gasp", "pant",
    "wheeze", "slap", "smack", "thump", "thud", "whack", "thwack", "slam", "grunt", "groan",
];
