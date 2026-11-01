/// One square piece of an `rgb24` picture as the network wants it: three planes (red, green, blue)
/// of `tile` x `tile` numbers from 0 to 1. `left` and `top` are where the piece starts in the
/// picture and may be negative or run past the edge: the edge pixels are repeated there.
pub fn cut_tile(
    rgb: &[u8],
    width: usize,
    height: usize,
    left: isize,
    top: isize,
    tile: usize,
) -> Vec<f32> {
    let plane = tile * tile;
    let mut values = vec![0f32; 3 * plane];
    for row in 0..tile {
        let y = (top + row as isize).clamp(0, height as isize - 1) as usize;
        for column in 0..tile {
            let x = (left + column as isize).clamp(0, width as isize - 1) as usize;
            let at = (y * width + x) * 3;
            for channel in 0..3 {
                values[channel * plane + row * tile + column] =
                    f32::from(rgb[at + channel]) / 255.0;
            }
        }
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_piece_that_runs_past_the_edge_repeats_the_edge_pixels() {
        // Two pixels: red, then blue.
        let rgb = [255u8, 0, 0, 0, 0, 255];
        let piece = cut_tile(&rgb, 2, 1, -1, 0, 4);
        let red: Vec<f32> = piece[..16].to_vec();
        let blue: Vec<f32> = piece[32..48].to_vec();
        // Columns -1, 0, 1, 2 of the one row, repeated down the four rows.
        assert_eq!(red[..4], [1.0, 1.0, 0.0, 0.0]);
        assert_eq!(blue[..4], [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(red[12..16], red[..4]);
    }
}
