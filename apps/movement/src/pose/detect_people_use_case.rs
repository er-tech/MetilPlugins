use std::path::Path;

use loomasplugins_onnx_device::{DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;
use ort::value::Tensor;

use crate::session::with_network;

use super::yolox_algorithm::{decode_people, prepare_detector_input};

/// The people a detector finds in one `rgb24` picture: boxes `x1, y1, x2, y2, score` in pixels of
/// the picture, the surest first, at least `min_score` sure and no more than `most`. The detector
/// looks at a square of `side` pixels.
pub fn detect_people(
    file: &Path,
    device: DeviceChoice,
    side: usize,
    rgb: &[u8],
    size: (usize, usize),
    min_score: f32,
    most: usize,
) -> Result<(Vec<[f32; 5]>, DeviceReport), Failure> {
    let (tensor, ratio) = prepare_detector_input(rgb, size.0, size.1, side);
    let input = Tensor::from_array(([1usize, 3, side, side], tensor))
        .map_err(|error| broken("the picture could not be given", &error))?;
    let (found, report) = with_network(file, device, |session| {
        let outputs = session
            .run(ort::inputs!["input" => input])
            .map_err(|error| broken("it could not look", &error))?;
        let (_, dets) = outputs["dets"]
            .try_extract_tensor::<f32>()
            .map_err(|error| broken("its boxes are not numbers", &error))?;
        let (_, labels) = outputs["labels"]
            .try_extract_tensor::<i64>()
            .map_err(|error| broken("its classes are not numbers", &error))?;
        Ok(decode_people(dets, labels, ratio, size, min_score, most))
    })?;
    Ok((found, report))
}

fn broken(what: &str, error: &dyn std::fmt::Display) -> Failure {
    Failure::status(500, format!("the detector: {what}: {error}"))
}
