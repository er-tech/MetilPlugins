use super::signal_contract::{Signal, Source};
use super::timed_item_contract::TimedItem;

/// How strong a label hint is when the finder gave no confidence.
const DEFAULT_WEIGHT: f64 = 0.5;

/// The short reason for a sound name, such as "shouting" for "Screaming".
fn reason_of(name: &str) -> &'static str {
    let name = name.to_lowercase();
    let has = |parts: &[&str]| parts.iter().any(|part| name.contains(part));
    if has(&["scream", "shout", "yell"]) {
        "shouting"
    } else if has(&["cry", "sob", "whimper"]) {
        "crying"
    } else if has(&["siren", "alarm"]) {
        "alarm"
    } else if has(&[
        "glass",
        "shatter",
        "explosion",
        "gunshot",
        "gunfire",
        "bang",
    ]) {
        "loud impact"
    } else {
        "alarming sound"
    }
}

/// Flags each found sound or thing whose label holds one of the sound names, ignoring case.
///
/// The strength is the confidence of the finder, or 0.5 when it gave none. Items without `label`
/// are skipped. A label matches the first name in the list that it holds.
pub fn label_signals(found: &[TimedItem], sound_names: &[String]) -> Vec<Signal> {
    let names: Vec<(&String, String)> = sound_names
        .iter()
        .map(|name| (name, name.to_lowercase()))
        .filter(|(_, lower)| !lower.is_empty())
        .collect();

    found
        .iter()
        .filter_map(|item| {
            let label = item.label.as_deref()?;
            let lower = label.to_lowercase();
            names
                .iter()
                .find(|(_, name)| lower.contains(name.as_str()))
                .map(|(name, _)| Signal {
                    start_ms: item.start_ms,
                    end_ms: item.end_ms,
                    source: Source::Labels,
                    reason: reason_of(name).to_owned(),
                    weight: item.confidence.unwrap_or(DEFAULT_WEIGHT).clamp(0.0, 1.0),
                    detail: format!("heard {label}"),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(label: &str, confidence: Option<f64>) -> TimedItem {
        TimedItem {
            start_ms: 500,
            end_ms: 1_500,
            text: None,
            label: Some(label.to_owned()),
            confidence,
            value: None,
        }
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn a_matching_label_keeps_its_confidence_and_gets_a_short_reason() {
        let signals = label_signals(&[found("Screaming", Some(0.8))], &names(&["scream"]));
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].reason, "shouting");
        assert!((signals[0].weight - 0.8).abs() < 1e-9);
        assert_eq!(signals[0].detail, "heard Screaming");
    }

    #[test]
    fn the_match_ignores_case_and_works_inside_a_longer_name() {
        let signals = label_signals(
            &[found("Smoke detector, smoke ALARM", None)],
            &names(&["alarm"]),
        );
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].reason, "alarm");
        assert!((signals[0].weight - 0.5).abs() < 1e-9);
    }

    #[test]
    fn a_label_not_in_the_list_is_ignored() {
        assert!(label_signals(&[found("Cat", Some(0.9))], &names(&["scream"])).is_empty());
    }

    #[test]
    fn a_name_the_rules_do_not_know_gets_the_general_reason() {
        let signals = label_signals(&[found("Thunder", Some(0.9))], &names(&["thunder"]));
        assert_eq!(signals[0].reason, "alarming sound");
    }
}
