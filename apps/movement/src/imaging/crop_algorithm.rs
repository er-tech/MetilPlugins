/// The box with middle (`cx`, `cy`) and size `box_width` by `box_height`, cut out of a picture of
/// `width` by `height` and scaled to `out_width` by `out_height` with bilinear sampling. What lies
/// outside the picture is black.
#[allow(clippy::too_many_arguments)]
pub fn crop_to(
    rgb: &[u8],
    width: usize,
    height: usize,
    centre: (f32, f32),
    size: (f32, f32),
    out_width: usize,
    out_height: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; out_width * out_height * 3];
    let left = centre.0 - size.0 / 2.0;
    let top = centre.1 - size.1 / 2.0;
    for v in 0..out_height {
        let sy = top + (v as f32 + 0.5) * size.1 / out_height as f32 - 0.5;
        for u in 0..out_width {
            let sx = left + (u as f32 + 0.5) * size.0 / out_width as f32 - 0.5;
            if sx < -0.5 || sy < -0.5 || sx > width as f32 - 0.5 || sy > height as f32 - 0.5 {
                continue;
            }
            let (fx, fy) = (
                sx.clamp(0.0, (width - 1) as f32),
                sy.clamp(0.0, (height - 1) as f32),
            );
            let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
            let (x1, y1) = ((x0 + 1).min(width - 1), (y0 + 1).min(height - 1));
            let (wx, wy) = (fx - x0 as f32, fy - y0 as f32);
            for channel in 0..3 {
                let at = |x: usize, y: usize| f32::from(rgb[(y * width + x) * 3 + channel]);
                let upper = at(x0, y0) * (1.0 - wx) + at(x1, y0) * wx;
                let lower = at(x0, y1) * (1.0 - wx) + at(x1, y1) * wx;
                out[(v * out_width + u) * 3 + channel] =
                    (upper * (1.0 - wy) + lower * wy).round() as u8;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crop_of_the_whole_picture_keeps_it_and_a_box_outside_is_black() {
        let picture: Vec<u8> = (0..6 * 4 * 3).map(|n| (n * 5 % 253) as u8).collect();
        let whole = crop_to(&picture, 6, 4, (3.0, 2.0), (6.0, 4.0), 6, 4);
        assert_eq!(whole, picture);
        let outside = crop_to(&picture, 6, 4, (100.0, 100.0), (6.0, 4.0), 6, 4);
        assert!(outside.iter().all(|v| *v == 0));
    }

    #[test]
    fn a_crop_takes_the_part_it_names() {
        let mut picture = vec![0u8; 8 * 8 * 3];
        for y in 4..8 {
            for x in 4..8 {
                picture[(y * 8 + x) * 3..(y * 8 + x) * 3 + 3].copy_from_slice(&[255, 255, 255]);
            }
        }
        let corner = crop_to(&picture, 8, 8, (6.0, 6.0), (4.0, 4.0), 2, 2);
        assert!(corner.iter().all(|v| *v > 240), "{corner:?}");
        let other = crop_to(&picture, 8, 8, (2.0, 2.0), (4.0, 4.0), 2, 2);
        assert!(other.iter().all(|v| *v < 10), "{other:?}");
    }
}
