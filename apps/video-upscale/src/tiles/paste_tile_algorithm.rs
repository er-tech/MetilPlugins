/// Where the answer for one piece goes in the larger picture.
#[derive(Debug, Clone, Copy)]
pub struct Placement {
    /// The side of the piece the network saw, in pixels of the picture before it grew.
    pub tile: usize,
    /// The pixels on each side of the piece that the answer drops.
    pub margin: usize,
    /// How many times larger the answer is than the piece.
    pub scale: usize,
    /// Where the part the piece keeps starts, in pixels of the picture before it grew.
    pub left: usize,
    pub top: usize,
}

/// Writes the kept part of `answer` (three planes of numbers from 0 to 1, `tile * scale` on a side)
/// into `output`, an `rgb24` picture of `out_width` x `out_height`. The kept part is the answer
/// without its margin, and what falls outside the picture is left out.
pub fn paste_tile(
    output: &mut [u8],
    out_width: usize,
    out_height: usize,
    answer: &[f32],
    place: &Placement,
) {
    let side = place.tile * place.scale;
    let plane = side * side;
    let kept = (place.tile - 2 * place.margin) * place.scale;
    let skipped = place.margin * place.scale;
    for row in 0..kept {
        let y = place.top * place.scale + row;
        if y >= out_height {
            break;
        }
        for column in 0..kept {
            let x = place.left * place.scale + column;
            if x >= out_width {
                break;
            }
            let from = (skipped + row) * side + skipped + column;
            for channel in 0..3 {
                let value = answer[channel * plane + from].clamp(0.0, 1.0);
                output[(y * out_width + x) * 3 + channel] = (value * 255.0 + 0.5) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::cut_tile_algorithm::cut_tile;
    use super::super::plan_tiles_algorithm::tiles_along;
    use super::*;

    /// What a network that only repeats each pixel `scale` times answers for a piece.
    fn repeat_pixels(piece: &[f32], tile: usize, scale: usize) -> Vec<f32> {
        let side = tile * scale;
        let mut answer = vec![0f32; 3 * side * side];
        for channel in 0..3 {
            for y in 0..side {
                for x in 0..side {
                    answer[channel * side * side + y * side + x] =
                        piece[channel * tile * tile + (y / scale) * tile + x / scale];
                }
            }
        }
        answer
    }

    #[test]
    fn the_pieces_put_back_together_equal_the_picture_repeated_with_no_seam() {
        let (width, height, tile, margin, scale) = (50usize, 37usize, 16usize, 4usize, 4usize);
        let rgb: Vec<u8> = (0..width * height * 3)
            .map(|n| (n * 7 % 256) as u8)
            .collect();
        let step = tile - 2 * margin;
        let mut output = vec![0u8; width * scale * height * scale * 3];
        for row in 0..tiles_along(height, step) {
            for column in 0..tiles_along(width, step) {
                let (left, top) = (column * step, row * step);
                let piece = cut_tile(
                    &rgb,
                    width,
                    height,
                    left as isize - margin as isize,
                    top as isize - margin as isize,
                    tile,
                );
                let answer = repeat_pixels(&piece, tile, scale);
                let place = Placement {
                    tile,
                    margin,
                    scale,
                    left,
                    top,
                };
                paste_tile(&mut output, width * scale, height * scale, &answer, &place);
            }
        }
        for y in 0..height * scale {
            for x in 0..width * scale {
                for channel in 0..3 {
                    assert_eq!(
                        output[(y * width * scale + x) * 3 + channel],
                        rgb[((y / scale) * width + x / scale) * 3 + channel],
                        "at {x},{y}"
                    );
                }
            }
        }
    }
}
