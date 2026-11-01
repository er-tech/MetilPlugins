use std::path::Path;

use metilplugins_onnx_device::DeviceChoice;
use metilplugins_plugin_sdk::Failure;

use crate::models::{ModelSpec, NETWORK_SCALE};
use crate::tiles::{cut_tile, paste_tile, tiles_along, Placement, TILE_MARGIN};

use super::upscaler_store::run_network;

/// One `rgb24` picture made four times larger by the network of `model`: an `rgb24` picture of
/// `width * 4` x `height * 4`. The picture is cut into pieces of `tile` pixels (the one size the
/// network accepts, when it has one) that share `TILE_MARGIN` pixels, and each answer is put back
/// without its margin.
pub fn upscale_frame(
    model: &ModelSpec,
    device: DeviceChoice,
    file: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    tile: usize,
) -> Result<Vec<u8>, Failure> {
    let scale = NETWORK_SCALE;
    let step = tile - 2 * TILE_MARGIN;
    let (out_width, out_height) = (width * scale, height * scale);
    let mut output = vec![0u8; out_width * out_height * 3];
    let side = tile * scale;
    for row in 0..tiles_along(height, step) {
        for column in 0..tiles_along(width, step) {
            let (left, top) = (column * step, row * step);
            let piece = cut_tile(
                rgb,
                width,
                height,
                left as isize - TILE_MARGIN as isize,
                top as isize - TILE_MARGIN as isize,
                tile,
            );
            let answer = run_network(model, device, file, piece, tile)?;
            if answer.len() != 3 * side * side {
                return Err(Failure::status(
                    422,
                    format!(
                        "the {} model answered with {} numbers for a piece of {tile} pixels, not the {} of a model that makes it {scale} times larger",
                        model.name,
                        answer.len(),
                        3 * side * side
                    ),
                ));
            }
            let place = Placement {
                tile,
                margin: TILE_MARGIN,
                scale,
                left,
                top,
            };
            paste_tile(&mut output, out_width, out_height, &answer, &place);
        }
    }
    Ok(output)
}
