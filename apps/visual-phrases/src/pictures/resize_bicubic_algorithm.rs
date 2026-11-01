/// Resizes an `rgb24` picture to `out_width` x `out_height` the way the Pillow library does with
/// its bicubic filter, which is how the model was trained: when the picture shrinks the filter is
/// stretched to cover the pixels it replaces (so nothing is skipped and edges do not crawl), the
/// horizontal pass is rounded to whole values and the vertical pass follows.
pub fn resize_bicubic(
    rgb: &[u8],
    width: usize,
    height: usize,
    out_width: usize,
    out_height: usize,
) -> Vec<u8> {
    let across = weights(width, out_width);
    let down = weights(height, out_height);
    // Horizontal pass: every row of the picture becomes `out_width` wide.
    let mut wide = vec![0u8; out_width * height * 3];
    for y in 0..height {
        for (x, (first, taps)) in across.iter().enumerate() {
            for channel in 0..3 {
                let sum: f32 = taps
                    .iter()
                    .enumerate()
                    .map(|(k, weight)| {
                        weight * f32::from(rgb[(y * width + first + k) * 3 + channel])
                    })
                    .sum();
                wide[(y * out_width + x) * 3 + channel] = to_byte(sum);
            }
        }
    }
    // Vertical pass.
    let mut out = vec![0u8; out_width * out_height * 3];
    for (y, (first, taps)) in down.iter().enumerate() {
        for x in 0..out_width {
            for channel in 0..3 {
                let sum: f32 = taps
                    .iter()
                    .enumerate()
                    .map(|(k, weight)| {
                        weight * f32::from(wide[((first + k) * out_width + x) * 3 + channel])
                    })
                    .sum();
                out[(y * out_width + x) * 3 + channel] = to_byte(sum);
            }
        }
    }
    out
}

fn to_byte(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

/// For each output position: the first input position it reads and the weight of each one from
/// there on, summing to 1.
fn weights(input: usize, output: usize) -> Vec<(usize, Vec<f32>)> {
    let scale = input as f32 / output as f32;
    let filter_scale = scale.max(1.0);
    let support = 2.0 * filter_scale;
    (0..output)
        .map(|position| {
            let centre = (position as f32 + 0.5) * scale;
            let first = ((centre - support + 0.5) as i64).max(0) as usize;
            let last = (((centre + support + 0.5) as i64).max(0) as usize).min(input);
            let mut taps: Vec<f32> = (first..last)
                .map(|at| bicubic((at as f32 - centre + 0.5) / filter_scale))
                .collect();
            let total: f32 = taps.iter().sum();
            if total != 0.0 {
                taps.iter_mut().for_each(|weight| *weight /= total);
            }
            (first, taps)
        })
        .collect()
}

/// The cubic filter with a = -0.5 (Catmull-Rom), zero from two away.
fn bicubic(x: f32) -> f32 {
    const A: f32 = -0.5;
    let x = x.abs();
    if x < 1.0 {
        ((A + 2.0) * x - (A + 3.0)) * x * x + 1.0
    } else if x < 2.0 {
        (((x - 5.0) * x + 8.0) * x - 4.0) * A
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_picture_stays_flat_whatever_the_size() {
        let rgb: Vec<u8> = (0..16 * 12).flat_map(|_| [10u8, 120, 250]).collect();
        let out = resize_bicubic(&rgb, 16, 12, 5, 7);
        assert_eq!(out.len(), 5 * 7 * 3);
        assert!(out.chunks(3).all(|pixel| pixel == [10, 120, 250]));
    }

    #[test]
    fn the_same_size_is_the_same_picture() {
        let rgb: Vec<u8> = (0..4 * 3 * 3).map(|n| (n * 7) as u8).collect();
        assert_eq!(resize_bicubic(&rgb, 4, 3, 4, 3), rgb);
    }

    #[test]
    fn halving_a_ramp_keeps_it_a_ramp() {
        // A picture 8 wide whose brightness goes up by 32 a pixel, shrunk to 4 wide.
        let rgb: Vec<u8> = (0..8).flat_map(|x| [(x * 32) as u8; 3]).collect();
        let out = resize_bicubic(&rgb, 8, 1, 4, 1);
        let reds: Vec<u8> = out.chunks(3).map(|pixel| pixel[0]).collect();
        assert!(reds.windows(2).all(|pair| pair[0] < pair[1]), "{reds:?}");
        assert!((i32::from(reds[0]) - 16).abs() <= 6, "{reds:?}");
    }
}
