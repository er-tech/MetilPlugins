use crate::people::Person;

/// A person is the same as in a recent picture when the middles of their boxes are no further apart
/// than this share of the height of the taller box.
const MAX_MOVE: f32 = 0.75;
/// A person who is lost is looked for again in this many pictures.
const MAX_GAP: usize = 3;

/// A person at one moment.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// The place of the picture among the pictures looked at.
    pub frame: usize,
    pub time_ms: u64,
    pub person: Person,
}

/// One person followed through the video. The number is only for this run: it is never an identity.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: usize,
    pub samples: Vec<Sample>,
}

/// Joins the people of the pictures, each given as `(time in milliseconds, the people in it)` in
/// order, into tracks. A person is matched with the nearest track by the middle of the box (no
/// face, no look, no identity), the nearest pairs first. A person nobody matches starts a track.
/// Track numbers start at 1.
pub fn track_people(frames: &[(u64, Vec<Person>)]) -> Vec<Track> {
    let mut tracks: Vec<Track> = Vec::new();
    for (frame, (time_ms, people)) in frames.iter().enumerate() {
        let mut pairs: Vec<(f32, usize, usize)> = Vec::new();
        for (track_at, track) in tracks.iter().enumerate() {
            let Some(last) = track.samples.last() else {
                continue;
            };
            if frame - last.frame > MAX_GAP {
                continue;
            }
            for (person_at, person) in people.iter().enumerate() {
                let (a, b) = (last.person.center(), person.center());
                let tall = last.person.height().max(person.height());
                let distance = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt() / tall;
                if distance <= MAX_MOVE {
                    pairs.push((distance, track_at, person_at));
                }
            }
        }
        pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
        let (mut track_taken, mut person_taken) =
            (vec![false; tracks.len()], vec![false; people.len()]);
        for (_, track_at, person_at) in pairs {
            if track_taken[track_at] || person_taken[person_at] {
                continue;
            }
            track_taken[track_at] = true;
            person_taken[person_at] = true;
            tracks[track_at].samples.push(Sample {
                frame,
                time_ms: *time_ms,
                person: people[person_at].clone(),
            });
        }
        for (person_at, person) in people.iter().enumerate() {
            if !person_taken[person_at] {
                let id = tracks.len() + 1;
                tracks.push(Track {
                    id,
                    samples: vec![Sample {
                        frame,
                        time_ms: *time_ms,
                        person: person.clone(),
                    }],
                });
            }
        }
    }
    tracks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::people::{Keypoint, KEYPOINTS};

    fn person_at(x: f32, y: f32) -> Person {
        Person {
            keypoints: [Keypoint::default(); KEYPOINTS],
            bbox: [x - 20.0, y - 50.0, x + 20.0, y + 50.0],
            score: 0.9,
        }
    }

    #[test]
    fn two_people_who_walk_past_each_other_keep_their_tracks() {
        let frames: Vec<(u64, Vec<Person>)> = (0..5)
            .map(|step| {
                let t = step as f32 * 12.0;
                (
                    step as u64 * 200,
                    vec![person_at(100.0 + t, 100.0), person_at(300.0 - t, 100.0)],
                )
            })
            .collect();
        let tracks = track_people(&frames);
        assert_eq!(tracks.len(), 2);
        assert!(tracks.iter().all(|track| track.samples.len() == 5));
        assert!(tracks[0].samples[4].person.center().0 > tracks[0].samples[0].person.center().0);
        assert!(tracks[1].samples[4].person.center().0 < tracks[1].samples[0].person.center().0);
    }

    #[test]
    fn a_person_who_leaves_for_a_moment_comes_back_to_the_same_track_but_not_after_a_long_time() {
        let mut frames: Vec<(u64, Vec<Person>)> = vec![(0, vec![person_at(100.0, 100.0)])];
        frames.push((200, vec![]));
        frames.push((400, vec![person_at(105.0, 100.0)]));
        for at in 3..9 {
            frames.push((at * 200, vec![]));
        }
        frames.push((1800, vec![person_at(105.0, 100.0)]));
        let tracks = track_people(&frames);
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].samples.len(), 2);
        assert_eq!(tracks[1].id, 2);
    }
}
