/// Adds the frames of an interleaved sound (`channels` samples per frame) to `mono`, each as the
/// average of its channels.
pub fn mix_to_mono(interleaved: &[f32], channels: usize, mono: &mut Vec<f32>) {
    if channels <= 1 {
        mono.extend_from_slice(interleaved);
        return;
    }
    let scale = 1.0 / channels as f32;
    mono.extend(
        interleaved
            .chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() * scale),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_channel_is_kept_and_several_are_averaged_frame_by_frame() {
        let mut mono = Vec::new();
        mix_to_mono(&[0.5, -0.5], 1, &mut mono);
        assert_eq!(mono, [0.5, -0.5]);

        let mut stereo = vec![9.0];
        mix_to_mono(&[1.0, 0.0, 0.5, 0.5, -1.0, 1.0], 2, &mut stereo);
        assert_eq!(stereo, [9.0, 0.5, 0.5, 0.0]);

        // A last frame that is not whole is dropped.
        let mut cut = Vec::new();
        mix_to_mono(&[1.0, 1.0, 1.0], 2, &mut cut);
        assert_eq!(cut, [1.0]);
    }
}
