/// How much the model's scores are stretched: `exp(4.764997)`, the `logit_scale` of
/// `google/siglip-base-patch16-224`.
const SCALE: f32 = 117.330_77;
/// Where the scale starts from: the `logit_bias` of the same model.
const BIAS: f32 = -12.932_437;

/// The chance, from 0 to 1, that a phrase describes a picture, from their numbers (each scaled to
/// length 1). SigLIP scores each phrase on its own with a sigmoid, so "is there a cat" can be asked
/// without other phrases to compete with. The numbers are low for short phrases: a good match is
/// often 0.2 to 0.6, and a phrase that is wrong is near 0.
pub fn match_probability(picture: &[f32], phrase: &[f32]) -> f32 {
    let alike: f32 = picture.iter().zip(phrase).map(|(a, b)| a * b).sum();
    1.0 / (1.0 + (-(SCALE * alike + BIAS)).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_numbers_are_a_near_certain_match_and_opposite_ones_are_none() {
        let a = [1.0, 0.0];
        assert!(match_probability(&a, &a) > 0.99);
        assert!(match_probability(&a, &[-1.0, 0.0]) < 0.001);
    }

    #[test]
    fn the_probability_grows_with_how_alike_they_are() {
        let at =
            |alike: f32| match_probability(&[1.0, 0.0], &[alike, (1.0 - alike * alike).sqrt()]);
        assert!(at(0.05) < at(0.10) && at(0.10) < at(0.15));
        // The bias puts half at a cosine of 12.93 / 117.33 = 0.11.
        assert!((at(0.1102) - 0.5).abs() < 0.02);
    }
}
