use std::path::Path;

use metilplugins_onnx_device::DeviceChoice;
use metilplugins_plugin_sdk::Failure;

use crate::matching::normalize;
use crate::session::{embed_pictures, DIMENSIONS};

use super::prepare_picture_algorithm::prepare_picture;

/// How many pictures go to the model at once.
const BATCH: usize = 8;

/// A picture of the video, as `rgb24` pixels and its size.
pub struct Picture<'a> {
    pub rgb: &'a [u8],
    pub width: usize,
    pub height: usize,
}

/// The numbers of each picture, scaled to length 1, one after another (`DIMENSIONS` for each).
/// `each` is told after every batch how many pictures are done, for progress and cancelling.
pub fn embed_all_pictures(
    file: &Path,
    device: DeviceChoice,
    pictures: &[Picture],
    mut each: impl FnMut(usize) -> Result<(), Failure>,
) -> Result<Vec<f32>, Failure> {
    let mut embeddings = Vec::with_capacity(pictures.len() * DIMENSIONS);
    for (batch, group) in pictures.chunks(BATCH).enumerate() {
        let values: Vec<f32> = group
            .iter()
            .flat_map(|picture| prepare_picture(picture.rgb, picture.width, picture.height))
            .collect();
        let mut numbers = embed_pictures(file, device, values, group.len())?;
        numbers.chunks_mut(DIMENSIONS).for_each(normalize);
        embeddings.extend(numbers);
        each(batch * BATCH + group.len())?;
    }
    Ok(embeddings)
}
