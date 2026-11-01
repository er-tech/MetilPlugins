/// A sound that was heard: its class (the number in the class map) from the first frame it is in
/// to the last of the same stretch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    pub label: usize,
    pub first_frame: usize,
    pub last_frame: usize,
    /// The surest the tagger was of it in any frame of the stretch.
    pub confidence: f32,
    /// In how many frames it was heard.
    pub frames: usize,
}
