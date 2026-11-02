/// The positions of `count` pictures spread evenly through `available` ones, first to last, in
/// order. All of them when there are no more than `count`; none when there are none.
pub fn choose_pictures(available: usize, count: usize) -> Vec<usize> {
    if available == 0 || count == 0 {
        return Vec::new();
    }
    if available <= count {
        return (0..available).collect();
    }
    if count == 1 {
        return vec![available / 2];
    }
    (0..count)
        .map(|n| n * (available - 1) / (count - 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_the_pictures_are_taken_when_there_are_not_more_than_asked_for() {
        assert_eq!(choose_pictures(3, 12), vec![0, 1, 2]);
        assert_eq!(choose_pictures(4, 4), vec![0, 1, 2, 3]);
        assert!(choose_pictures(0, 5).is_empty());
        assert!(choose_pictures(5, 0).is_empty());
    }

    #[test]
    fn the_pictures_are_spread_from_the_first_to_the_last() {
        assert_eq!(choose_pictures(100, 5), vec![0, 24, 49, 74, 99]);
        assert_eq!(choose_pictures(10, 2), vec![0, 9]);
    }

    #[test]
    fn one_picture_is_the_middle_one() {
        assert_eq!(choose_pictures(9, 1), vec![4]);
    }
}
