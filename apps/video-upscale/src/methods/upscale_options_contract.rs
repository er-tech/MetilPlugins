use metilplugins_onnx_device::DeviceChoice;

use crate::models::ModelSpec;

/// The params of `upscale`, read and checked.
#[derive(Debug, Clone, Copy)]
pub struct UpscaleOptions {
    pub model: &'static ModelSpec,
    pub device: DeviceChoice,
    /// The side of a piece, for a network that takes any size.
    pub tile: usize,
    /// How many times larger the video is: 2 or 4.
    pub scale: usize,
}
