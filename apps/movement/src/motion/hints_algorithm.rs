use std::collections::HashMap;

use crate::people::{joint, Person};
use crate::tracking::Track;

use super::torso_algorithm::{body_center, tilt_degrees, torso_length};
use super::vocabulary_config::{
    CLOSE_AND_FAST, FALLING, FAST_ARM, FAST_LEG, ON_THE_GROUND, STRIKING,
};

/// A pause longer than this between two pictures of a person is not a movement.
const MAX_INTERVAL_MS: u64 = 1500;
/// A fall is looked for over this many milliseconds.
const FALL_WINDOW_MS: u64 = 1000;
/// A fall is a drop of the hips of this many torso lengths, ending with the torso leaning at least
/// `FALL_TILT` degrees.
const FALL_DROP: f32 = 0.8;
const FALL_TILT: f32 = 45.0;
/// A person lying has the torso leaning at least this much and the box wider than it is high.
const GROUND_TILT: f32 = 60.0;
/// A hand moves towards another person when its direction is within about 45 degrees of the way to
/// them, and it ends no further from them than this many torso lengths.
const STRIKE_COSINE: f32 = 0.7;
const STRIKE_REACH: f32 = 1.5;
/// Two people are close when their boxes' middles are no further apart than this share of the
/// height of the taller.
const CLOSE_SHARE: f32 = 0.9;

/// What the movements are told apart by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionSettings {
    /// A wrist counts as fast above this many torso lengths a second.
    pub arm_speed: f32,
    /// An ankle counts as fast above this many torso lengths a second.
    pub leg_speed: f32,
}

/// One reason, in one moment, to say a movement: what was seen, by whom and how far past its limit.
#[derive(Debug, Clone, PartialEq)]
pub struct Hint {
    pub time_ms: u64,
    /// Where the interval it was seen over ends.
    pub end_ms: u64,
    pub label: &'static str,
    pub person: usize,
    /// How many times the limit it reached (1 is just at the limit).
    pub strength: f32,
    /// How sure the model was of the keypoints it rests on, 0 to 1.
    pub score: f32,
    pub detail: String,
}

