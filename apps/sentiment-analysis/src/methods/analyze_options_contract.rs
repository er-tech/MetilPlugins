use loomasplugins_onnx_device::DeviceChoice;

use crate::models::ModelSpec;

/// The params of `analyze`, read and checked.
#[derive(Debug, Clone, Copy)]
pub struct AnalyzeOptions {
    pub model: &'static ModelSpec,
    pub device: DeviceChoice,
}
