/// The lowest and highest sample under one column of the picture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Column {
    pub min: f32,
    pub max: f32,
}

/// Divides the sound into `width` columns of equal length and finds the lowest and highest sample
/// in each, so the picture shows the shape of the waves however long the sound is. A sound with
/// fewer samples than columns repeats its samples; an empty sound gives flat columns.
pub fn compute_columns(samples: &[f32], width: usize) -> Vec<Column> {
    if samples.is_empty() {
        return vec![Column { min: 0.0, max: 0.0 }; width];
    }
    (0..width)
        .map(|column| {
            let start = column * samples.len() / width;
            let end = ((column + 1) * samples.len() / width)
                .max(start + 1)
                .min(samples.len());
            let part =
                &samples[start.min(samples.len() - 1)..end.max(start.min(samples.len() - 1) + 1)];
            Column {
                min: part.iter().copied().fold(f32::INFINITY, f32::min),
                max: part.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            }
        })
        .collect()
}

/// The highest sample level (ignoring the sign) in the columns: 0 for silence.
pub fn peak_of(columns: &[Column]) -> f32 {
    columns.iter().fold(0.0_f32, |peak, column| {
        peak.max(column.max.abs()).max(column.min.abs())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_column_holds_the_lowest_and_the_highest_of_its_part() {
        let samples = [0.1, 0.5, -0.2, 0.3, -0.9, 0.0, 0.2, 0.4];
        let columns = compute_columns(&samples, 4);
        assert_eq!(columns.len(), 4);
        assert_eq!(columns[0], Column { min: 0.1, max: 0.5 });
        assert_eq!(
            columns[1],
            Column {
                min: -0.2,
                max: 0.3
            }
        );
        assert_eq!(
            columns[2],
            Column {
                min: -0.9,
                max: 0.0
            }
        );
        assert_eq!(columns[3], Column { min: 0.2, max: 0.4 });
        assert!((peak_of(&columns) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn a_short_sound_repeats_its_samples_and_an_empty_one_is_flat() {
        let columns = compute_columns(&[0.5, -0.5], 5);
        assert_eq!(columns.len(), 5);
        assert!(columns.iter().all(|c| c.min <= c.max));
        assert_eq!(columns[0].max, 0.5);
        assert_eq!(columns[4].min, -0.5);
        assert_eq!(
            compute_columns(&[], 3),
            vec![Column { min: 0.0, max: 0.0 }; 3]
        );
        assert_eq!(peak_of(&compute_columns(&[], 3)), 0.0);
    }

    #[test]
    fn every_sample_belongs_to_a_column_whatever_the_division() {
        let samples: Vec<f32> = (0..1000).map(|n| n as f32).collect();
        let columns = compute_columns(&samples, 7);
        assert_eq!(columns[0].min, 0.0);
        assert_eq!(columns[6].max, 999.0);
        for pair in columns.windows(2) {
            // The next column starts right where this one ended.
            assert_eq!(pair[1].min, pair[0].max + 1.0);
        }
    }
}
