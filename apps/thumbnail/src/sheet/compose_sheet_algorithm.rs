/// The colour of the space between the pictures and round them (rgb).
const BACKGROUND: [u8; 3] = [16, 16, 16];

/// A contact sheet: `rgb24` pixels, the size of the whole picture and how many rows it has.
#[derive(Debug, PartialEq, Eq)]
pub struct Sheet {
    pub rgb: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub columns: usize,
    pub rows: usize,
}

/// Lays `pictures` out in `columns` columns (never more than there are pictures), row by row,
/// with `gap` pixels between them and round the edge. Each picture is `width` x `height` pixels
/// of `rgb24`; a missing picture at the end of the last row stays background. A picture of another
/// size is the caller's mistake and panics.
pub fn compose_sheet(
    pictures: &[&[u8]],
    width: usize,
    height: usize,
    columns: usize,
    gap: usize,
) -> Sheet {
    let columns = columns.clamp(1, pictures.len().max(1));
    let rows = pictures.len().div_ceil(columns).max(1);
    let sheet_width = columns * width + (columns + 1) * gap;
    let sheet_height = rows * height + (rows + 1) * gap;
    let mut rgb: Vec<u8> = BACKGROUND
        .iter()
        .copied()
        .cycle()
        .take(sheet_width * sheet_height * 3)
        .collect();
    for (index, picture) in pictures.iter().enumerate() {
        assert_eq!(
            picture.len(),
            width * height * 3,
            "a picture of another size"
        );
        let left = gap + (index % columns) * (width + gap);
        let top = gap + (index / columns) * (height + gap);
        for line in 0..height {
            let from = line * width * 3;
            let to = ((top + line) * sheet_width + left) * 3;
            rgb[to..to + width * 3].copy_from_slice(&picture[from..from + width * 3]);
        }
    }
    Sheet {
        rgb,
        width: sheet_width,
        height: sheet_height,
        columns,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture(value: u8) -> Vec<u8> {
        vec![value; 2 * 2 * 3]
    }

    #[test]
    fn the_pictures_go_row_by_row_with_the_gap_between_and_round_them() {
        let (a, b, c) = (picture(200), picture(100), picture(50));
        let sheet = compose_sheet(&[&a, &b, &c], 2, 2, 2, 1);
        assert_eq!((sheet.width, sheet.height, sheet.rows), (7, 7, 2));
        let pixel = |x: usize, y: usize| sheet.rgb[(y * sheet.width + x) * 3];
        assert_eq!(pixel(0, 0), 16, "the border is background");
        assert_eq!(pixel(1, 1), 200, "the first picture starts after the gap");
        assert_eq!(pixel(4, 1), 100, "the second is in the second column");
        assert_eq!(pixel(1, 4), 50, "the third starts the second row");
        assert_eq!(
            pixel(4, 4),
            16,
            "the place of a missing picture stays background"
        );
    }

    #[test]
    fn more_columns_than_pictures_make_one_row_as_wide_as_the_pictures() {
        let (a, b) = (picture(9), picture(8));
        let sheet = compose_sheet(&[&a, &b], 2, 2, 12, 0);
        assert_eq!(
            (sheet.width, sheet.height, sheet.columns, sheet.rows),
            (4, 2, 2, 1)
        );
    }

    #[test]
    #[should_panic(expected = "another size")]
    fn a_picture_of_another_size_is_refused() {
        let wrong = vec![0u8; 5];
        compose_sheet(&[&wrong], 2, 2, 1, 0);
    }
}