/// Every hint of every track. Speeds are in torso lengths a second, so that a person far away and a
/// person close by are judged alike.
pub fn find_hints(tracks: &[Track], settings: &MotionSettings) -> Vec<Hint> {
    let units: Vec<f32> = tracks.iter().map(torso_length).collect();
    let by_frame = people_by_frame(tracks);
    let mut hints = Vec::new();
    for (track, unit) in tracks.iter().zip(&units) {
        for pair in track.samples.windows(2) {
            let (before, now) = (&pair[0], &pair[1]);
            let step = now.time_ms.saturating_sub(before.time_ms);
            if step == 0 || step > MAX_INTERVAL_MS {
                continue;
            }
            let seconds = step as f32 / 1000.0;
            let at = |hint: Hint| Hint {
                time_ms: before.time_ms,
                end_ms: now.time_ms,
                ..hint
            };
            let arm = fastest(
                &before.person,
                &now.person,
                &[joint::LEFT_WRIST, joint::RIGHT_WRIST],
                *unit,
                seconds,
            );
            if let Some(Fastest {
                speed,
                name,
                score,
                from,
                to,
            }) = arm
            {
                if speed >= settings.arm_speed {
                    hints.push(at(hint(
                        FAST_ARM,
                        track.id,
                        speed / settings.arm_speed,
                        score,
                        format!("{name} {speed:.1} torso lengths a second"),
                    )));
                    let others = by_frame.get(&now.frame).map(Vec::as_slice).unwrap_or(&[]);
                    if let Some(target) = struck_at(from, to, *unit, others, track.id) {
                        hints.push(at(hint(
                            STRIKING,
                            track.id,
                            speed / settings.arm_speed,
                            score,
                            format!(
                                "{name} {speed:.1} torso lengths a second, towards person {target}"
                            ),
                        )));
                    }
                }
            }
            let leg = fastest(
                &before.person,
                &now.person,
                &[joint::LEFT_ANKLE, joint::RIGHT_ANKLE],
                *unit,
                seconds,
            );
            if let Some(Fastest {
                speed, name, score, ..
            }) = leg
            {
                if speed >= settings.leg_speed {
                    hints.push(at(hint(
                        FAST_LEG,
                        track.id,
                        speed / settings.leg_speed,
                        score,
                        format!("{name} {speed:.1} torso lengths a second"),
                    )));
                }
            }
        }
        for (place, now) in track.samples.iter().enumerate() {
            let Some(tilt) = tilt_degrees(&now.person) else {
                continue;
            };
            let end = track
                .samples
                .get(place + 1)
                .map_or(now.time_ms + 200, |next| next.time_ms);
            let at = |hint: Hint| Hint {
                time_ms: now.time_ms,
                end_ms: end,
                ..hint
            };
            if tilt >= GROUND_TILT && now.person.width() > now.person.height() * 0.8 {
                hints.push(at(hint(
                    ON_THE_GROUND,
                    track.id,
                    tilt / GROUND_TILT,
                    0.8,
                    format!("the torso leans {tilt:.0} degrees from upright"),
                )));
            }
            if tilt >= FALL_TILT {
                let hips_now = now.person.middle(joint::LEFT_HIP, joint::RIGHT_HIP);
                let earlier = track.samples[..place]
                    .iter()
                    .rev()
                    .take_while(|s| now.time_ms - s.time_ms <= FALL_WINDOW_MS)
                    .filter_map(|s| s.person.middle(joint::LEFT_HIP, joint::RIGHT_HIP));
                let highest = earlier.map(|hips| hips.1).fold(f32::INFINITY, f32::min);
                if let (Some(hips), true) = (hips_now, highest.is_finite()) {
                    let drop = (hips.1 - highest) / *unit;
                    if drop >= FALL_DROP {
                        hints.push(at(hint(FALLING, track.id, drop / FALL_DROP, 0.8, format!("the hips dropped {drop:.1} torso lengths and the torso leans {tilt:.0} degrees"))));
                    }
                }
            }
        }
    }
    hints.extend(close_and_fast(tracks, &hints));
    hints.sort_by_key(|hint| (hint.time_ms, hint.person));
    hints
}

fn hint(label: &'static str, person: usize, strength: f32, score: f32, detail: String) -> Hint {
    Hint {
        time_ms: 0,
        end_ms: 0,
        label,
        person,
        strength,
        score,
        detail,
    }
}

/// The people of every picture, with their track numbers.
fn people_by_frame(tracks: &[Track]) -> HashMap<usize, Vec<(usize, &Person)>> {
    let mut by_frame: HashMap<usize, Vec<(usize, &Person)>> = HashMap::new();
    for track in tracks {
        for sample in &track.samples {
            by_frame
                .entry(sample.frame)
                .or_default()
                .push((track.id, &sample.person));
        }
    }
    by_frame
}

/// The fastest joint of one person between two moments.
struct Fastest {
    /// In torso lengths a second.
    speed: f32,
    name: &'static str,
    /// The mean score of the keypoints it rests on.
    score: f32,
    from: (f32, f32),
    to: (f32, f32),
}

/// The fastest of the `joints` of one person between two moments, in torso lengths a second and
/// relative to the middle of the body, with its name, the mean score of the keypoints it rests on
/// and where it was and where it went in the picture. A person with no torso seen has none.
fn fastest(
    before: &Person,
    now: &Person,
    joints: &[usize],
    unit: f32,
    seconds: f32,
) -> Option<Fastest> {
    let (centre_before, centre_now) = (body_center(before)?, body_center(now)?);
    joints
        .iter()
        .filter_map(|&at| {
            let (a, b) = (before.joint(at)?, now.joint(at)?);
            let (from, to) = (
                (a.x - centre_before.0, a.y - centre_before.1),
                (b.x - centre_now.0, b.y - centre_now.1),
            );
            let distance = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
            Some(Fastest {
                speed: distance / unit / seconds,
                name: joint_name(at),
                score: (a.score + b.score) / 2.0,
                from: (a.x, a.y),
                to: (b.x, b.y),
            })
        })
        .max_by(|a, b| a.speed.total_cmp(&b.speed))
}

