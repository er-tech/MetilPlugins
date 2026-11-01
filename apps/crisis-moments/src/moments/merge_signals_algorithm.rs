use std::collections::BTreeMap;

use super::signal_contract::{Moment, Signal, Source};

/// How sure the rules are about hints from several inputs: the strongest hint of each input counts
/// once, and the inputs add up as independent evidence (1 minus the chance that all are wrong).
fn confidence_of(signals: &[Signal]) -> f64 {
    let mut best: BTreeMap<Source, f64> = BTreeMap::new();
    for signal in signals {
        let entry = best.entry(signal.source).or_insert(0.0);
        *entry = entry.max(signal.weight);
    }
    let all_wrong: f64 = best.values().map(|weight| 1.0 - weight).product();
    ((1.0 - all_wrong) * 1000.0).round() / 1000.0
}

/// Joins hints that overlap, or are at most `gap_ms` apart, into moments, and keeps those that are
/// at least `min_ms` long and have a confidence of at least `threshold`.
///
/// The reason of a moment is the reason of its strongest hint from words or sounds, or from loudness when there is none. The moments come in time order.
pub fn merge_signals(
    mut signals: Vec<Signal>,
    gap_ms: u64,
    min_ms: u64,
    threshold: f64,
) -> Vec<Moment> {
    signals.sort_by_key(|signal| (signal.start_ms, signal.end_ms));
    let mut groups: Vec<Vec<Signal>> = Vec::new();
    let mut group_end = 0_u64;
    for signal in signals {
        match groups.last_mut() {
            Some(group) if signal.start_ms <= group_end.saturating_add(gap_ms) => {
                group_end = group_end.max(signal.end_ms);
                group.push(signal);
            }
            _ => {
                group_end = signal.end_ms;
                groups.push(vec![signal]);
            }
        }
    }

    groups
        .into_iter()
        .filter_map(|mut group| {
            let start_ms = group.iter().map(|signal| signal.start_ms).min()?;
            let end_ms = group.iter().map(|signal| signal.end_ms).max()?;
            let confidence = confidence_of(&group);
            if end_ms - start_ms < min_ms || confidence < threshold {
                return None;
            }
            // Loudness says little about why, so a hint from words or sounds names the moment first.
            group.sort_by(|a, b| {
                (a.source == Source::Volume)
                    .cmp(&(b.source == Source::Volume))
                    .then(b.weight.total_cmp(&a.weight))
            });
            let reason = group[0].reason.clone();
            Some(Moment {
                start_ms,
                end_ms,
                reason,
                confidence,
                signals: group,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(source: Source, start_ms: u64, end_ms: u64, weight: f64, reason: &str) -> Signal {
        Signal {
            start_ms,
            end_ms,
            source,
            reason: reason.to_owned(),
            weight,
            detail: String::new(),
        }
    }

    #[test]
    fn hints_that_overlap_become_one_moment_and_agreeing_inputs_raise_the_confidence() {
        let moments = merge_signals(
            vec![
                signal(Source::Labels, 1_000, 3_000, 0.7, "shouting"),
                signal(Source::Volume, 2_000, 3_000, 0.5, "sudden loud sound"),
            ],
            0,
            0,
            0.0,
        );
        assert_eq!(moments.len(), 1);
        assert_eq!((moments[0].start_ms, moments[0].end_ms), (1_000, 3_000));
        assert_eq!(moments[0].reason, "shouting");
        assert!((moments[0].confidence - 0.85).abs() < 1e-9);
    }

    #[test]
    fn loudness_never_names_a_moment_that_words_or_sounds_also_flag() {
        let moments = merge_signals(
            vec![
                signal(Source::Volume, 0, 1_000, 1.0, "sudden loud sound"),
                signal(Source::Text, 0, 1_000, 0.6, "words of alarm"),
            ],
            0,
            0,
            0.0,
        );
        assert_eq!(moments[0].reason, "words of alarm");
    }

    #[test]
    fn two_hints_of_one_input_do_not_add_up() {
        let moments = merge_signals(
            vec![
                signal(Source::Labels, 0, 1_000, 0.6, "alarm"),
                signal(Source::Labels, 500, 1_500, 0.6, "alarm"),
            ],
            0,
            0,
            0.0,
        );
        assert!((moments[0].confidence - 0.6).abs() < 1e-9);
    }

    #[test]
    fn the_gap_joins_near_hints_and_a_larger_distance_keeps_them_apart() {
        let hints = || {
            vec![
                signal(Source::Text, 0, 1_000, 0.6, "a"),
                signal(Source::Text, 2_000, 3_000, 0.6, "b"),
            ]
        };
        assert_eq!(merge_signals(hints(), 1_000, 0, 0.0).len(), 1);
        assert_eq!(merge_signals(hints(), 999, 0, 0.0).len(), 2);
    }

    #[test]
    fn weak_and_short_moments_are_dropped() {
        let hints = vec![
            signal(Source::Text, 0, 400, 0.6, "a"),
            signal(Source::Text, 10_000, 12_000, 0.3, "b"),
        ];
        assert!(merge_signals(hints.clone(), 0, 500, 0.5).is_empty());
        assert_eq!(merge_signals(hints, 0, 0, 0.5).len(), 1);
    }

    #[test]
    fn no_hints_make_no_moments() {
        assert!(merge_signals(Vec::new(), 1_000, 0, 0.0).is_empty());
    }
}
