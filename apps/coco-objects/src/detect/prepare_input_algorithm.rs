/// The grey the detector was trained to see around a picture that does not fill its square.
const PAD: f32 = 114.0;

/// A picture ready for the detector: the values it takes and how much the picture was shrunk.
pub struct PreparedInput {
    /// Three planes (blue, green, red) of `size * size` values from 0 to 255, the way the model
    /// was trained.
    pub tensor: Vec<f32>,
    /// New size over old size: a box in the tensor divided by it is a box in the picture.
    pub ratio: f32,
}

/// Fits an `rgb24` picture in a square of `size`, keeping its shape: it is shrunk (or grown) to
/// touch the square on one side with bilinear sampling, put in the top left corner, and the rest
/// is grey.
pub fn prepare_input(rgb: &[u8], width: usize, height: usize, size: usize) -> PreparedInput {
    let ratio = (size as f32 / height as f32).min(size as f32 / width as f32);
    let new_width = ((width as f32 * ratio) as usize).clamp(1, size);
    let new_height = ((height as f32 * ratio) as usize).clamp(1, size);
    let plane = size * size;
    let mut tensor = vec![PAD; 3 * plane];
    let last_x = (width - 1) as f32;
    let last_y = (height - 1) as f32;
    for y in 0..new_height {
        let source_y = ((y as f32 + 0.5) / ratio - 0.5).clamp(0.0, last_y);
        let (y0, fy) = (source_y as usize, source_y.fract());
        let y1 = (y0 + 1).min(height - 1);
        for x in 0..new_width {
            let source_x = ((x as f32 + 0.5) / ratio - 0.5).clamp(0.0, last_x);
            let (x0, fx) = (source_x as usize, source_x.fract());
            let x1 = (x0 + 1).min(width - 1);
            for channel in 0..3 {
                let at =
                    |row: usize, column: usize| rgb[(row * width + column) * 3 + channel] as f32;
                let top = at(y0, x0) * (1.0 - fx) + at(y0, x1) * fx;
                let bottom = at(y1, x0) * (1.0 - fx) + at(y1, x1) * fx;
                // The model was trained on blue, green, red: the reverse of the picture's order.
                tensor[(2 - channel) * plane + y * size + x] = top * (1.0 - fy) + bottom * fy;
            }
        }
    }
    PreparedInput { tensor, ratio }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_picture_is_shrunk_to_the_square_top_left_with_grey_below_and_blue_comes_first() {
        // 8 x 4 pixels, all red (255, 0, 0), into a square of 4: shrunk by a half to 4 x 2.
        let rgb: Vec<u8> = (0..8 * 4).flat_map(|_| [255u8, 0, 0]).collect();
        let prepared = prepare_input(&rgb, 8, 4, 4);

        assert_eq!(prepared.ratio, 0.5);
        assert_eq!(prepared.tensor.len(), 3 * 16);
        let (blue, green, red) = (
            &prepared.tensor[0..16],
            &prepared.tensor[16..32],
            &prepared.tensor[32..48],
        );
        // The picture is the top two rows...
        assert!(red[..8].iter().all(|v| (*v - 255.0).abs() < 1e-3));
        assert!(blue[..8].iter().chain(&green[..8]).all(|v| v.abs() < 1e-3));
        // ...and the two rows under it are the grey.
        assert!(red[8..].iter().chain(&blue[8..]).all(|v| *v == 114.0));
    }

    #[test]
    fn a_picture_of_the_size_of_the_square_is_copied_as_it_is() {
        let rgb = [10u8, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120];
        let prepared = prepare_input(&rgb, 2, 2, 2);
        assert_eq!(prepared.ratio, 1.0);
        // Blue plane: the third value of each pixel.
        assert_eq!(&prepared.tensor[0..4], &[30.0, 60.0, 90.0, 120.0]);
        assert_eq!(&prepared.tensor[8..12], &[10.0, 40.0, 70.0, 100.0]);
    }
}
