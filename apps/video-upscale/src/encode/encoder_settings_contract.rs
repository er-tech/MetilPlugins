/// What the encoder needs to know about the pictures it gets and the video it writes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EncoderSettings {
    /// The size of the pictures written to the encoder.
    pub in_width: usize,
    pub in_height: usize,
    /// The size of the video: the pictures are scaled to it (Lanczos) when it is not their size.
    pub out_width: usize,
    pub out_height: usize,
    pub frame_rate: f64,
}
