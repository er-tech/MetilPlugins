use super::tables_contract::{Tables, BANDS, BINS, FFT_SIZE};

/// The tables from the bytes of `deepfilter-auxiliary.bin`: little-endian 32-bit numbers, the
/// matrix from frequencies to bands (481 x 32), the one from bands back to frequencies (32 x 481),
/// and the window (960). Anything of another size is an error that says what was found.
pub fn read_tables(bytes: &[u8]) -> Result<Tables, String> {
    let wanted = (BINS * BANDS * 2 + FFT_SIZE) * 4;
    if bytes.len() != wanted {
        return Err(format!(
            "the tables are {} bytes, and {wanted} were expected",
            bytes.len()
        ));
    }
    let numbers: Vec<f32> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|four| f32::from_le_bytes(*four))
        .collect();
    let (forward, rest) = numbers.split_at(BINS * BANDS);
    let (inverse, window) = rest.split_at(BANDS * BINS);

    let bands = (0..BANDS)
        .map(|band| {
            (0..BINS)
                .map(|bin| (bin, forward[bin * BANDS + band]))
                .filter(|(_, weight)| *weight != 0.0)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let spread = (0..BANDS)
        .map(|band| {
            (0..BINS)
                .map(|bin| (bin, inverse[band * BINS + bin]))
                .filter(|(_, weight)| *weight != 0.0)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if bands.iter().chain(&spread).any(Vec::is_empty) {
        return Err("a band of the tables has no frequency in it".to_string());
    }
    Ok(Tables {
        bands,
        spread,
        window: window.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tables for bands of one frequency each in the first 32 bins, and a flat window.
    fn bytes() -> Vec<u8> {
        let mut numbers = vec![0f32; BINS * BANDS * 2 + FFT_SIZE];
        for band in 0..BANDS {
            numbers[band * BANDS + band] = 1.0; // forward: bin `band` is band `band`
            numbers[BINS * BANDS + band * BINS + band] = 1.0; // inverse
        }
        numbers[BINS * BANDS * 2..].fill(0.5);
        numbers.iter().flat_map(|n| n.to_le_bytes()).collect()
    }

    #[test]
    fn the_matrices_become_the_frequencies_of_each_band_and_the_window_is_kept() {
        let tables = read_tables(&bytes()).unwrap();
        assert_eq!(tables.bands.len(), BANDS);
        assert_eq!(tables.bands[3], vec![(3, 1.0)]);
        assert_eq!(tables.spread[31], vec![(31, 1.0)]);
        assert_eq!(tables.window.len(), FFT_SIZE);
        assert!(tables.window.iter().all(|w| *w == 0.5));
    }

    #[test]
    fn a_file_of_the_wrong_size_and_an_empty_band_are_errors() {
        assert!(read_tables(&[0u8; 10]).unwrap_err().contains("10 bytes"));
        let empty = vec![0u8; (BINS * BANDS * 2 + FFT_SIZE) * 4];
        assert!(read_tables(&empty).unwrap_err().contains("no frequency"));
    }
}
