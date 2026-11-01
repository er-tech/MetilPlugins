use realfft::num_complex::Complex32;

use crate::tables::BINS;

/// The frames of a transform one after another: `BINS` complex numbers for each.
#[derive(Debug, Clone, PartialEq)]
pub struct Spectrogram {
    pub frames: usize,
    pub values: Vec<Complex32>,
}

impl Spectrogram {
    pub fn frame(&self, index: usize) -> &[Complex32] {
        &self.values[index * BINS..(index + 1) * BINS]
    }
}
