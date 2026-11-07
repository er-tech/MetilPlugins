use std::path::Path;

use loomasplugins_onnx_device::{DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;
use ort::value::Tensor;

use crate::people::{Person, KEYPOINTS};
use crate::session::with_network;

use super::rtmpose_algorithm::{
    crop_for, decode_simcc, prepare_pose_input, POSE_HEIGHT, POSE_WIDTH,
};

/// The keypoints of the person in `bbox` (`x1, y1, x2, y2, score`) of one `rgb24` picture.
pub fn estimate_rtmpose(
    file: &Path,
    device: DeviceChoice,
    rgb: &[u8],
    size: (usize, usize),
    bbox: &[f32; 5],
) -> Result<(Person, DeviceReport), Failure> {
    let (centre, crop) = crop_for(bbox);
    let tensor = prepare_pose_input(rgb, size, centre, crop);
    let input = Tensor::from_array(([1usize, 3, POSE_HEIGHT, POSE_WIDTH], tensor))
        .map_err(|error| broken("the crop could not be given", &error))?;
    let (keypoints, report) = with_network(file, device, |session| {
        let outputs = session
            .run(ort::inputs!["input" => input])
            .map_err(|error| broken("it could not look", &error))?;
        let (_, simcc_x) = outputs["simcc_x"]
            .try_extract_tensor::<f32>()
            .map_err(|error| broken("its answer is not numbers", &error))?;
        let (_, simcc_y) = outputs["simcc_y"]
            .try_extract_tensor::<f32>()
            .map_err(|error| broken("its answer is not numbers", &error))?;
        if simcc_x.len() % KEYPOINTS != 0 || simcc_y.len() % KEYPOINTS != 0 || simcc_x.is_empty() {
            return Err(broken("its answer has the wrong size", &simcc_x.len()));
        }
        Ok(decode_simcc(simcc_x, simcc_y, centre, crop))
    })?;
    Ok((
        Person {
            keypoints,
            bbox: [bbox[0], bbox[1], bbox[2], bbox[3]],
            score: bbox[4],
        },
        report,
    ))
}

fn broken(what: &str, error: &dyn std::fmt::Display) -> Failure {
    Failure::status(500, format!("the pose network: {what}: {error}"))
}
