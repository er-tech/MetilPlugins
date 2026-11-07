use loomasplugins_onnx_device::DeviceChoice;

use crate::models::{Detector, Language};

/// The params of `find`, read and checked.
#[derive(Debug, Clone, Copy)]
pub struct FindOptions {
    pub language: &'static Language,
    pub detector: &'static Detector,
    pub device: DeviceChoice,
    /// Text read with less confidence than this is left out.
    pub min_confidence: f32,
    /// Text with fewer letters or digits than this is left out.
    pub min_chars: usize,
    /// A text that is not seen for longer than this has ended.
    pub gap_ms: u64,
}
