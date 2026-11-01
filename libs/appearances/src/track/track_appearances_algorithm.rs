use std::collections::BTreeMap;

use super::appearance_contract::{Appearance, Sample};

/// Joins what was found sample by sample into appearances, for each key. A thing that is out of the
/// samples for more than `gap_ms` (counted from the last sample it was in) has left, and when it
/// comes back that is a new appearance. A stretch seen in fewer than `min_samples` samples was a
/// flicker and is dropped.
///
/// `samples` must be in time order. The appearances come back in the order they start.
pub fn track_appearances(samples: &[Sample], gap_ms: u64, min_samples: usize) -> Vec<Appearance> {
    let mut open: BTreeMap<usize, (Appearance, u64)> = BTreeMap::new();
    let mut done: Vec<Appearance> = Vec::new();
    for sample in samples {
        let mut keys: Vec<usize> = sample.present.iter().map(|found| found.key).collect();
        keys.sort_unstable();
        keys.dedup();
        for key in keys {
            let here: Vec<f32> = sample
                .present
                .iter()
                .filter(|found| found.key == key)
                .map(|found| found.score)
                .collect();
            let best = here.iter().copied().fold(0.0, f32::max);
            let end_ms = sample.time_ms + sample.duration_ms;
            if let Some((current, last_seen)) = open.get_mut(&key) {
                if sample.time_ms.saturating_sub(*last_seen) <= gap_ms {
                    current.end_ms = end_ms;
                    current.confidence = current.confidence.max(best);
                    current.peak_count = current.peak_count.max(here.len());
                    current.samples += 1;
                    *last_seen = sample.time_ms;
                    continue;
                }
                done.push(*current);
            }
            open.insert(
                key,
                (
                    Appearance {
                        key,
                        first_frame: sample.frame,
                        start_ms: sample.time_ms,
                        end_ms,
                        confidence: best,
                        peak_count: here.len(),
                        samples: 1,
                    },
                    sample.time_ms,
                ),
            );
        }
    }
    done.extend(open.into_values().map(|(appearance, _)| appearance));
    done.retain(|appearance| appearance.samples >= min_samples);
    done.sort_by_key(|appearance| (appearance.start_ms, appearance.key));
    done
}

#[cfg(test)]
mod tests {
    use super::super::appearance_contract::Presence;
    use super::*;

    /// A sample every 500 ms, with the (key, score) pairs found in each.
    fn timeline(samples: &[&[(usize, f32)]]) -> Vec<Sample> {
        samples
            .iter()
            .enumerate()
            .map(|(index, found)| Sample {
                frame: index,
                time_ms: index as u64 * 500,
                duration_ms: 500,
                present: found
                    .iter()
                    .map(|(key, score)| Presence {
                        key: *key,
                        score: *score,
                    })
                    .collect(),
            })
            .collect()
    }

    #[test]
    fn a_thing_that_stays_is_one_appearance_that_starts_at_its_first_sample() {
        let samples = timeline(&[
            &[],
            &[(15, 0.6)],
            &[(15, 0.9)],
            &[(15, 0.7), (15, 0.5)],
            &[],
        ]);
        let found = track_appearances(&samples, 2000, 1);

        assert_eq!(found.len(), 1);
        let cat = found[0];
        assert_eq!((cat.key, cat.first_frame, cat.start_ms), (15, 1, 500));
        assert_eq!(cat.end_ms, 2000, "the last sample it is in ends at 2000");
        assert_eq!((cat.samples, cat.peak_count), (3, 2));
        assert_eq!(cat.confidence, 0.9);
    }

    #[test]
    fn a_thing_that_leaves_for_long_and_comes_back_is_a_second_appearance() {
        // In at 0, gone from 500 to 2500 (more than 1000), back at 3000.
        let samples = timeline(&[&[(15, 0.8)], &[], &[], &[], &[], &[], &[(15, 0.8)]]);
        let found = track_appearances(&samples, 1000, 1);
        let starts: Vec<u64> = found.iter().map(|a| a.start_ms).collect();
        assert_eq!(starts, [0, 3000]);

        // With a longer gap the same video is one appearance.
        assert_eq!(track_appearances(&samples, 5000, 1).len(), 1);
    }

    #[test]
    fn a_flicker_is_dropped_and_each_key_is_followed_on_its_own() {
        let samples = timeline(&[&[(15, 0.8), (0, 0.9)], &[(0, 0.9)], &[(0, 0.9)]]);
        let found = track_appearances(&samples, 2000, 2);
        assert_eq!(found.len(), 1, "key 15 was in one sample only");
        assert_eq!(found[0].key, 0);
        assert_eq!(found[0].samples, 3);
    }

    #[test]
    fn nothing_found_gives_no_appearances() {
        assert!(track_appearances(&timeline(&[&[], &[]]), 2000, 1).is_empty());
        assert!(track_appearances(&[], 2000, 1).is_empty());
    }
}
