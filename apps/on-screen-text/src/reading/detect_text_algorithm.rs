use loomasplugins_picture_ops::resize_bilinear;

use super::text_box_contract::TextBox;

/// A picture whose longer side is more than this is scaled down to it before the detector looks.
const LIMIT_SIDE: usize = 960;
/// The numbers the detector was trained with, for blue, green and red.
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];
/// A pixel is text when the detector gives it at least this chance.
const PIXEL_THRESHOLD: f32 = 0.3;
/// A line is kept when its pixels have this chance on average.
const BOX_THRESHOLD: f32 = 0.6;
/// The detector draws the text a little smaller than it is, so each box is grown by this many
/// times (area over perimeter) in every direction.
const UNCLIP: f32 = 1.5;
/// A patch of fewer pixels than this is noise.
const MIN_AREA: usize = 20;

/// A picture as the detector takes it.
#[derive(Debug, Clone, PartialEq)]
pub struct DetectorInput {
    pub tensor: Vec<f32>,
    pub height: usize,
    pub width: usize,
}

/// A picture as the detector takes it: scaled down when its long side is more than 960, each side a
/// multiple of 32, as blue, green and red numbers normalised with the mean and spread of the
/// training, one plane after another.
pub fn prepare_detector_input(rgb: &[u8], width: usize, height: usize) -> DetectorInput {
    let longest = width.max(height);
    let scale = if longest > LIMIT_SIDE {
        LIMIT_SIDE as f32 / longest as f32
    } else {
        1.0
    };
    let multiple = |side: usize| (((side as f32 * scale) / 32.0).round() as usize * 32).max(32);
    let (new_width, new_height) = (multiple(width), multiple(height));
    let small = resize_bilinear(rgb, width, height, new_width, new_height);
    let plane = new_width * new_height;
    let mut tensor = vec![0.0f32; 3 * plane];
    for at in 0..plane {
        for (channel, source) in [2usize, 1, 0].into_iter().enumerate() {
            let value = f32::from(small[at * 3 + source]) / 255.0;
            tensor[channel * plane + at] = (value - MEAN[channel]) / STD[channel];
        }
    }
    DetectorInput {
        tensor,
        height: new_height,
        width: new_width,
    }
}

