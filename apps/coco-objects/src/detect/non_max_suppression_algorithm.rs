use super::detection_contract::Detection;

/// Keeps the best box of each pile of boxes around the same thing: going from the most sure to the
/// least, a box is dropped when one already kept, of the same class, covers more than
/// `max_overlap` of it (intersection over union).
pub fn non_max_suppression(mut found: Vec<Detection>, max_overlap: f32) -> Vec<Detection> {
    found.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<Detection> = Vec::new();
    for candidate in found {
        let covered = kept
            .iter()
            .any(|best| best.class == candidate.class && overlap(best, &candidate) > max_overlap);
        if !covered {
            kept.push(candidate);
        }
    }
    kept
}

fn overlap(a: &Detection, b: &Detection) -> f32 {
    let width = (a.x2.min(b.x2) - a.x1.max(b.x1)).max(0.0);
    let height = (a.y2.min(b.y2) - a.y1.max(b.y1)).max(0.0);
    let both = width * height;
    let union = a.area() + b.area() - both;
    if union <= 0.0 {
        0.0
    } else {
        both / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(class: usize, score: f32, x1: f32) -> Detection {
        Detection {
            class,
            score,
            x1,
            y1: 0.0,
            x2: x1 + 10.0,
            y2: 10.0,
        }
    }

    #[test]
    fn the_surest_of_two_boxes_on_the_same_thing_is_kept() {
        let kept = non_max_suppression(vec![at(15, 0.6, 1.0), at(15, 0.9, 0.0)], 0.45);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].score, 0.9);
    }

    #[test]
    fn boxes_far_apart_or_of_other_classes_are_both_kept() {
        let kept = non_max_suppression(
            vec![at(15, 0.9, 0.0), at(15, 0.8, 50.0), at(16, 0.7, 0.0)],
            0.45,
        );
        assert_eq!(kept.len(), 3);
        assert_eq!(kept[0].score, 0.9, "the surest first");
    }
}
