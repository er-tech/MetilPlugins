/// How good a grey picture is as a thumbnail: `(sharpness, brightness, score)`.
///
/// - **Sharpness** is the variance of the Laplacian (each pixel against its four neighbours): a
///   picture with edges and texture has a large one, a blur, a fade or a flat colour a small one.
/// - **Brightness** is the average pixel, 0 to 1.
/// - **Score** is the sharpness squeezed into 0 to 1 (`s / (s + 100)`) times how well exposed the
///   picture is: 1 at middle grey, falling to 0 at black and at white, so a black frame or a
///   washed out one never wins however sharp its noise is.
pub fn score_picture(pixels: &[u8], width: usize, height: usize) -> (f64, f64, f64) {
    if width == 0 || height == 0 || pixels.len() < width * height {
        return (0.0, 0.0, 0.0);
    }
    let brightness =
        pixels.iter().map(|pixel| f64::from(*pixel)).sum::<f64>() / pixels.len() as f64 / 255.0;
    let exposure = (1.0 - (brightness - 0.5).abs() * 2.0).clamp(0.0, 1.0);

    let sharpness = if width < 3 || height < 3 {
        0.0
    } else {
        let at = |x: usize, y: usize| f64::from(pixels[y * width + x]);
        let mut sum = 0.0;
        let mut sum_squares = 0.0;
        let mut count = 0.0;
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let laplacian =
                    at(x - 1, y) + at(x + 1, y) + at(x, y - 1) + at(x, y + 1) - 4.0 * at(x, y);
                sum += laplacian;
                sum_squares += laplacian * laplacian;
                count += 1.0;
            }
        }
        let mean = sum / count;
        sum_squares / count - mean * mean
    };
    let score = sharpness / (sharpness + 100.0) * exposure;
    (sharpness, brightness, score)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkerboard(size: usize, low: u8, high: u8) -> Vec<u8> {
        (0..size * size)
            .map(|n| {
                if (n % size + n / size).is_multiple_of(2) {
                    low
                } else {
                    high
                }
            })
            .collect()
    }

    #[test]
    fn a_sharp_picture_scores_far_above_a_flat_one_and_a_smooth_one() {
        let (sharp, _, sharp_score) = score_picture(&checkerboard(16, 40, 220), 16, 16);
        let (flat, _, flat_score) = score_picture(&vec![128; 256], 16, 16);
        let ramp: Vec<u8> = (0..256).map(|n| (n % 16 * 4 + 60) as u8).collect();
        let (smooth, _, smooth_score) = score_picture(&ramp, 16, 16);
        assert!(
            sharp > 1000.0 && flat == 0.0 && smooth < 1.0,
            "{sharp} {flat} {smooth}"
        );
        assert!(sharp_score > 0.8 && flat_score == 0.0 && smooth_score < 0.01);
    }

    #[test]
    fn black_and_white_frames_never_win_however_sharp_their_noise_is() {
        let dark = checkerboard(16, 0, 12);
        let white = checkerboard(16, 243, 255);
        let (_, dark_brightness, dark_score) = score_picture(&dark, 16, 16);
        let (_, white_brightness, white_score) = score_picture(&white, 16, 16);
        assert!(dark_brightness < 0.05 && white_brightness > 0.95);
        assert!(
            dark_score < 0.2 && white_score < 0.2,
            "{dark_score} {white_score}"
        );
        let (_, middle, good) = score_picture(&checkerboard(16, 90, 170), 16, 16);
        assert!((middle - 0.51).abs() < 0.02);
        assert!(good > dark_score && good > white_score);
    }

    #[test]
    fn a_description_that_does_not_fit_the_pixels_scores_nothing() {
        assert_eq!(score_picture(&[1, 2, 3], 8, 8), (0.0, 0.0, 0.0));
        assert_eq!(score_picture(&[], 0, 0), (0.0, 0.0, 0.0));
        // Too small to have an inside.
        assert_eq!(score_picture(&[9; 4], 2, 2).0, 0.0);
    }
}
