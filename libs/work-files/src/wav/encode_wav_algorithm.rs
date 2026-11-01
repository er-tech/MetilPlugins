/// A WAV file (16-bit, one channel) of `samples`, which are between -1 and 1: a header of 44 bytes
/// and the samples as little-endian whole numbers. A sample out of range is held at the end.
pub fn encode_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_bytes = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + samples.len() * 2);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // size of the format chunk
    wav.extend_from_slice(&1u16.to_le_bytes()); // whole-number samples
    wav.extend_from_slice(&1u16.to_le_bytes()); // one channel
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // bytes a second
    wav.extend_from_slice(&2u16.to_le_bytes()); // bytes a sample
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits a sample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        let whole = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        wav.extend_from_slice(&whole.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_and_the_samples_are_what_a_wav_reader_expects() {
        let wav = encode_wav(&[0.0, 1.0, -1.0, 2.0], 16_000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()), 36 + 8);
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 16_000);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 8);
        let samples: Vec<i16> = wav[44..]
            .chunks(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        assert_eq!(
            samples,
            [0, 32767, -32767, 32767],
            "out of range is held at the end"
        );
    }
}
