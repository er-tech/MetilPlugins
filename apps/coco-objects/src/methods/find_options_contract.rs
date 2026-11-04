use loomasplugins_onnx_device::DeviceChoice;

use crate::models::ModelSpec;

/// What a call to `find` asked for, checked and with the defaults filled in.
#[derive(Debug, Clone, PartialEq)]
pub struct FindOptions {
    pub model: &'static ModelSpec,
    /// Where the detector runs: the GPU when there is a discrete one (`auto`), `cpu`, or `gpu`.
    pub device: DeviceChoice,
    /// One flag for each of the 80 classes: whether to look for it.
    pub wanted: [bool; 80],
    /// How sure the detector must be of a thing, from 0 to 1.
    pub min_confidence: f32,
    /// How long a class can be out of the pictures before it counts as gone.
    pub gap_ms: u64,
    /// In how few pictures a thing is still worth reporting.
    pub min_samples: usize,
}
