use crate::people::{Keypoint, Person, KEYPOINTS};
use loomasplugins_picture_ops::resize_bilinear;

/// The long side of the picture MoveNet looks at; the other side is a multiple of 32.
const LONG_SIDE: usize = 256;

/// A picture as MoveNet takes it: scaled so that its long side is 256 and its short side a multiple
/// of 32, as whole numbers 0 to 255 in the order of the rows, red green blue. Also the height and
/// the width.
pub fn prepare_movenet_input(rgb: &[u8], width: usize, height: usize) -> (Vec<i32>, usize, usize) {
    let multiple = |short: usize, long: usize| {
        (((short * LONG_SIDE) as f32 / long as f32 / 32.0).round() as usize * 32).max(32)
    };
    let (new_width, new_height) = if width >= height {
        (LONG_SIDE, multiple(height, width))
    } else {
        (multiple(width, height), LONG_SIDE)
    };
    let small = resize_bilinear(rgb, width, height, new_width, new_height);
    (
        small.into_iter().map(i32::from).collect(),
        new_height,
        new_width,
    )
}

/// The people in the answer of MoveNet: 6 rows of 56 numbers. A row is 17 keypoints as
/// `y, x, score` (from 0 to 1 across the picture), then the box as `y1, x1, y2, x2` and the score of
/// the person. The people at least `min_score` sure, in pixels of the picture, the surest first.
pub fn decode_movenet(answer: &[f32], size: (usize, usize), min_score: f32) -> Vec<Person> {
    let (width, height) = (size.0 as f32, size.1 as f32);
    let (rows, _) = answer.as_chunks::<56>();
    let mut people: Vec<Person> = rows
        .iter()
        .filter(|row| row[55] >= min_score)
        .map(|row| {
            let mut keypoints = [Keypoint::default(); KEYPOINTS];
            for (at, keypoint) in keypoints.iter_mut().enumerate() {
                *keypoint = Keypoint {
                    x: row[at * 3 + 1] * width,
                    y: row[at * 3] * height,
                    score: row[at * 3 + 2],
                };
            }
            Person {
                keypoints,
                bbox: [
                    row[52] * width,
                    row[51] * height,
                    row[54] * width,
                    row[53] * height,
                ],
                score: row[55],
            }
        })
        .collect();
    people.sort_by(|a, b| b.score.total_cmp(&a.score));
    people
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_picture_gets_a_long_side_of_256_and_a_short_side_that_is_a_multiple_of_32() {
        let rgb = vec![10u8; 640 * 360 * 3];
        let (numbers, height, width) = prepare_movenet_input(&rgb, 640, 360);
        assert_eq!((width, height), (256, 160)); // 144 rounds to 160
        assert_eq!(numbers.len(), 256 * 160 * 3);
        let (_, height, width) = prepare_movenet_input(&vec![0u8; 360 * 640 * 3], 360, 640);
        assert_eq!((width, height), (160, 256)); // 144 rounds to 160
    }

    #[test]
    fn people_are_read_in_picture_pixels_and_the_unsure_ones_are_left_out() {
        let mut answer = vec![0.0f32; 6 * 56];
        for (person, score) in [(0usize, 0.2f32), (1, 0.8)] {
            let row = &mut answer[person * 56..(person + 1) * 56];
            row[0] = 0.5; // nose y
            row[1] = 0.25; // nose x
            row[2] = 0.9;
            row[51..55].copy_from_slice(&[0.1, 0.2, 0.9, 0.4]);
            row[55] = score;
        }
        let people = decode_movenet(&answer, (640, 360), 0.3);
        assert_eq!(people.len(), 1);
        let nose = people[0].keypoints[0];
        assert_eq!((nose.x, nose.y), (160.0, 180.0));
        assert_eq!(people[0].bbox, [128.0, 36.0, 256.0, 324.0]);
    }
}
