use super::similarity_algorithm::similarity;

/// Two readings are the same text when they are at least this alike.
const SAME_TEXT: f32 = 0.8;
/// And in the same place when their boxes overlap by at least this share.
const SAME_PLACE: f32 = 0.3;

/// One line read in one picture, with its box as shares of the picture (0 to 1): `x1, y1, x2, y2`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadLine {
    pub text: String,
    pub confidence: f32,
    pub bbox: [f32; 4],
}

/// What was read in one picture: when it is shown and when it stops (in milliseconds) and the lines.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub time_ms: u64,
    pub end_ms: u64,
    pub lines: Vec<ReadLine>,
}

/// A text that stays on the screen: the same text, in the same place, in pictures that follow one
/// another.
#[derive(Debug, Clone, PartialEq)]
pub struct Appearance {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    /// The mean confidence of the readings.
    pub confidence: f32,
    pub samples: usize,
    /// Where it is, as shares of the picture: `x1, y1, x2, y2`, the box of the first reading.
    pub bbox: [f32; 4],
}

struct Open {
    appearance: Appearance,
    confidence_sum: f32,
    best_confidence: f32,
    last_time_ms: u64,
    last_bbox: [f32; 4],
}

/// Joins the readings, in time order, into appearances. A line that is at least 80% like a text
/// seen in the same place (their boxes overlap by 30% or more) no more than `gap_ms` after it was
/// last seen is the same appearance (so a letter the recognizer gets wrong in one picture does not
/// split it); the text kept is the one the recognizer was surest of. Lines with a confidence below
/// `min_confidence` or fewer than `min_chars` letters or digits are left out. Appearances come in
/// the order they start.
pub fn merge_appearances(
    readings: &[Reading],
    gap_ms: u64,
    min_confidence: f32,
    min_chars: usize,
) -> Vec<Appearance> {
    let mut all: Vec<Open> = Vec::new();
    for reading in readings {
        for line in &reading.lines {
            let letters = line.text.chars().filter(|c| c.is_alphanumeric()).count();
            if line.confidence < min_confidence || letters < min_chars {
                continue;
            }
            let found = all
                .iter_mut()
                .filter(|open| reading.time_ms <= open.last_time_ms + gap_ms)
                .filter(|open| overlap(&open.last_bbox, &line.bbox) >= SAME_PLACE)
                .filter(|open| similarity(&open.appearance.text, &line.text) >= SAME_TEXT)
                .max_by(|a, b| a.last_time_ms.cmp(&b.last_time_ms));
            match found {
                Some(open) => {
                    open.appearance.end_ms = reading.end_ms.max(open.appearance.end_ms);
                    open.appearance.samples += 1;
                    open.confidence_sum += line.confidence;
                    open.last_time_ms = reading.time_ms;
                    open.last_bbox = line.bbox;
                    if line.confidence > open.best_confidence {
                        open.best_confidence = line.confidence;
                        open.appearance.text = line.text.clone();
                    }
                }
                None => all.push(Open {
                    appearance: Appearance {
                        text: line.text.clone(),
                        start_ms: reading.time_ms,
                        end_ms: reading.end_ms,
                        confidence: line.confidence,
                        samples: 1,
                        bbox: line.bbox,
                    },
                    confidence_sum: line.confidence,
                    best_confidence: line.confidence,
                    last_time_ms: reading.time_ms,
                    last_bbox: line.bbox,
                }),
            }
        }
    }
    all.into_iter()
        .map(|mut open| {
            open.appearance.confidence = open.confidence_sum / open.appearance.samples as f32;
            open.appearance
        })
        .collect()
}

/// How much of the smaller box lies inside the other, from 0 to 1.
fn overlap(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let width = (a[2].min(b[2]) - a[0].max(b[0])).max(0.0);
    let height = (a[3].min(b[3]) - a[1].max(b[1])).max(0.0);
    let smaller = ((a[2] - a[0]) * (a[3] - a[1])).min((b[2] - b[0]) * (b[3] - b[1]));
    if smaller <= 0.0 {
        return 0.0;
    }
    width * height / smaller
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(time_ms: u64, lines: &[(&str, f32, [f32; 4])]) -> Reading {
        Reading {
            time_ms,
            end_ms: time_ms + 1000,
            lines: lines
                .iter()
                .map(|(text, confidence, bbox)| ReadLine {
                    text: text.to_string(),
                    confidence: *confidence,
                    bbox: *bbox,
                })
                .collect(),
        }
    }

    const TITLE: [f32; 4] = [0.1, 0.1, 0.6, 0.2];
    const SIGN: [f32; 4] = [0.1, 0.7, 0.5, 0.8];

    #[test]
    fn the_same_text_in_the_same_place_in_following_pictures_is_one_appearance() {
        let readings = [
            reading(0, &[("Capítulo um", 0.9, TITLE)]),
            reading(1000, &[("Capítulo um", 0.95, TITLE), ("Saída", 0.8, SIGN)]),
            reading(2000, &[("Capítulo urn", 0.7, TITLE)]),
            reading(9000, &[("Capítulo um", 0.9, TITLE)]),
        ];
        let appearances = merge_appearances(&readings, 2500, 0.5, 2);
        assert_eq!(appearances.len(), 3, "{appearances:?}");
        let title = &appearances[0];
        assert_eq!((title.start_ms, title.end_ms, title.samples), (0, 3000, 3));
        assert_eq!(title.text, "Capítulo um"); // the surest reading, not the last
        assert!((title.confidence - (0.9 + 0.95 + 0.7) / 3.0).abs() < 1e-6);
        assert_eq!(appearances[1].text, "Saída");
        assert_eq!(appearances[2].start_ms, 9000); // after a gap longer than 2.5 s
    }

    #[test]
    fn a_text_somewhere_else_or_with_other_words_is_another_appearance_and_weak_lines_are_left_out()
    {
        let readings = [
            reading(
                0,
                &[
                    ("Hello world", 0.9, TITLE),
                    ("x", 0.99, SIGN),
                    ("Maybe text", 0.3, SIGN),
                ],
            ),
            reading(
                1000,
                &[("Hello world", 0.9, SIGN), ("Goodbye now", 0.9, TITLE)],
            ),
        ];
        let appearances = merge_appearances(&readings, 2500, 0.5, 2);
        assert_eq!(appearances.len(), 3, "{appearances:?}");
        assert!(appearances
            .iter()
            .all(|a| a.text != "x" && a.text != "Maybe text"));
    }
}
