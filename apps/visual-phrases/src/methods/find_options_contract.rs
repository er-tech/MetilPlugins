use loomasplugins_onnx_device::DeviceChoice;

/// What a call to `find` asked for, checked and with the defaults filled in.
#[derive(Debug, Clone, PartialEq)]
pub struct FindOptions {
    /// What to look for, in words.
    pub phrases: Vec<String>,
    /// Where the image model runs: `cpu` (the default here), `gpu` or `auto`.
    pub device: DeviceChoice,
    /// How sure the model must be that a phrase describes a picture, from 0 to 1.
    pub min_confidence: f32,
    /// How long a phrase can stop being true before it counts as over.
    pub gap_ms: u64,
    /// In how few pictures a phrase is still worth reporting.
    pub min_samples: usize,
}
