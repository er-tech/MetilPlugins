use super::hints_algorithm::Hint;

/// A movement: hints of one kind about one person, close together in time, as one.
#[derive(Debug, Clone, PartialEq)]
pub struct Movement {
    pub start_ms: u64,
    pub end_ms: u64,
    pub label: &'static str,
    pub confidence: f32,
    pub person: usize,
    pub detail: String,
    /// In how many pictures it was seen.
    pub samples: usize,
}

/// Joins the hints that have the same label and person and are no more than `gap_ms` apart, in time
/// order. A movement is kept when it was seen in at least `min_samples` pictures and its confidence
/// reaches `min_confidence`. The confidence is half the score of the keypoints plus another half for
/// how far past its limit the strongest hint went (up to twice the limit), all times the score:
/// just at the limit is half of the score, twice the limit or more is the whole of it.
pub fn join_hints(
    hints: &[Hint],
    gap_ms: u64,
    min_samples: usize,
    min_confidence: f32,
) -> Vec<Movement> {
    let mut sorted: Vec<&Hint> = hints.iter().collect();
    sorted.sort_by_key(|hint| hint.time_ms);
    let mut open: Vec<(Movement, f32, f32)> = Vec::new();
    let mut done: Vec<Movement> = Vec::new();
    for hint in sorted {
        let found = open.iter_mut().find(|(movement, _, _)| {
            movement.label == hint.label
                && movement.person == hint.person
                && hint.time_ms <= movement.end_ms + gap_ms
        });
        match found {
            Some((movement, strongest, score_sum)) => {
                movement.end_ms = movement.end_ms.max(hint.end_ms);
                movement.samples += 1;
                *score_sum += hint.score;
                if hint.strength > *strongest {
                    *strongest = hint.strength;
                    movement.detail = hint.detail.clone();
                }
            }
            None => {
                // A movement that nothing has joined for longer than the gap is closed.
                let mut still = Vec::new();
                for (movement, strongest, score_sum) in open.drain(..) {
                    if movement.label == hint.label && movement.person == hint.person {
                        done.push(finish(movement, strongest, score_sum));
                    } else {
                        still.push((movement, strongest, score_sum));
                    }
                }
                open = still;
                open.push((
                    Movement {
                        start_ms: hint.time_ms,
                        end_ms: hint.end_ms,
                        label: hint.label,
                        confidence: 0.0,
                        person: hint.person,
                        detail: hint.detail.clone(),
                        samples: 1,
                    },
                    hint.strength,
                    hint.score,
                ));
            }
        }
    }
    done.extend(
        open.into_iter()
            .map(|(movement, strongest, score_sum)| finish(movement, strongest, score_sum)),
    );
    done.retain(|movement| {
        movement.samples >= min_samples && movement.confidence >= min_confidence
    });
    done.sort_by_key(|movement| (movement.start_ms, movement.person));
    done
}

fn finish(mut movement: Movement, strongest: f32, score_sum: f32) -> Movement {
    let score = score_sum / movement.samples as f32;
    let past = (strongest - 1.0).clamp(0.0, 1.0);
    movement.confidence = (((0.5 + 0.5 * past) * score) * 1000.0).round() / 1000.0;
    movement
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hint(time_ms: u64, label: &'static str, person: usize, strength: f32) -> Hint {
        Hint {
            time_ms,
            end_ms: time_ms + 200,
            label,
            person,
            strength,
            score: 0.8,
            detail: format!("at {time_ms}"),
        }
    }

    #[test]
    fn hints_close_in_time_are_one_movement_and_a_long_gap_or_another_person_makes_another() {
        let hints = [
            hint(0, "fast arm movement", 1, 1.2),
            hint(200, "fast arm movement", 1, 2.0),
            hint(1000, "fast arm movement", 1, 1.1),
            hint(4000, "fast arm movement", 1, 1.1),
            hint(200, "fast arm movement", 2, 1.1),
        ];
        let movements = join_hints(&hints, 1000, 1, 0.0);
        assert_eq!(movements.len(), 3);
        let first = movements
            .iter()
            .find(|m| m.person == 1 && m.start_ms == 0)
            .unwrap();
        assert_eq!((first.end_ms, first.samples), (1200, 3));
        assert_eq!(first.detail, "at 200");
    }

    #[test]
    fn a_movement_needs_the_samples_and_the_confidence_asked_for() {
        let hints = [
            hint(0, "falling", 1, 1.0),
            hint(5000, "falling", 1, 3.0),
            hint(5200, "falling", 1, 3.0),
        ];
        assert_eq!(join_hints(&hints, 1000, 2, 0.0).len(), 1);
        let sure = join_hints(&hints, 1000, 1, 0.7);
        assert_eq!(sure.len(), 1);
        assert!(
            (sure[0].confidence - 0.8).abs() < 1e-6,
            "{}",
            sure[0].confidence
        );
        let weak = join_hints(&hints[..1], 1000, 1, 0.0);
        assert!((weak[0].confidence - 0.4).abs() < 1e-6);
    }
}
