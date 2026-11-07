use crate::imaging::crop_to;
use crate::people::{Keypoint, KEYPOINTS};

/// The size of the picture the pose network takes: 192 wide, 256 high.
pub const POSE_WIDTH: usize = 192;
pub const POSE_HEIGHT: usize = 256;
/// The network answers with 2 numbers for each pixel of its input, along each axis.
const SIMCC_RATIO: f32 = 2.0;
/// The box of a person is made this much larger, so that the arms and legs are inside it.
const PADDING: f32 = 1.25;

const MEAN: [f32; 3] = [123.675, 116.28, 103.53];
const STD: [f32; 3] = [58.395, 57.12, 57.375];

/// Where in the picture the network looks for a person: the middle and the size of the crop, with
/// the shape the network takes (192 by 256) and the padding.
pub fn crop_for(bbox: &[f32; 5]) -> ((f32, f32), (f32, f32)) {
    let centre = ((bbox[0] + bbox[2]) / 2.0, (bbox[1] + bbox[3]) / 2.0);
    let (mut width, mut height) = ((bbox[2] - bbox[0]) * PADDING, (bbox[3] - bbox[1]) * PADDING);
    let shape = POSE_WIDTH as f32 / POSE_HEIGHT as f32;
    if width > height * shape {
        height = width / shape;
    } else {
        width = height * shape;
    }
    (centre, (width, height))
}

/// The crop of the person as the network takes it: normalised, red green blue, one plane after
/// another.
pub fn prepare_pose_input(
    rgb: &[u8],
    size: (usize, usize),
    centre: (f32, f32),
    crop: (f32, f32),
) -> Vec<f32> {
    let cut = crop_to(rgb, size.0, size.1, centre, crop, POSE_WIDTH, POSE_HEIGHT);
    let plane = POSE_WIDTH * POSE_HEIGHT;
    let mut tensor = vec![0.0f32; 3 * plane];
    for at in 0..plane {
        for channel in 0..3 {
            tensor[channel * plane + at] =
                (f32::from(cut[at * 3 + channel]) - MEAN[channel]) / STD[channel];
        }
    }
    tensor
}

/// The keypoints in the answer of the network: `simcc_x` and `simcc_y` hold, for each of the 17
/// keypoints, how likely each place along the width and along the height is. The best place on each
/// axis is the keypoint, put back in pixels of the picture; its score is the smaller of the two
/// best likelihoods.
pub fn decode_simcc(
    simcc_x: &[f32],
    simcc_y: &[f32],
    centre: (f32, f32),
    crop: (f32, f32),
) -> [Keypoint; KEYPOINTS] {
    let (columns, rows) = (simcc_x.len() / KEYPOINTS, simcc_y.len() / KEYPOINTS);
    let mut keypoints = [Keypoint::default(); KEYPOINTS];
    for (at, keypoint) in keypoints.iter_mut().enumerate() {
        let (x_at, x_best) = best(&simcc_x[at * columns..(at + 1) * columns]);
        let (y_at, y_best) = best(&simcc_y[at * rows..(at + 1) * rows]);
        let (x, y) = (x_at as f32 / SIMCC_RATIO, y_at as f32 / SIMCC_RATIO);
        *keypoint = Keypoint {
            x: x / POSE_WIDTH as f32 * crop.0 + centre.0 - crop.0 / 2.0,
            y: y / POSE_HEIGHT as f32 * crop.1 + centre.1 - crop.1 / 2.0,
            score: x_best.min(y_best),
        };
    }
    keypoints
}

/// The place and the value of the biggest number.
fn best(values: &[f32]) -> (usize, f32) {
    values
        .iter()
        .copied()
        .enumerate()
        .fold((0, f32::NEG_INFINITY), |top, (at, value)| {
            if value > top.1 {
                (at, value)
            } else {
                top
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crop_has_the_shape_of_the_network_and_is_a_quarter_larger_than_the_box() {
        let ((cx, cy), (w, h)) = crop_for(&[100.0, 50.0, 180.0, 250.0, 0.9]);
        assert_eq!((cx, cy), (140.0, 150.0));
        assert!((h - 250.0).abs() < 1e-3 && (w - 250.0 * 0.75).abs() < 1e-3);
        let (_, (w, h)) = crop_for(&[0.0, 0.0, 400.0, 100.0, 0.9]);
        assert!((w - 500.0).abs() < 1e-3 && (h - 500.0 / 0.75).abs() < 1e-3);
    }

    #[test]
    fn the_best_place_on_each_axis_is_the_keypoint_and_the_smaller_likelihood_its_score() {
        let mut x = vec![0.0f32; KEYPOINTS * 384];
        let mut y = vec![0.0f32; KEYPOINTS * 512];
        x[3 * 384 + 192] = 0.9; // the middle of the width: 96 of 192
        y[3 * 512 + 128] = 0.6; // a quarter of the height: 64 of 256
        let keypoints = decode_simcc(&x, &y, (500.0, 400.0), (192.0, 256.0));
        let k = keypoints[3];
        assert!((k.x - 500.0).abs() < 1e-3, "{k:?}");
        assert!((k.y - (400.0 - 128.0 + 64.0)).abs() < 1e-3, "{k:?}");
        assert!((k.score - 0.6).abs() < 1e-6);
    }

    #[test]
    fn the_crop_is_normalised_with_the_mean_and_the_spread_of_the_network() {
        let rgb = vec![123u8; 20 * 20 * 3];
        let tensor = prepare_pose_input(&rgb, (20, 20), (10.0, 10.0), (20.0, 20.0 * 256.0 / 192.0));
        assert_eq!(tensor.len(), 3 * POSE_WIDTH * POSE_HEIGHT);
        let plane = POSE_WIDTH * POSE_HEIGHT;
        let middle = plane / 2 + POSE_WIDTH / 2;
        assert!((tensor[middle] - (123.0 - 123.675) / 58.395).abs() < 0.05);
        assert!((tensor[plane + middle] - (123.0 - 116.28) / 57.12).abs() < 0.05);
    }
}
