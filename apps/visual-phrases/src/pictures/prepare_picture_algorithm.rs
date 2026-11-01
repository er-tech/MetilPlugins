use super::resize_bicubic_algorithm::resize_bicubic;

/// The side of the square picture the model looks at.
pub const SIDE: usize = 224;

/// An `rgb24` picture as the image model takes it: squeezed (not cropped, so nothing is lost at the
/// edges) to `SIDE` x `SIDE`, and as three planes (red, green, blue) of numbers from -1 to 1.
pub fn prepare_picture(rgb: &[u8], width: usize, height: usize) -> Vec<f32> {
    let squeezed = resize_bicubic(rgb, width, height, SIDE, SIDE);
    let plane = SIDE * SIDE;
    let mut values = vec![0f32; 3 * plane];
    for (at, pixel) in squeezed.as_chunks::<3>().0.iter().enumerate() {
        for channel in 0..3 {
            values[channel * plane + at] = f32::from(pixel[channel]) / 127.5 - 1.0;
        }
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_becomes_three_planes_of_numbers_from_minus_one_to_one() {
        // 2 x 2 pixels: black, white, pure red, pure blue.
        let rgb = [0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255];
        let values = prepare_picture(&rgb, 2, 2);
        let plane = SIDE * SIDE;
        assert_eq!(values.len(), 3 * plane);
        assert!(values.iter().all(|v| (-1.0..=1.0).contains(v)));
        // The top left corner is black, and the top right (white) is 1 in every plane.
        assert_eq!(values[0], -1.0);
        let top_right = SIDE - 1;
        assert!((values[top_right] - 1.0).abs() < 1e-6);
        assert!((values[plane + top_right] - 1.0).abs() < 1e-6);
        // Bottom left is red: 1 in the red plane, -1 in green and blue.
        let bottom_left = (SIDE - 1) * SIDE;
        assert!((values[bottom_left] - 1.0).abs() < 1e-6);
        assert!((values[plane + bottom_left] + 1.0).abs() < 1e-6);
        assert!((values[2 * plane + bottom_left] + 1.0).abs() < 1e-6);
    }
}
