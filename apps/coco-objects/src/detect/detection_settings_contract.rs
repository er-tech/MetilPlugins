use loomasplugins_onnx_device::DeviceChoice;

/// What one look at a picture is told besides the picture: where the detector runs, how sure it
/// must be of a thing, and which of the 80 classes count.
#[derive(Debug, Clone, Copy)]
pub struct DetectionSettings {
    pub device: DeviceChoice,
    pub min_score: f32,
    pub wanted: [bool; 80],
}
