/// A picture of `width` by `height` scaled to `new_width` by `new_height` with bilinear sampling
/// (the middle of a pixel is its coordinate plus a half).
pub fn resize_bilinear(
    rgb: &[u8],
    width: usize,
    height: usize,
    new_width: usize,
    new_height: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; new_width * new_height * 3];
    let (sx, sy) = (
        width as f32 / new_width as f32,
        height as f32 / new_height as f32,
    );
    for y in 0..new_height {
        let fy = ((y as f32 + 0.5) * sy - 0.5).clamp(0.0, (height - 1) as f32);
        let (y0, wy) = (fy.floor() as usize, fy - fy.floor());
        let y1 = (y0 + 1).min(height - 1);
        for x in 0..new_width {
            let fx = ((x as f32 + 0.5) * sx - 0.5).clamp(0.0, (width - 1) as f32);
            let (x0, wx) = (fx.floor() as usize, fx - fx.floor());
            let x1 = (x0 + 1).min(width - 1);
            for channel in 0..3 {
                let at = |xx: usize, yy: usize| f32::from(rgb[(yy * width + xx) * 3 + channel]);
                let top = at(x0, y0) * (1.0 - wx) + at(x1, y0) * wx;
                let bottom = at(x0, y1) * (1.0 - wx) + at(x1, y1) * wx;
                out[(y * new_width + x) * 3 + channel] =
                    (top * (1.0 - wy) + bottom * wy).round() as u8;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_size_gives_the_same_picture_and_a_flat_colour_stays_flat() {
        let picture: Vec<u8> = (0..4 * 3 * 3).map(|n| (n * 7 % 251) as u8).collect();
        assert_eq!(resize_bilinear(&picture, 4, 3, 4, 3), picture);
        let flat = vec![90u8; 8 * 6 * 3];
        assert!(resize_bilinear(&flat, 8, 6, 3, 2).iter().all(|v| *v == 90));
    }

    #[test]
    fn halving_a_two_tone_picture_gives_its_halves() {
        let mut picture = vec![0u8; 4 * 2 * 3];
        for y in 0..2 {
            for x in 2..4 {
                picture[(y * 4 + x) * 3..(y * 4 + x) * 3 + 3].copy_from_slice(&[200, 200, 200]);
            }
        }
        let small = resize_bilinear(&picture, 4, 2, 2, 1);
        assert_eq!(small.len(), 6);
        assert!(small[0] < 80 && small[3] > 120, "{small:?}");
    }
}
