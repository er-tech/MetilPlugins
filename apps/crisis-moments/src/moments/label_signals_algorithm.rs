use super::signal_contract::{Signal, Source};
use super::timed_item_contract::TimedItem;

/// How strong a label hint is when the finder gave no confidence.
const DEFAULT_WEIGHT: f64 = 0.5;

/// What a sound name stands for: the short reason written for it, and how much of the finder's
/// confidence it keeps. Sounds that many harmless things make (a thump, a gasp) keep less.
struct Kind {
    reason: &'static str,
    keeps: f64,
}

/// The kind of a sound name, such as "screaming" for "Screaming". A name the rules do not know is
/// a "sound of distress" that keeps most of the confidence of the finder.
fn kind_of(name: &str) -> Kind {
    let name = name.to_lowercase();
    let has = |parts: &[&str]| parts.iter().any(|part| name.contains(part));
    if has(&["scream"]) {
        Kind {
            reason: "screaming",
            keeps: 1.0,
        }
    } else if has(&["shout", "yell", "bellow"]) {
        Kind {
            reason: "shouting",
            keeps: 1.0,
        }
    } else if has(&["cry", "sob", "whimper", "wail", "moan"]) {
        Kind {
            reason: "crying",
            keeps: 1.0,
        }
    } else if has(&["gasp", "pant", "wheeze"]) {
        Kind {
            reason: "fast breathing",
            keeps: 0.7,
        }
    } else if has(&["slap", "smack", "thump", "thud", "whack", "thwack", "slam"]) {
        Kind {
            reason: "hitting sounds",
            keeps: 0.6,
        }
    } else if has(&["grunt", "groan"]) {
        Kind {
            reason: "groaning",
            keeps: 0.5,
        }
    } else {
        Kind {
            reason: "sound of distress",
            keeps: 0.7,
        }
    }
}

/// Flags each found sound whose label holds one of the sound names, ignoring case.
///
/// The strength is the confidence of the finder (0.5 when it gave none) times what its kind keeps.
/// Items without `label` are skipped. A label matches the first name in the list that it holds.
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
                .map(|(name, _)| {
                    let kind = kind_of(name);
                    let heard = item.confidence.unwrap_or(DEFAULT_WEIGHT).clamp(0.0, 1.0);
                    Signal {
                        start_ms: item.start_ms,
                        end_ms: item.end_ms,
                        source: Source::Labels,
                        reason: kind.reason.to_owned(),
                        weight: heard * kind.keeps,
                        detail: format!("heard {label}"),
                    }
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

    fn reason_and_weight(label: &str, name: &str) -> (String, f64) {
        let signals = label_signals(&[found(label, Some(0.8))], &names(&[name]));
        (signals[0].reason.clone(), signals[0].weight)
    }

    #[test]
    fn a_matching_label_keeps_its_confidence_and_gets_a_short_reason() {
        let signals = label_signals(&[found("Screaming", Some(0.8))], &names(&["scream"]));
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].reason, "screaming");
        assert!((signals[0].weight - 0.8).abs() < 1e-9);
        assert_eq!(signals[0].detail, "heard Screaming");
    }

    #[test]
    fn each_kind_of_sound_has_its_reason_and_keeps_its_share_of_the_confidence() {
        for (label, name, reason, keeps) in [
            ("Shout", "shout", "shouting", 1.0),
            ("Yell", "yell", "shouting", 1.0),
            ("Crying, sobbing", "cry", "crying", 1.0),
            ("Whimper", "whimper", "crying", 1.0),
            ("Wail, moan", "wail", "crying", 1.0),
            ("Gasp", "gasp", "fast breathing", 0.7),
            ("Pant", "pant", "fast breathing", 0.7),
            ("Slap, smack", "slap", "hitting sounds", 0.6),
            ("Thump, thud", "thump", "hitting sounds", 0.6),
            ("Whack, thwack", "whack", "hitting sounds", 0.6),
            ("Grunt", "grunt", "groaning", 0.5),
            ("Groan", "groan", "groaning", 0.5),
        ] {
            let (found_reason, weight) = reason_and_weight(label, name);
            assert_eq!(found_reason, reason, "{label}");
            assert!((weight - 0.8 * keeps).abs() < 1e-9, "{label}: {weight}");
        }
    }

    #[test]
    fn the_match_ignores_case_and_works_inside_a_longer_name() {
        let signals = label_signals(&[found("CRYING, SOBBING", None)], &names(&["sob"]));
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].reason, "crying");
        assert!((signals[0].weight - 0.5).abs() < 1e-9);
    }

    #[test]
    fn a_label_not_in_the_list_is_ignored() {
        assert!(label_signals(&[found("Cat", Some(0.9))], &names(&["scream"])).is_empty());
        assert!(label_signals(&[found("Siren", Some(0.9))], &names(&["scream"])).is_empty());
    }

    #[test]
    fn a_name_the_rules_do_not_know_gets_the_general_reason() {
        let (reason, weight) = reason_and_weight("Sniff", "sniff");
        assert_eq!(reason, "sound of distress");
        assert!((weight - 0.56).abs() < 1e-9);
    }
}
