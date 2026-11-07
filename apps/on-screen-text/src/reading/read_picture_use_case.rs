use std::path::Path;

use loomasplugins_onnx_device::{DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;
use ort::value::Tensor;

use crate::models::ModelPaths;
use crate::session::with_network;

use super::detect_text_algorithm::{find_text_boxes, prepare_detector_input};
use super::recognize_text_algorithm::{decode_ctc, prepare_line_input, LINE_HEIGHT};

/// What a network answered: the shape of its answer and its numbers.
type Answer = (Vec<i64>, Vec<f32>);

/// A line of text read in a picture.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    /// The mean chance of its characters, 0 to 1.
    pub confidence: f32,
    /// The corners in pixels of the picture: `x1, y1, x2, y2`.
    pub bbox: [f32; 4],
}

/// The text in one `rgb24` picture of `size` (width, height): where the lines are, then what each
/// says. `characters` is the list the recognizer knows (see `load_characters`). Lines in which
/// nothing could be read are left out. Also what the networks really run on.
pub fn read_picture(
    paths: &ModelPaths,
    characters: &[String],
    device: DeviceChoice,
    rgb: &[u8],
    size: (usize, usize),
) -> Result<(Vec<Line>, Option<DeviceReport>), Failure> {
    let input = prepare_detector_input(rgb, size.0, size.1);
    let tensor = Tensor::from_array(([1usize, 3, input.height, input.width], input.tensor.clone()))
        .map_err(|error| broken("the picture could not be given", &error))?;
    let (chances, detected_on) = run_first_output(&paths.detector, device, tensor)?;
    let mut report = Some(detected_on);
    let boxes = find_text_boxes(&chances.1, input.width, input.height, size);
    let mut lines = Vec::with_capacity(boxes.len());
    for text_box in &boxes {
        let line = prepare_line_input(rgb, size, text_box);
        let tensor = Tensor::from_array(([1usize, 3, LINE_HEIGHT, line.width], line.tensor))
            .map_err(|error| broken("the line could not be given", &error))?;
        let ((shape, answer), used) = run_first_output(&paths.recognizer, device, tensor)?;
        report = Some(used);
        let classes = shape.last().copied().unwrap_or(0).max(0) as usize;
        let (text, confidence) = decode_ctc(&answer, classes, characters);
        if !text.trim().is_empty() {
            lines.push(Line {
                text: text.trim().to_string(),
                confidence,
                bbox: [text_box.x1, text_box.y1, text_box.x2, text_box.y2],
            });
        }
    }
    Ok((lines, report))
}

/// Runs the network in `file` with one input and gives its first output (its shape and its numbers)
/// and what it runs on.
fn run_first_output(
    file: &Path,
    device: DeviceChoice,
    input: Tensor<f32>,
) -> Result<(Answer, DeviceReport), Failure> {
    let (output, report) = with_network(file, device, |session| {
        let input_name = session
            .inputs()
            .first()
            .map(|input| input.name().to_string())
            .ok_or_else(|| broken("it has no input", &"empty"))?;
        let output_name = session
            .outputs()
            .first()
            .map(|output| output.name().to_string())
            .ok_or_else(|| broken("it has no output", &"empty"))?;
        let outputs = session
            .run(ort::inputs![input_name.as_str() => input])
            .map_err(|error| broken("it could not read", &error))?;
        let (shape, numbers) = outputs[output_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|error| broken("its answer is not numbers", &error))?;
        Ok((
            shape.iter().copied().collect::<Vec<i64>>(),
            numbers.to_vec(),
        ))
    })?;
    Ok((output, report))
}

fn broken(what: &str, error: &dyn std::fmt::Display) -> Failure {
    Failure::status(500, format!("the network: {what}: {error}"))
}
