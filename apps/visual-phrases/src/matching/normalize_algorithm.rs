/// Scales `numbers` to length 1 (the model's numbers are compared by the angle between them, not by
/// how large they are). A row of zeros is left as it is.
pub fn normalize(numbers: &mut [f32]) {
    let length = numbers.iter().map(|n| n * n).sum::<f32>().sqrt();
    if length > 0.0 {
        numbers.iter_mut().for_each(|n| *n /= length);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_gets_length_one_and_a_row_of_zeros_stays() {
        let mut row = [3.0, 4.0];
        normalize(&mut row);
        assert!((row[0] - 0.6).abs() < 1e-6 && (row[1] - 0.8).abs() < 1e-6);
        let mut zeros = [0.0, 0.0];
        normalize(&mut zeros);
        assert_eq!(zeros, [0.0, 0.0]);
    }
}