/// The lines of text in the answer of the detector, a map of `width` by `height` chances: the
/// pixels with at least 0.3 are joined into patches, a patch is a line when it has 20 pixels or
/// more and its pixels have 0.6 on average, and its box is grown by the rule above. The boxes are in
/// pixels of the picture of size `picture` (width, height), kept inside it, from the top down and
/// from left to right.
pub fn find_text_boxes(
    chances: &[f32],
    width: usize,
    height: usize,
    picture: (usize, usize),
) -> Vec<TextBox> {
    if chances.len() < width * height {
        return Vec::new();
    }
    let mut seen = vec![false; width * height];
    let mut boxes = Vec::new();
    let mut stack = Vec::new();
    for start in 0..width * height {
        if seen[start] || chances[start] <= PIXEL_THRESHOLD {
            continue;
        }
        seen[start] = true;
        stack.push(start);
        let (mut left, mut right, mut top, mut bottom) = (width, 0, height, 0);
        let (mut area, mut sum) = (0usize, 0.0f32);
        while let Some(at) = stack.pop() {
            let (x, y) = (at % width, at / width);
            left = left.min(x);
            right = right.max(x);
            top = top.min(y);
            bottom = bottom.max(y);
            area += 1;
            sum += chances[at];
            let neighbours = [
                (x > 0).then(|| at - 1),
                (x + 1 < width).then(|| at + 1),
                (y > 0).then(|| at - width),
                (y + 1 < height).then(|| at + width),
            ];
            for next in neighbours.into_iter().flatten() {
                if !seen[next] && chances[next] > PIXEL_THRESHOLD {
                    seen[next] = true;
                    stack.push(next);
                }
            }
        }
        let score = sum / area as f32;
        if area < MIN_AREA || score < BOX_THRESHOLD {
            continue;
        }
        let (box_width, box_height) = ((right + 1 - left) as f32, (bottom + 1 - top) as f32);
        let grow = box_width * box_height * UNCLIP / (2.0 * (box_width + box_height));
        let to_picture_x = picture.0 as f32 / width as f32;
        let to_picture_y = picture.1 as f32 / height as f32;
        let text_box = TextBox {
            x1: ((left as f32 - grow) * to_picture_x).clamp(0.0, picture.0 as f32),
            y1: ((top as f32 - grow) * to_picture_y).clamp(0.0, picture.1 as f32),
            x2: (((right + 1) as f32 + grow) * to_picture_x).clamp(0.0, picture.0 as f32),
            y2: (((bottom + 1) as f32 + grow) * to_picture_y).clamp(0.0, picture.1 as f32),
            score,
        };
        if text_box.x2 - text_box.x1 >= 4.0 && text_box.y2 - text_box.y1 >= 4.0 {
            boxes.push(text_box);
        }
    }
    boxes.sort_by(|a, b| a.y1.total_cmp(&b.y1).then(a.x1.total_cmp(&b.x1)));
    boxes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_is_scaled_down_to_960_and_each_side_is_a_multiple_of_32() {
        let rgb = vec![50u8; 1920 * 1080 * 3];
        let input = prepare_detector_input(&rgb, 1920, 1080);
        assert_eq!((input.width, input.height), (960, 544));
        assert_eq!(input.tensor.len(), 3 * 960 * 544);
        let small = prepare_detector_input(&vec![0u8; 100 * 50 * 3], 100, 50);
        assert_eq!((small.width, small.height), (96, 64));
    }

    #[test]
    fn colours_go_in_as_blue_green_red_with_the_mean_and_spread_taken_off() {
        let mut rgb = vec![0u8; 32 * 32 * 3];
        for pixel in 0..32 * 32 {
            rgb[pixel * 3] = 255;
        }
        let input = prepare_detector_input(&rgb, 32, 32);
        let plane = 32 * 32;
        assert!((input.tensor[0] - (0.0 - MEAN[0]) / STD[0]).abs() < 1e-4);
        assert!((input.tensor[2 * plane] - (1.0 - MEAN[2]) / STD[2]).abs() < 1e-4);
    }

    #[test]
    fn a_patch_of_chances_is_a_box_grown_and_put_back_in_picture_pixels_and_noise_is_left_out() {
        let (width, height) = (100, 60);
        let mut chances = vec![0.0f32; width * height];
        for y in 20..30 {
            for x in 10..60 {
                chances[y * width + x] = 0.9;
            }
        }
        chances[5 * width + 90] = 0.95; // one pixel of noise
        for x in 70..90 {
            chances[50 * width + x] = 0.35; // a faint patch
        }
        let boxes = find_text_boxes(&chances, width, height, (200, 120));
        assert_eq!(boxes.len(), 1, "{boxes:?}");
        let found = boxes[0];
        // 50 by 10 grows by 50 * 10 * 1.5 / 120 = 6.25 on each side, then doubles in the picture.
        assert!((found.x1 - (10.0 - 6.25) * 2.0).abs() < 0.01, "{found:?}");
        assert!((found.x2 - (60.0 + 6.25) * 2.0).abs() < 0.01);
        assert!((found.y1 - (20.0 - 6.25) * 2.0).abs() < 0.01);
        assert!(found.score > 0.89);
    }

    #[test]
    fn lines_come_from_the_top_down() {
        let (width, height) = (64, 64);
        let mut chances = vec![0.0f32; width * height];
        for (top, left) in [(40usize, 4usize), (8, 20)] {
            for y in top..top + 8 {
                for x in left..left + 30 {
                    chances[y * width + x] = 0.9;
                }
            }
        }
        let boxes = find_text_boxes(&chances, width, height, (64, 64));
        assert_eq!(boxes.len(), 2);
        assert!(boxes[0].y1 < boxes[1].y1);
    }
}
