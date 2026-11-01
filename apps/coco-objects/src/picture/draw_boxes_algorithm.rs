use crate::detect::Detection;

/// Colours for the boxes, taken by the number of the class so a class keeps its colour.
const COLOURS: [[u8; 3]; 6] = [
    [255, 64, 64],
    [64, 220, 64],
    [64, 128, 255],
    [255, 200, 0],
    [220, 64, 255],
    [0, 220, 220],
];

/// Draws the outline of each box on an `rgb24` picture, `thickness` pixels wide and inside the
/// box, so the thing is not covered.
pub fn draw_boxes(
    rgb: &mut [u8],
    width: usize,
    height: usize,
    boxes: &[Detection],
    thickness: usize,
) {
    for found in boxes {
        let colour = COLOURS[found.class % COLOURS.len()];
        let x1 = (found.x1.max(0.0) as usize).min(width.saturating_sub(1));
        let y1 = (found.y1.max(0.0) as usize).min(height.saturating_sub(1));
        let x2 = (found.x2.max(0.0) as usize).min(width.saturating_sub(1));
        let y2 = (found.y2.max(0.0) as usize).min(height.saturating_sub(1));
        for y in y1..=y2 {
            for x in x1..=x2 {
                let on_edge = x < x1 + thickness
                    || x + thickness > x2
                    || y < y1 + thickness
                    || y + thickness > y2;
                if on_edge {
                    let at = (y * width + x) * 3;
                    rgb[at..at + 3].copy_from_slice(&colour);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_outline_is_drawn_inside_the_box_and_the_inside_is_left_alone() {
        let mut rgb = vec![0u8; 10 * 10 * 3];
        let found = Detection {
            class: 0,
            score: 0.9,
            x1: 2.0,
            y1: 2.0,
            x2: 7.0,
            y2: 7.0,
        };
        draw_boxes(&mut rgb, 10, 10, &[found], 1);
        let pixel = |x: usize, y: usize| rgb[(y * 10 + x) * 3..(y * 10 + x) * 3 + 3].to_vec();
        assert_eq!(pixel(2, 2), COLOURS[0]);
        assert_eq!(pixel(7, 4), COLOURS[0]);
        assert_eq!(pixel(4, 4), [0, 0, 0], "the inside is not covered");
        assert_eq!(pixel(0, 0), [0, 0, 0], "nor is the outside");
    }

    #[test]
    fn a_box_off_the_picture_does_not_panic() {
        let mut rgb = vec![0u8; 4 * 4 * 3];
        let found = Detection {
            class: 1,
            score: 0.9,
            x1: -5.0,
            y1: -5.0,
            x2: 50.0,
            y2: 50.0,
        };
        draw_boxes(&mut rgb, 4, 4, &[found], 2);
        assert!(rgb.iter().any(|value| *value != 0));
    }
}
