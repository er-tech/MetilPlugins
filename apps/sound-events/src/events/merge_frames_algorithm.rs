use super::event_contract::Event;

/// Joins what was scored frame by frame into events, for each class. `scores` holds the frames one
/// after another, `classes` scores each. A class is heard in a frame when its score reaches
/// `min_confidence`. Heard in frames at most `gap_frames` + 1 apart, it is one event; farther apart,
/// the sound stopped and came back and that is another. A stretch heard in fewer than `min_frames`
/// frames is dropped.
///
/// The events come back in the order they start.
pub fn merge_frames(
    scores: &[f32],
    classes: usize,
    wanted: &[bool],
    min_confidence: f32,
    gap_frames: usize,
    min_frames: usize,
) -> Vec<Event> {
    let frames = scores.len() / classes;
    let mut events: Vec<Event> = Vec::new();
    for label in (0..classes).filter(|label| wanted[*label]) {
        let mut open: Option<Event> = None;
        for frame in 0..frames {
            let score = scores[frame * classes + label];
            if score < min_confidence {
                continue;
            }
            open = match open {
                Some(mut event) if frame - event.last_frame <= gap_frames + 1 => {
                    event.last_frame = frame;
                    event.confidence = event.confidence.max(score);
                    event.frames += 1;
                    Some(event)
                }
                other => {
                    events.extend(other);
                    Some(Event {
                        label,
                        first_frame: frame,
                        last_frame: frame,
                        confidence: score,
                        frames: 1,
                    })
                }
            };
        }
        events.extend(open);
    }
    events.retain(|event| event.frames >= min_frames);
    events.sort_by_key(|event| (event.first_frame, event.label));
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scores of 3 classes for the frames given as rows.
    fn scores(rows: &[[f32; 3]]) -> Vec<f32> {
        rows.iter().flatten().copied().collect()
    }

    #[test]
    fn a_sound_that_goes_on_is_one_event_with_its_surest_score() {
        let found = merge_frames(
            &scores(&[
                [0.0, 0.6, 0.0],
                [0.0, 0.9, 0.0],
                [0.0, 0.7, 0.0],
                [0.0, 0.0, 0.0],
            ]),
            3,
            &[true; 3],
            0.5,
            0,
            1,
        );
        assert_eq!(found.len(), 1);
        let event = found[0];
        assert_eq!(
            (event.label, event.first_frame, event.last_frame),
            (1, 0, 2)
        );
        assert_eq!((event.confidence, event.frames), (0.9, 3));
    }

    #[test]
    fn a_gap_longer_than_allowed_splits_it_and_a_shorter_one_does_not() {
        // Heard in frames 0, 2 and 5.
        let rows = [
            [0.8, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.8, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.8, 0.0, 0.0],
        ];
        let starts = |gap| -> Vec<usize> {
            merge_frames(&scores(&rows), 3, &[true; 3], 0.5, gap, 1)
                .iter()
                .map(|event| event.first_frame)
                .collect()
        };
        assert_eq!(starts(0), [0, 2, 5], "one frame of silence splits it");
        assert_eq!(
            starts(1),
            [0, 5],
            "frames 0 and 2 are one, 5 is two frames after"
        );
        assert_eq!(starts(2), [0]);
    }

    #[test]
    fn a_blip_is_dropped_and_unwanted_classes_are_ignored() {
        let rows = [[0.9, 0.9, 0.0], [0.0, 0.9, 0.0]];
        let found = merge_frames(&scores(&rows), 3, &[true, false, true], 0.5, 0, 2);
        assert!(
            found.is_empty(),
            "class 0 was one frame, class 1 was not wanted"
        );
        let found = merge_frames(&scores(&rows), 3, &[true; 3], 0.5, 0, 2);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].label, 1);
    }

    #[test]
    fn no_frames_give_no_events() {
        assert!(merge_frames(&[], 3, &[true; 3], 0.5, 0, 1).is_empty());
    }
}
