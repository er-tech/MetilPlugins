use metilplugins_onnx_device::DeviceChoice;

/// What a call to `find` asked for, checked and with the defaults filled in.
#[derive(Debug, Clone, PartialEq)]
pub struct FindOptions {
    /// Where the tagger runs.
    pub device: DeviceChoice,
    /// One flag for each of the 521 classes: whether to look for it.
    pub wanted: [bool; 521],
    /// How sure the tagger must be of a sound, from 0 to 1.
    pub min_confidence: f32,
    /// How long a sound can stop before it counts as gone.
    pub gap_ms: u64,
    /// In how few frames (about half a second apart) a sound is still worth reporting.
    pub min_frames: usize,
    /// Whether to give a clip of each event.
    pub clips: bool,
    /// How much sound before and after an event goes in its clip.
    pub clip_pad_ms: u64,
    /// The longest a clip can be; an event that is longer is cut short from its start.
    pub max_clip_ms: u64,
}
