/// The pixels each piece shares with its neighbours on each side, in the picture before it grows.
/// The network sees them but its answer for them is dropped, so no seam shows where pieces meet.
pub const TILE_MARGIN: usize = 16;

/// How many pieces cover a side of `length` pixels when each piece keeps `step` pixels of it.
/// Piece `k` keeps the pixels from `k * step` and sees `TILE_MARGIN` more on each side.
pub fn tiles_along(length: usize, step: usize) -> usize {
    length.div_ceil(step).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pieces_cover_the_side_and_one_is_enough_for_a_small_side() {
        assert_eq!(tiles_along(96, 96), 1);
        assert_eq!(tiles_along(97, 96), 2);
        assert_eq!(tiles_along(1920, 96), 20);
        assert_eq!(tiles_along(10, 96), 1);
    }
}
