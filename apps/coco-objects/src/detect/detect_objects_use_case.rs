use std::path::Path;

use loomasplugins_plugin_sdk::Failure;

use crate::models::ModelSpec;

use super::decode_output_algorithm::decode_output;
use super::detection_contract::Detection;
use super::detection_settings_contract::DetectionSettings;
use super::detector_store::run_detector;
use super::non_max_suppression_algorithm::non_max_suppression;
use super::prepare_input_algorithm::prepare_input;

/// Boxes that cover more than this much of a surer box of the same class are the same thing.
const MAX_OVERLAP: f32 = 0.45;

/// What the detector finds in one `rgb24` picture: the things of the `wanted` classes it is at
/// least `min_score` sure of, the surest first, with their boxes in pixels of the picture.
pub fn detect_objects(
    model: &ModelSpec,
    file: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    settings: &DetectionSettings,
) -> Result<Vec<Detection>, Failure> {
    let DetectionSettings {
        device,
        min_score,
        wanted,
    } = settings;
    let prepared = prepare_input(rgb, width, height, model.input_size);
    let answer = run_detector(model, *device, file, prepared.tensor)?;
    let found = decode_output(
        &answer,
        model.input_size,
        prepared.ratio,
        *min_score,
        wanted,
    )
    .map_err(|message| Failure::status(422, message))?;
    let (right, bottom) = (width as f32, height as f32);
    Ok(non_max_suppression(found, MAX_OVERLAP)
        .into_iter()
        .map(|found| Detection {
            x1: found.x1.clamp(0.0, right),
            y1: found.y1.clamp(0.0, bottom),
            x2: found.x2.clamp(0.0, right),
            y2: found.y2.clamp(0.0, bottom),
            ..found
        })
        .collect())
}