fn joint_name(at: usize) -> &'static str {
    match at {
        joint::LEFT_WRIST => "left wrist",
        joint::RIGHT_WRIST => "right wrist",
        joint::LEFT_ANKLE => "left ankle",
        _ => "right ankle",
    }
}

/// The track of the person a hand that went from `from` to `to` is heading to, when it is going
/// towards someone (and ends close to them).
fn struck_at(
    from: (f32, f32),
    to: (f32, f32),
    unit: f32,
    others: &[(usize, &Person)],
    own: usize,
) -> Option<usize> {
    let way = (to.0 - from.0, to.1 - from.1);
    let length = (way.0 * way.0 + way.1 * way.1).sqrt();
    if length < 1.0 {
        return None;
    }
    others
        .iter()
        .filter(|(id, _)| *id != own)
        .filter_map(|(id, other)| {
            let target = other
                .middle(joint::LEFT_SHOULDER, joint::RIGHT_SHOULDER)
                .into_iter()
                .chain(other.middle(joint::LEFT_HIP, joint::RIGHT_HIP))
                .fold(None, |sum: Option<((f32, f32), f32)>, point| match sum {
                    None => Some((point, 1.0)),
                    Some((total, count)) => {
                        Some(((total.0 + point.0, total.1 + point.1), count + 1.0))
                    }
                })
                .map(|(total, count)| (total.0 / count, total.1 / count))
                .unwrap_or_else(|| other.center());
            let to_target = (target.0 - from.0, target.1 - from.1);
            let reach = ((target.0 - to.0).powi(2) + (target.1 - to.1).powi(2)).sqrt();
            let aim = (way.0 * to_target.0 + way.1 * to_target.1)
                / (length * (to_target.0.powi(2) + to_target.1.powi(2)).sqrt().max(1.0));
            (aim >= STRIKE_COSINE && reach <= STRIKE_REACH * unit).then_some((reach, *id))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id)
}

/// A hint for each picture where two people are close and one of them moves an arm or a leg fast.
fn close_and_fast(tracks: &[Track], hints: &[Hint]) -> Vec<Hint> {
    let by_frame = people_by_frame(tracks);
    let mut found = Vec::new();
    for fast in hints
        .iter()
        .filter(|h| h.label == FAST_ARM || h.label == FAST_LEG)
    {
        let Some((frame, person)) = tracks.iter().find(|t| t.id == fast.person).and_then(|t| {
            t.samples
                .iter()
                .find(|s| s.time_ms == fast.end_ms)
                .map(|s| (s.frame, &s.person))
        }) else {
            continue;
        };
        let near = by_frame
            .get(&frame)
            .into_iter()
            .flatten()
            .filter(|(id, _)| *id != fast.person)
            .find(|(_, other)| {
                let (a, b) = (person.center(), other.center());
                ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
                    <= CLOSE_SHARE * person.height().max(other.height())
            });
        if let Some((other, _)) = near {
            found.push(Hint {
                label: CLOSE_AND_FAST,
                detail: format!(
                    "person {} and person {other} are close, and {}",
                    fast.person, fast.detail
                ),
                ..fast.clone()
            });
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::people::{Keypoint, KEYPOINTS};
    use crate::tracking::Sample;

    /// A person with a torso of 100 pixels, the left wrist at `wrist`, and the hips `drop` lower.
    fn person(x: f32, wrist: (f32, f32), drop: f32, lean: f32) -> Person {
        let mut keypoints = [Keypoint::default(); KEYPOINTS];
        let seen = |x: f32, y: f32| Keypoint { x, y, score: 0.9 };
        keypoints[joint::LEFT_SHOULDER] = seen(x - 20.0, 100.0);
        keypoints[joint::RIGHT_SHOULDER] = seen(x + 20.0, 100.0);
        keypoints[joint::LEFT_HIP] = seen(x - 20.0 + lean, 200.0 + drop);
        keypoints[joint::RIGHT_HIP] = seen(x + 20.0 + lean, 200.0 + drop);
        keypoints[joint::LEFT_WRIST] = seen(wrist.0, wrist.1);
        Person {
            keypoints,
            bbox: [x - 40.0, 60.0, x + 40.0, 300.0 + drop],
            score: 0.9,
        }
    }

    fn track(id: usize, people: Vec<Person>) -> Track {
        Track {
            id,
            samples: people
                .into_iter()
                .enumerate()
                .map(|(frame, person)| Sample {
                    frame,
                    time_ms: frame as u64 * 200,
                    person,
                })
                .collect(),
        }
    }

    const SETTINGS: MotionSettings = MotionSettings {
        arm_speed: 4.0,
        leg_speed: 5.0,
    };

    #[test]
    fn a_wrist_that_moves_a_torso_length_in_a_fifth_of_a_second_is_a_fast_arm_and_a_slow_one_is_not(
    ) {
        let fast = track(
            1,
            vec![
                person(200.0, (100.0, 150.0), 0.0, 0.0),
                person(200.0, (200.0, 150.0), 0.0, 0.0),
            ],
        );
        let hints = find_hints(&[fast], &SETTINGS);
        let arm: Vec<_> = hints.iter().filter(|h| h.label == FAST_ARM).collect();
        assert_eq!(arm.len(), 1);
        assert!((arm[0].strength - 5.0 / 4.0).abs() < 0.01, "{:?}", arm[0]);
        assert!(arm[0]
            .detail
            .contains("left wrist 5.0 torso lengths a second"));
        let slow = track(
            1,
            vec![
                person(200.0, (100.0, 150.0), 0.0, 0.0),
                person(200.0, (130.0, 150.0), 0.0, 0.0),
            ],
        );
        assert!(find_hints(&[slow], &SETTINGS).is_empty());
    }

    #[test]
    fn a_person_who_crosses_the_picture_with_still_arms_and_legs_has_no_fast_arm_or_leg() {
        let shifted = |dx: f32| {
            let mut walker = person(200.0 + dx, (260.0 + dx, 150.0), 0.0, 0.0);
            walker.keypoints[joint::LEFT_ANKLE] = Keypoint {
                x: 200.0 + dx,
                y: 290.0,
                score: 0.9,
            };
            walker
        };
        let walker = track(1, vec![shifted(0.0), shifted(150.0), shifted(300.0)]);
        assert!(find_hints(&[walker], &SETTINGS).is_empty());
    }

    #[test]
    fn a_fast_hand_heading_for_another_person_is_a_striking_motion_towards_them() {
        let hitter = track(
            1,
            vec![
                person(200.0, (260.0, 150.0), 0.0, 0.0),
                person(200.0, (380.0, 150.0), 0.0, 0.0),
            ],
        );
        let other = track(
            2,
            vec![
                person(450.0, (500.0, 150.0), 0.0, 0.0),
                person(450.0, (500.0, 150.0), 0.0, 0.0),
            ],
        );
        let hints = find_hints(&[hitter, other], &SETTINGS);
        let strike = hints
            .iter()
            .find(|h| h.label == STRIKING)
            .expect("a strike");
        assert_eq!(strike.person, 1);
        assert!(
            strike.detail.contains("towards person 2"),
            "{}",
            strike.detail
        );
        let away = track(
            1,
            vec![
                person(200.0, (260.0, 150.0), 0.0, 0.0),
                person(200.0, (120.0, 150.0), 0.0, 0.0),
            ],
        );
        let other = track(
            2,
            vec![
                person(450.0, (500.0, 150.0), 0.0, 0.0),
                person(450.0, (500.0, 150.0), 0.0, 0.0),
            ],
        );
        assert!(find_hints(&[away, other], &SETTINGS)
            .iter()
            .all(|h| h.label != STRIKING));
    }

    /// A person with a torso of 100 pixels whose hips are at (`x`, `hip_y`) and whose torso leans
    /// `angle` degrees from upright, the shoulders above and to the side of the hips.
    fn posed(x: f32, hip_y: f32, angle: f32) -> Person {
        let (sin, cos) = (angle.to_radians().sin(), angle.to_radians().cos());
        let mut keypoints = [Keypoint::default(); KEYPOINTS];
        let seen = |x: f32, y: f32| Keypoint { x, y, score: 0.9 };
        let (shoulder_x, shoulder_y) = (x - 100.0 * sin, hip_y - 100.0 * cos);
        keypoints[joint::LEFT_SHOULDER] = seen(shoulder_x - 20.0 * cos, shoulder_y + 20.0 * sin);
        keypoints[joint::RIGHT_SHOULDER] = seen(shoulder_x + 20.0 * cos, shoulder_y - 20.0 * sin);
        keypoints[joint::LEFT_HIP] = seen(x - 20.0 * cos, hip_y + 20.0 * sin);
        keypoints[joint::RIGHT_HIP] = seen(x + 20.0 * cos, hip_y - 20.0 * sin);
        Person {
            keypoints,
            bbox: [x - 100.0, hip_y - 100.0, x + 100.0, hip_y + 100.0],
            score: 0.9,
        }
    }

    #[test]
    fn hips_that_drop_while_the_torso_leans_over_are_a_fall_and_a_crouch_is_not() {
        // Hips from 200 down to 330 (1.3 torso lengths) while the torso goes from upright to lying.
        let fall = track(
            1,
            vec![
                posed(200.0, 200.0, 0.0),
                posed(200.0, 240.0, 40.0),
                posed(200.0, 330.0, 80.0),
            ],
        );
        let hints = find_hints(&[fall], &SETTINGS);
        let falling = hints.iter().find(|h| h.label == FALLING).expect("a fall");
        assert!(
            falling.strength > 1.0 && falling.detail.contains("torso lengths"),
            "{falling:?}"
        );
        // A crouch drops the hips as far and stays upright.
        let crouch = track(
            1,
            vec![
                posed(200.0, 200.0, 0.0),
                posed(200.0, 260.0, 5.0),
                posed(200.0, 330.0, 10.0),
            ],
        );
        assert!(find_hints(&[crouch], &SETTINGS)
            .iter()
            .all(|h| h.label != FALLING));
        // Lying down slowly, over longer than the window of a fall, is not a fall.
        let slow = Track {
            id: 1,
            samples: [(0u64, 200.0, 0.0), (2000, 240.0, 40.0), (4000, 330.0, 80.0)]
                .into_iter()
                .enumerate()
                .map(|(frame, (time_ms, hip_y, angle))| Sample {
                    frame,
                    time_ms,
                    person: posed(200.0, hip_y, angle),
                })
                .collect(),
        };
        assert!(find_hints(&[slow], &SETTINGS)
            .iter()
            .all(|h| h.label != FALLING));
    }

    #[test]
    fn a_wide_box_with_a_torso_that_lies_is_a_person_on_the_ground() {
        let mut lying = person(200.0, (0.0, 0.0), 0.0, 100.0);
        lying.bbox = [100.0, 150.0, 320.0, 230.0];
        for keypoint in [joint::LEFT_HIP, joint::RIGHT_HIP] {
            lying.keypoints[keypoint].y = 105.0;
        }
        let hints = find_hints(&[track(1, vec![lying.clone(), lying])], &SETTINGS);
        assert!(hints.iter().filter(|h| h.label == ON_THE_GROUND).count() >= 2);
    }

    #[test]
    fn a_fast_arm_next_to_another_person_is_also_two_people_close_with_fast_movement() {
        let mover = track(
            1,
            vec![
                person(200.0, (100.0, 150.0), 0.0, 0.0),
                person(200.0, (200.0, 150.0), 0.0, 0.0),
            ],
        );
        let near = track(
            2,
            vec![
                person(260.0, (0.0, 0.0), 0.0, 0.0),
                person(260.0, (0.0, 0.0), 0.0, 0.0),
            ],
        );
        let far = track(
            2,
            vec![
                person(900.0, (0.0, 0.0), 0.0, 0.0),
                person(900.0, (0.0, 0.0), 0.0, 0.0),
            ],
        );
        assert!(find_hints(&[mover.clone(), near], &SETTINGS)
            .iter()
            .any(|h| h.label == CLOSE_AND_FAST));
        assert!(find_hints(&[mover, far], &SETTINGS)
            .iter()
            .all(|h| h.label != CLOSE_AND_FAST));
    }
}
