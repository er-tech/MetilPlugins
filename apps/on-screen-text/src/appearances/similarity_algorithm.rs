/// How alike two texts are, from 0 to 1, ignoring capitals, spaces and punctuation: one minus the
/// number of letters to change to turn one into the other over the length of the longer. Two texts
/// with nothing to compare are alike only when both are empty.
pub fn similarity(a: &str, b: &str) -> f32 {
    let (a, b) = (letters(a), letters(b));
    let longest = a.len().max(b.len());
    if longest == 0 {
        return 1.0;
    }
    1.0 - distance(&a, &b) as f32 / longest as f32
}

fn letters(text: &str) -> Vec<char> {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// The number of letters to insert, delete or change to turn `a` into `b`.
fn distance(a: &[char], b: &[char]) -> usize {
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, left) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, right) in b.iter().enumerate() {
            let up = row[j + 1];
            row[j + 1] = (row[j] + 1)
                .min(up + 1)
                .min(diagonal + usize::from(left != right));
            diagonal = up;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_words_are_alike_whatever_the_capitals_and_the_marks() {
        assert_eq!(
            similarity("Saída de emergência!", "saída de EMERGÊNCIA"),
            1.0
        );
        let plain = similarity("Saida de emergencia", "Saída de emergência");
        assert!((plain - (1.0 - 2.0 / 17.0)).abs() < 1e-6, "{plain}");
        assert_eq!(similarity("Hello, World", "hello world"), 1.0);
        assert_eq!(similarity("", "..."), 1.0);
    }

    #[test]
    fn one_wrong_letter_in_a_long_text_is_alike_and_another_text_is_not() {
        assert!(similarity("Atenção saída de emergência", "Atenção saída de emergéncia") > 0.9);
        assert!(similarity("Atenção", "Hello world") < 0.3);
    }
}
