use loomasplugins_onnx_device::DeviceChoice;

/// What a call to `extract` asked for, checked and with the defaults filled in.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractOptions {
    /// Where the network runs.
    pub device: DeviceChoice,
    /// How much of the original sound to add back into the voice, from 0 to 1.
    pub keep: f32,
    /// The sample rate of the tracks that are given: 16000 (what the speech and sound plugins
    /// take) or 48000 (what the model works at).
    pub sample_rate: u32,
    /// Whether to give the rest as well as the voice.
    pub rest: bool,
    /// Whether to give each track as a WAV file in the result, too.
    pub files: bool,
}
