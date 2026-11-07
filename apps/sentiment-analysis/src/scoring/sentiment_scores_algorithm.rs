use serde_json::{json, Value};

/// How sure the model is of each of the three, and what that comes to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scored {
    pub positive: f64,
    pub neutral: f64,
    pub negative: f64,
}

/// The labels, in the order the model answers them.
const LABELS: [&str; 3] = ["positive", "neutral", "negative"];
/// A mean score above this is positive and below its opposite is negative (see `overall`).
const OVERALL_THRESHOLD: f64 = 0.25;

impl Scored {
    /// The label with the most probability.
    pub fn label(&self) -> &'static str {
        let chances = [self.positive, self.neutral, self.negative];
        let best = (0..3).fold(0, |best, at| {
            if chances[at] > chances[best] {
                at
            } else {
                best
            }
        });
        LABELS[best]
    }

    /// The probability of that label.
    pub fn confidence(&self) -> f64 {
        self.positive.max(self.neutral).max(self.negative)
    }

    /// From -1 (certainly negative) to 1 (certainly positive): positive minus negative.
    pub fn score(&self) -> f64 {
        self.positive - self.negative
    }

    /// The part of a result for one segment: label, confidence, score and the three chances,
    /// rounded to three decimals.
    pub fn to_value(self) -> Value {
        json!({
            "label": self.label(),
            "confidence": round(self.confidence()),
            "score": round(self.score()),
            "probabilities": {
                "positive": round(self.positive),
                "neutral": round(self.neutral),
                "negative": round(self.negative),
            },
        })
    }
}

fn round(number: f64) -> f64 {
    (number * 1000.0).round() / 1000.0
}

/// The chances of the three labels for each row of `logits` (three numbers each), by softmax.
pub fn score_logits(logits: &[f32]) -> Vec<Scored> {
    let (rows, _) = logits.as_chunks::<3>();
    rows.iter()
        .map(|row| {
            let top = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let exponents: Vec<f64> = row.iter().map(|l| f64::from((l - top).exp())).collect();
            let sum: f64 = exponents.iter().sum();
            Scored {
                positive: exponents[0] / sum,
                neutral: exponents[1] / sum,
                negative: exponents[2] / sum,
            }
        })
        .collect()
}

/// The overall value of many segments, each given as `(milliseconds it lasts, scored)`: the mean
/// score weighted by how long each lasts (a segment of no length counts as one millisecond), the
/// label that mean comes to (above 0.25 positive, below -0.25 negative, else neutral), and how
/// many segments there were of each label.
pub fn overall(segments: &[(u64, Scored)]) -> Value {
    let weight = |ms: u64| ms.max(1) as f64;
    let total: f64 = segments.iter().map(|(ms, _)| weight(*ms)).sum();
    let mean = if total > 0.0 {
        segments
            .iter()
            .map(|(ms, scored)| weight(*ms) * scored.score())
            .sum::<f64>()
            / total
    } else {
        0.0
    };
    let label = if mean > OVERALL_THRESHOLD {
        "positive"
    } else if mean < -OVERALL_THRESHOLD {
        "negative"
    } else {
        "neutral"
    };
    let count = |wanted: &str| {
        segments
            .iter()
            .filter(|(_, scored)| scored.label() == wanted)
            .count()
    };
    json!({
        "label": label,
        "score": if segments.is_empty() { Value::Null } else { json!(round(mean)) },
        "segments": segments.len(),
        "positive": count("positive"),
        "neutral": count("neutral"),
        "negative": count("negative"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logits_become_chances_that_add_to_one_and_the_biggest_names_the_label() {
        let scored = score_logits(&[3.0, 0.5, -2.0, -1.0, 0.0, 4.0]);
        assert_eq!(scored.len(), 2);
        for each in &scored {
            assert!((each.positive + each.neutral + each.negative - 1.0).abs() < 1e-9);
        }
        assert_eq!(
            (scored[0].label(), scored[1].label()),
            ("positive", "negative")
        );
        assert!(scored[0].score() > 0.8 && scored[1].score() < -0.9);
        assert_eq!(scored[0].to_value()["label"], "positive");
    }

    #[test]
    fn the_overall_weighs_each_segment_by_its_length_and_counts_the_labels() {
        let good = score_logits(&[4.0, 0.0, -4.0])[0];
        let bad = score_logits(&[-4.0, 0.0, 4.0])[0];
        let value = overall(&[(1000, good), (9000, bad)]);
        assert_eq!(value["label"], "negative");
        assert_eq!(
            (value["positive"].as_u64(), value["negative"].as_u64()),
            (Some(1), Some(1))
        );
        let balanced = overall(&[(5000, good), (5000, bad)]);
        assert_eq!(balanced["label"], "neutral");
        assert_eq!(overall(&[])["score"], Value::Null);
    }
}
