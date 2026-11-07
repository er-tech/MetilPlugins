use loomasplugins_onnx_device::DeviceChoice;

use crate::models::ModelSpec;
use crate::motion::MotionSettings;

/// The params of `find`, read and checked.
#[derive(Debug, Clone, Copy)]
pub struct FindOptions {
    pub model: &'static ModelSpec,
    pub device: DeviceChoice,
    /// Every how many of the pictures Loomas decodes (five a second) one is looked at.
    pub stride: usize,
    pub motion: MotionSettings,
    /// A person counts when the model is at least this sure.
    pub min_person_score: f32,
    /// No more people than this are followed in a picture.
    pub most_people: usize,
    /// A movement is kept when its confidence reaches this.
    pub min_confidence: f32,
    /// Movements of the same kind this close in time are one.
    pub gap_ms: u64,
    /// A movement is kept when it was seen in this many pictures.
    pub min_samples: usize,
}
