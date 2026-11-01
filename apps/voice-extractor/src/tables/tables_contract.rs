/// The sizes the network was trained with.
pub const FFT_SIZE: usize = 960;
/// How far apart two frames start, in samples (a half of the frame).
pub const HOP: usize = 480;
/// How many frequencies a frame of the transform has.
pub const BINS: usize = 481;
/// How many bands the frequencies are gathered in to tell loud from quiet.
pub const BANDS: usize = 32;
/// How many of the lowest frequencies the deep filter works on.
pub const FILTER_BINS: usize = 96;
/// How many frames the deep filter reads: the frame, and two before and after it.
pub const FILTER_ORDER: usize = 5;
/// How many of those come after the frame.
pub const FILTER_LOOKAHEAD: usize = 2;

/// The constants of the model: which frequencies go in which band, and the window.
#[derive(Debug, Clone, PartialEq)]
pub struct Tables {
    /// For each band, the frequencies in it and the weight of each one when the energy of the band
    /// is worked out (they sum to 1: the mean).
    pub bands: Vec<Vec<(usize, f32)>>,
    /// For each band, the frequencies a gain of the band is given to.
    pub spread: Vec<Vec<(usize, f32)>>,
    /// The window of the transform, `FFT_SIZE` values.
    pub window: Vec<f32>,
}
