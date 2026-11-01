use super::detection_contract::Detection;

/// The detector looks at the picture on three grids, each cell a square of this many pixels.
const STRIDES: [usize; 3] = [8, 16, 32];
/// A row of its output: the centre and size of a box, how sure it is there is something, and a
/// score for each of the 80 classes.
const FIELDS: usize = 5 + 80;

/// How many rows (candidate boxes) the detector gives for a square of `size`.
pub fn candidate_rows(size: usize) -> usize {
    STRIDES.iter().map(|stride| (size / stride).pow(2)).sum()
}

/// Turns what the detector gave (`output`, rows of 85 values) into boxes in pixels of the original
/// picture, keeping those whose best class is one of the `wanted` and whose score (how sure there
/// is something, times how sure of the class) reaches `min_score`.
///
/// The centre and size of a box are relative to a cell of the grid, so the cell's place and the
/// stride put them back in pixels of the square, and `ratio` (how much the picture was shrunk to
/// fit it) in pixels of the picture.
pub fn decode_output(
    output: &[f32],
    size: usize,
    ratio: f32,
    min_score: f32,
    wanted: &[bool; 80],
) -> Result<Vec<Detection>, String> {
    let rows = candidate_rows(size);
    if output.len() != rows * FIELDS {
        return Err(format!(
            "the model gave {} values, and {} were expected for a picture of {size}",
            output.len(),
            rows * FIELDS
        ));
    }
    let mut found = Vec::new();
    let mut row = 0;
    for stride in STRIDES {
        let cells = size / stride;
        for grid_y in 0..cells {
            for grid_x in 0..cells {
                let values = &output[row * FIELDS..(row + 1) * FIELDS];
                row += 1;
                let (class, class_score) =
                    values[5..]
                        .iter()
                        .enumerate()
                        .fold((0, f32::MIN), |best, (class, score)| {
                            if *score > best.1 {
                                (class, *score)
                            } else {
                                best
                            }
                        });
                let score = values[4] * class_score;
                if score < min_score || !wanted[class] {
                    continue;
                }
                let stride = stride as f32;
                let centre_x = (values[0] + grid_x as f32) * stride;
                let centre_y = (values[1] + grid_y as f32) * stride;
                let half_width = values[2].exp() * stride / 2.0;
                let half_height = values[3].exp() * stride / 2.0;
                found.push(Detection {
                    class,
                    score,
                    x1: (centre_x - half_width) / ratio,
                    y1: (centre_y - half_height) / ratio,
                    x2: (centre_x + half_width) / ratio,
                    y2: (centre_y + half_height) / ratio,
                });
            }
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The output of a detector for a square of 32: one cell of 32, four of 16 and sixteen of 8.
    fn blank(size: usize) -> Vec<f32> {
        vec![0.0; candidate_rows(size) * FIELDS]
    }

    #[test]
    fn a_confident_cell_becomes_a_box_in_pixels_of_the_picture() {
        let mut output = blank(32);
        // The first rows are the grid of 8: row 5 is the cell at x = 1, y = 1 (4 cells a side).
        let row = 5 * FIELDS;
        output[row] = 0.5; // centre x inside the cell
        output[row + 1] = 0.5;
        output[row + 2] = 0.0; // width = exp(0) * 8 = 8
        output[row + 3] = 0.0; // height = exp(0) * 8 = 8
        output[row + 4] = 0.9; // something is there
        output[row + 5 + 15] = 0.8; // a cat

        let wanted = [true; 80];
        let found = decode_output(&output, 32, 0.5, 0.5, &wanted).unwrap();

        assert_eq!(found.len(), 1);
        let cat = found[0];
        assert_eq!(cat.class, 15);
        assert!((cat.score - 0.72).abs() < 1e-5);
        // Centre (0.5 + 1) * 8 = 12 in the square, 24 in the picture; the box is 8 in the square,
        // 16 in the picture, so y goes from 16 to 32.
        assert!((cat.y1 - 16.0).abs() < 1e-4 && (cat.y2 - 32.0).abs() < 1e-4);
        assert!(((cat.x1 + cat.x2) / 2.0 - 24.0).abs() < 1e-4);
    }

    #[test]
    fn a_class_nobody_asked_for_and_a_weak_score_are_dropped() {
        let mut output = blank(32);
        output[4] = 0.9;
        output[5 + 15] = 0.8;
        let mut dogs_only = [false; 80];
        dogs_only[16] = true;
        assert!(decode_output(&output, 32, 1.0, 0.5, &dogs_only)
            .unwrap()
            .is_empty());
        assert!(decode_output(&output, 32, 1.0, 0.9, &[true; 80])
            .unwrap()
            .is_empty());
        assert_eq!(
            decode_output(&output, 32, 1.0, 0.7, &[true; 80])
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn an_output_of_the_wrong_length_is_an_error_that_says_so() {
        let error = decode_output(&[0.0; 10], 32, 1.0, 0.5, &[true; 80]).unwrap_err();
        assert!(error.contains("10 values"), "{error}");
    }
}
