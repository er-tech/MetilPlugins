use crate::people::{joint, Person};
use crate::tracking::Track;

/// The length of the torso of a person, shoulders to hips, in pixels: the unit speeds are counted
/// in, so that they do not depend on how big the person is in the picture. The middle value over
/// the track; where the torso is never seen, a third of the height of the box.
pub fn torso_length(track: &Track) -> f32 {
    let mut lengths: Vec<f32> = track
        .samples
        .iter()
        .filter_map(|sample| torso_vector(&sample.person))
        .map(|(dx, dy)| (dx * dx + dy * dy).sqrt())
        .filter(|length| *length > 1.0)
        .collect();
    if lengths.is_empty() {
        let mut heights: Vec<f32> = track
            .samples
            .iter()
            .map(|s| s.person.height() / 3.0)
            .collect();
        heights.sort_by(f32::total_cmp);
        return heights
            .get(heights.len() / 2)
            .copied()
            .unwrap_or(1.0)
            .max(1.0);
    }
    lengths.sort_by(f32::total_cmp);
    lengths[lengths.len() / 2]
}

/// The vector from the middle of the shoulders to the middle of the hips, when both are seen.
pub fn torso_vector(person: &Person) -> Option<(f32, f32)> {
    let shoulders = person.middle(joint::LEFT_SHOULDER, joint::RIGHT_SHOULDER)?;
    let hips = person.middle(joint::LEFT_HIP, joint::RIGHT_HIP)?;
    Some((hips.0 - shoulders.0, hips.1 - shoulders.1))
}

/// The middle of the body: the middle of the shoulders and the hips, as many of them as are seen.
/// Hands and feet are followed relative to it, so that a person who walks across the picture is
/// not taken for one who swings arms and legs.
pub fn body_center(person: &Person) -> Option<(f32, f32)> {
    let points: Vec<(f32, f32)> = [
        person.middle(joint::LEFT_SHOULDER, joint::RIGHT_SHOULDER),
        person.middle(joint::LEFT_HIP, joint::RIGHT_HIP),
    ]
    .into_iter()
    .flatten()
    .collect();
    if points.is_empty() {
        return None;
    }
    let count = points.len() as f32;
    Some((
        points.iter().map(|p| p.0).sum::<f32>() / count,
        points.iter().map(|p| p.1).sum::<f32>() / count,
    ))
}

/// How far the torso leans from upright, in degrees: 0 is upright, 90 is lying, more is upside
/// down. `None` when the shoulders or the hips are not seen.
pub fn tilt_degrees(person: &Person) -> Option<f32> {
    let (dx, dy) = torso_vector(person)?;
    Some(dx.abs().atan2(dy).to_degrees())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::people::{Keypoint, KEYPOINTS};

    fn standing(torso: f32, lean: f32) -> Person {
        let mut keypoints = [Keypoint::default(); KEYPOINTS];
        let seen = |x: f32, y: f32| Keypoint { x, y, score: 0.9 };
        keypoints[joint::LEFT_SHOULDER] = seen(90.0, 100.0);
        keypoints[joint::RIGHT_SHOULDER] = seen(110.0, 100.0);
        keypoints[joint::LEFT_HIP] = seen(90.0 + lean, 100.0 + torso);
        keypoints[joint::RIGHT_HIP] = seen(110.0 + lean, 100.0 + torso);
        Person {
            keypoints,
            bbox: [60.0, 80.0, 140.0, 300.0],
            score: 0.9,
        }
    }

    #[test]
    fn the_tilt_is_zero_upright_ninety_lying_and_over_ninety_upside_down() {
        assert!(tilt_degrees(&standing(80.0, 0.0)).unwrap() < 1.0);
        assert!((tilt_degrees(&standing(0.001, 80.0)).unwrap() - 90.0).abs() < 1.0);
        assert!(tilt_degrees(&standing(-80.0, 10.0)).unwrap() > 90.0);
        assert!(tilt_degrees(&Person {
            keypoints: [Keypoint::default(); KEYPOINTS],
            bbox: [0.0; 4],
            score: 1.0
        })
        .is_none());
    }

    #[test]
    fn the_torso_length_is_the_middle_value_of_the_track_and_falls_back_to_the_box() {
        let sample = |person: Person, frame| crate::tracking::Sample {
            frame,
            time_ms: frame as u64 * 200,
            person,
        };
        let track = Track {
            id: 1,
            samples: vec![
                sample(standing(80.0, 0.0), 0),
                sample(standing(60.0, 0.0), 1),
                sample(standing(200.0, 0.0), 2),
            ],
        };
        assert!((torso_length(&track) - 80.0).abs() < 0.01);
        let unseen = Person {
            keypoints: [Keypoint::default(); KEYPOINTS],
            bbox: [0.0, 0.0, 50.0, 300.0],
            score: 1.0,
        };
        let blind = Track {
            id: 2,
            samples: vec![sample(unseen, 0)],
        };
        assert!((torso_length(&blind) - 100.0).abs() < 0.01);
    }
}
