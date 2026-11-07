use loomasplugins_picture_ops::crop_to;

use super::text_box_contract::TextBox;

/// The recognizer takes lines this high, and as wide as they come out when the box keeps its shape
/// (no narrower than 16 and no wider than 1280).
pub const LINE_HEIGHT: usize = 48;
const MIN_WIDTH: usize = 16;
const MAX_WIDTH: usize = 1280;

/// A line of text as the recognizer takes it.
#[derive(Debug, Clone, PartialEq)]
pub struct LineInput {
    pub tensor: Vec<f32>,
    pub width: usize,
}

/// The line in `text_box` cut out of the picture and scaled to the height the recognizer takes, as
/// blue, green and red numbers from -1 to 1, one plane after another.
pub fn prepare_line_input(rgb: &[u8], size: (usize, usize), text_box: &TextBox) -> LineInput {
    let (box_width, box_height) = (
        (text_box.x2 - text_box.x1).max(1.0),
        (text_box.y2 - text_box.y1).max(1.0),
    );
    let width =
        ((LINE_HEIGHT as f32 * box_width / box_height).ceil() as usize).clamp(MIN_WIDTH, MAX_WIDTH);
    let centre = (
        (text_box.x1 + text_box.x2) / 2.0,
        (text_box.y1 + text_box.y2) / 2.0,
    );
    let line = crop_to(
        rgb,
        size.0,
        size.1,
        centre,
        (box_width, box_height),
        width,
        LINE_HEIGHT,
    );
    let plane = width * LINE_HEIGHT;
    let mut tensor = vec![0.0f32; 3 * plane];
    for at in 0..plane {
        for (channel, source) in [2usize, 1, 0].into_iter().enumerate() {
            tensor[channel * plane + at] = (f32::from(line[at * 3 + source]) / 255.0 - 0.5) / 0.5;
        }
    }
    LineInput { tensor, width }
}

/// The characters the recognizer knows, in the order of its answer: a blank first, then one for each
/// line of the dictionary, then a space.
pub fn load_characters(dictionary: &str) -> Vec<String> {
    let mut characters = vec![String::new()];
    characters.extend(dictionary.lines().map(str::to_string));
    characters.push(" ".to_string());
    characters
}

/// The text in the answer of the recognizer: `chances` holds, for each step along the line, how
/// likely each of `classes` characters is. The likeliest character of each step is taken, a blank
/// separates letters, and the same character in a row is one. Gives the text and the mean chance of
/// the characters that were kept (0 when there are none).
pub fn decode_ctc(chances: &[f32], classes: usize, characters: &[String]) -> (String, f32) {
    if classes == 0 {
        return (String::new(), 0.0);
    }
    let mut text = String::new();
    let mut kept = Vec::new();
    let mut last = 0;
    for step in chances.chunks_exact(classes) {
        let (best, chance) =
            step.iter()
                .copied()
                .enumerate()
                .fold((0, f32::NEG_INFINITY), |top, (at, value)| {
                    if value > top.1 {
                        (at, value)
                    } else {
                        top
                    }
                });
        if best != 0 && best != last {
            if let Some(character) = characters.get(best) {
                text.push_str(character);
                kept.push(chance);
            }
        }
        last = best;
    }
    let confidence = if kept.is_empty() {
        0.0
    } else {
        kept.iter().sum::<f32>() / kept.len() as f32
    };
    (repair_utf8_pairs(&text), confidence)
}

/// The dictionaries of the Latin recognizers hold the two characters that spell a letter in UTF-8
/// read as Latin-1 (`Ã` and a soft hyphen for `í`), and the network sometimes writes a letter that
/// way. A character from U+00C2 or U+00C3 followed by one from U+0080 to U+00BF is taken for the
/// two bytes of one letter and put back as that letter.
fn repair_utf8_pairs(text: &str) -> String {
    let mut repaired = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(first) = characters.next() {
        let second = characters.peek().copied();
        match (first, second) {
            ('\u{c2}' | '\u{c3}', Some(next)) if ('\u{80}'..='\u{bf}').contains(&next) => {
                let letter = ((u32::from(first) & 0x1f) << 6) | (u32::from(next) & 0x3f);
                repaired.push(char::from_u32(letter).unwrap_or(first));
                characters.next();
            }
            _ => repaired.push(first),
        }
    }
    repaired
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_letter_written_as_two_latin_1_characters_is_put_back_and_real_text_is_left_alone() {
        use super::repair_utf8_pairs;
        assert_eq!(
            repair_utf8_pairs("sa\u{c3}\u{ad}da, \u{c3}\u{a7}\u{c3}\u{a3}o"),
            "sa\u{ed}da, \u{e7}\u{e3}o"
        );
        assert_eq!(
            repair_utf8_pairs("\u{c3}gua \u{c3}\u{81}rea \u{c3}"),
            "\u{c3}gua \u{c1}rea \u{c3}"
        );
        assert_eq!(repair_utf8_pairs("Ol\u{e1}, mundo"), "Ol\u{e1}, mundo");
    }

    use super::*;

    #[test]
    fn the_characters_are_a_blank_the_dictionary_and_a_space() {
        let characters = load_characters("a\nb\nç\n");
        assert_eq!(characters, ["", "a", "b", "ç", " "]);
    }

    #[test]
    fn the_text_is_the_likeliest_character_of_each_step_without_blanks_and_repeats() {
        let characters = load_characters("a\nb\nc");
        let step = |best: usize| {
            let mut row = vec![0.05f32; 5];
            row[best] = 0.9;
            row
        };
        // a a blank b b c space blank c
        let chances: Vec<f32> = [1, 1, 0, 2, 2, 3, 4, 0, 3]
            .into_iter()
            .flat_map(step)
            .collect();
        let (text, confidence) = decode_ctc(&chances, 5, &characters);
        assert_eq!(text, "abc c");
        assert!((confidence - 0.9).abs() < 1e-6);
        assert_eq!(decode_ctc(&[], 5, &characters), (String::new(), 0.0));
    }

    #[test]
    fn a_line_is_scaled_to_48_high_and_keeps_its_shape_within_the_limits() {
        let rgb = vec![255u8; 200 * 100 * 3];
        let wide = TextBox {
            x1: 0.0,
            y1: 40.0,
            x2: 192.0,
            y2: 60.0,
            score: 0.9,
        };
        let input = prepare_line_input(&rgb, (200, 100), &wide);
        assert_eq!(input.width, 461); // 48 * 192 / 20, rounded up
        assert_eq!(input.tensor.len(), 3 * 461 * 48);
        assert!(input.tensor.iter().all(|v| (*v - 1.0).abs() < 1e-4));
        let tiny = TextBox {
            x1: 10.0,
            y1: 10.0,
            x2: 12.0,
            y2: 40.0,
            score: 0.9,
        };
        assert_eq!(prepare_line_input(&rgb, (200, 100), &tiny).width, 16);
    }
}
