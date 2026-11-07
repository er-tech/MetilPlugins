use loomasplugins_onnx_device::{DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;
use ort::value::Tensor;

use crate::models::{ModelPaths, ModelSpec, Pipeline};
use crate::people::Person;
use crate::session::with_network;

use super::detect_people_use_case::detect_people;
use super::estimate_rtmpose_use_case::estimate_rtmpose;
use super::movenet_algorithm::{decode_movenet, prepare_movenet_input};

/// MoveNet rates the people it finds lower than a detector does (0.3 to 0.45 for people who are
/// clearly there), so the limit it is held to is never higher than this.
const MOVENET_MOST_SURE: f32 = 0.25;

/// What a picture is searched with.
#[derive(Debug, Clone, Copy)]
pub struct FindSettings {
    pub device: DeviceChoice,
    /// A person counts when the detector is at least this sure.
    pub min_score: f32,
    /// No more people than this are followed in a picture.
    pub most: usize,
}

/// The people in one `rgb24` picture of `size` (width, height) with the keypoints of their bodies,
/// the surest first, and what the networks really run on.
pub fn find_people(
    spec: &ModelSpec,
    paths: &ModelPaths,
    rgb: &[u8],
    size: (usize, usize),
    settings: &FindSettings,
) -> Result<(Vec<Person>, Option<DeviceReport>), Failure> {
    match spec.pipeline {
        Pipeline::RtmPose => {
            let detector = paths
                .detector
                .as_deref()
                .ok_or_else(|| Failure::status(500, "this model has no detector"))?;
            let (boxes, _) = detect_people(
                detector,
                settings.device,
                spec.detector_size,
                rgb,
                size,
                settings.min_score,
                settings.most,
            )?;
            let mut people = Vec::with_capacity(boxes.len());
            let mut report = None;
            for bbox in &boxes {
                let (person, used) =
                    estimate_rtmpose(&paths.pose, settings.device, rgb, size, bbox)?;
                people.push(person);
                report = Some(used);
            }
            Ok((people, report))
        }
        Pipeline::MoveNet => {
            let (numbers, height, width) = prepare_movenet_input(rgb, size.0, size.1);
            let input =
                Tensor::from_array(([1usize, height, width, 3], numbers)).map_err(|error| {
                    Failure::status(500, format!("the picture could not be given: {error}"))
                })?;
            let (answer, report) = with_network(&paths.pose, settings.device, |session| {
                let outputs = session
                    .run(ort::inputs!["input" => input])
                    .map_err(|error| {
                        Failure::status(500, format!("the network could not look: {error}"))
                    })?;
                let (_, numbers) =
                    outputs["output_0"]
                        .try_extract_tensor::<f32>()
                        .map_err(|error| {
                            Failure::status(500, format!("its answer is not numbers: {error}"))
                        })?;
                Ok(numbers.to_vec())
            })?;
            let mut people =
                decode_movenet(&answer, size, settings.min_score.min(MOVENET_MOST_SURE));
            people.truncate(settings.most);
            Ok((people, Some(report)))
        }
    }
}
