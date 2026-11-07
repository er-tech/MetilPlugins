use crate::imaging::resize_bilinear;

/// A picture as the detector takes it: scaled to fit a square of `side` pixels and put in its corner on grey
/// (114), as blue, green, red numbers from 0 to 255, one plane after another. Also the ratio the
/// picture was scaled by.
pub fn prepare_detector_input(
    rgb: &[u8],
    width: usize,
    height: usize,
    side: usize,
) -> (Vec<f32>, f32) {
    let ratio = (side as f32 / height as f32).min(side as f32 / width as f32);
    let new_width = ((width as f32 * ratio).round() as usize).clamp(1, side);
    let new_height = ((height as f32 * ratio).round() as usize).clamp(1, side);
    let small = resize_bilinear(rgb, width, height, new_width, new_height);
    let plane = side * side;
    let mut tensor = vec![114.0f32; 3 * plane];
    for y in 0..new_height {
        for x in 0..new_width {
            let from = (y * new_width + x) * 3;
            let at = y * side + x;
            tensor[at] = f32::from(small[from + 2]);
            tensor[plane + at] = f32::from(small[from + 1]);
            tensor[2 * plane + at] = f32::from(small[from]);
        }
    }
    (tensor, ratio)
}

/// The boxes of people in the answer of the detector: `dets` holds `labels.len()` rows of
/// `x1, y1, x2, y2, score` in pixels of the square, `labels` the class of each (0 is a person).
/// The boxes of people at least `min_score` sure, put back in pixels of the picture and kept inside
/// it, the surest first and no more than `most`.
pub fn decode_people(
    dets: &[f32],
    labels: &[i64],
    ratio: f32,
    size: (usize, usize),
    min_score: f32,
    most: usize,
) -> Vec<[f32; 5]> {
    let (rows, _) = dets.as_chunks::<5>();
    let mut found: Vec<[f32; 5]> = rows
        .iter()
        .zip(labels)
        .filter(|(row, label)| **label == 0 && row[4] >= min_score)
        .map(|(row, _)| {
            [
                (row[0] / ratio).clamp(0.0, size.0 as f32),
                (row[1] / ratio).clamp(0.0, size.1 as f32),
                (row[2] / ratio).clamp(0.0, size.0 as f32),
                (row[3] / ratio).clamp(0.0, size.1 as f32),
                row[4],
            ]
        })
        .filter(|b| b[2] - b[0] >= 4.0 && b[3] - b[1] >= 4.0)
        .collect();
    found.sort_by(|a, b| b[4].total_cmp(&a[4]));
    found.truncate(most);
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_picture_is_scaled_to_the_width_and_put_in_the_corner_on_grey() {
        let rgb = vec![200u8; 832 * 416 * 3];
        let (tensor, ratio) = prepare_detector_input(&rgb, 832, 416, 416);
        assert!((ratio - 0.5).abs() < 1e-6);
        let plane = 416 * 416;
        assert_eq!(tensor.len(), 3 * plane);
        assert_eq!(tensor[0], 200.0);
        assert_eq!(tensor[416 * 300], 114.0);
        let (large, ratio) = prepare_detector_input(&rgb, 832, 416, 640);
        assert_eq!(large.len(), 3 * 640 * 640);
        assert!((ratio - 640.0 / 832.0).abs() < 1e-6);
    }

    #[test]
    fn colours_go_in_as_blue_green_red() {
        let mut rgb = vec![0u8; 4 * 4 * 3];
        rgb[0..3].copy_from_slice(&[10, 20, 30]);
        let (tensor, _) = prepare_detector_input(&rgb, 4, 4, 416);
        let plane = 416 * 416;
        assert_eq!(
            (tensor[0], tensor[plane], tensor[2 * plane]),
            (30.0, 20.0, 10.0)
        );
    }

    #[test]
    fn only_people_that_are_sure_enough_are_kept_in_picture_pixels_surest_first() {
        let dets = [
            10.0, 20.0, 110.0, 220.0, 0.6, //
            0.0, 0.0, 50.0, 50.0, 0.9, //
            300.0, 10.0, 400.0, 100.0, 0.95, //
            0.0, 0.0, 1.0, 1.0, 0.99,
        ];
        let found = decode_people(&dets, &[0, 0, 15, 0], 0.5, (640, 480), 0.5, 5);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0][4], 0.9);
        assert_eq!(&found[1][..4], &[20.0, 40.0, 220.0, 440.0]);
        assert_eq!(
            decode_people(&dets, &[0, 0, 15, 0], 0.5, (640, 480), 0.5, 1).len(),
            1
        );
    }
}
