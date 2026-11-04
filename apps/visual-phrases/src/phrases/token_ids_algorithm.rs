use tokenizers::Tokenizer;

/// How many pieces the text model takes for a phrase: longer phrases are cut, shorter ones padded.
pub const LENGTH: usize = 64;
/// The piece that ends a phrase and fills the rest (`</s>`).
const END: i64 = 1;

/// The pieces of a phrase as the text model takes them: lower case (the model was trained on it),
/// `LENGTH` of them, the end piece after the last word and padding after that. A phrase of more
/// than `LENGTH` pieces is cut and still ends with the end piece.
pub fn token_ids(tokenizer: &Tokenizer, phrase: &str) -> Result<Vec<i64>, String> {
    let encoded = tokenizer
        .encode(phrase.to_lowercase(), true)
        .map_err(|error| format!("{phrase:?} could not be cut into pieces: {error}"))?;
    let mut ids: Vec<i64> = encoded.get_ids().iter().map(|id| i64::from(*id)).collect();
    if ids.len() > LENGTH {
        ids.truncate(LENGTH - 1);
        ids.push(END);
    }
    ids.resize(LENGTH, END);
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The tokenizer, when it is already on this computer (the first real run downloads it).
    fn cached_tokenizer() -> Option<Tokenizer> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()?;
        let file = PathBuf::from(home).join(".loomas/models/visual-phrases/tokenizer.json");
        Tokenizer::from_file(file).ok()
    }

    #[test]
    fn a_phrase_is_lower_case_pieces_then_the_end_piece_then_padding() {
        let Some(tokenizer) = cached_tokenizer() else {
            eprintln!("skipped: the tokenizer is not downloaded yet");
            return;
        };
        let ids = token_ids(&tokenizer, "A Photo of Two Cats").unwrap();
        assert_eq!(ids.len(), LENGTH);
        assert_eq!(&ids[..8], &[262, 266, 1304, 267, 395, 6473, 1, 1]);
        assert!(ids[6..].iter().all(|id| *id == 1));
    }

    #[test]
    fn a_phrase_that_is_too_long_is_cut_and_still_ends_with_the_end_piece() {
        let Some(tokenizer) = cached_tokenizer() else {
            eprintln!("skipped: the tokenizer is not downloaded yet");
            return;
        };
        let ids = token_ids(&tokenizer, &"two cats ".repeat(100)).unwrap();
        assert_eq!(ids.len(), LENGTH);
        assert_eq!(ids[LENGTH - 1], 1);
        assert_eq!(ids[0], 395);
    }
}
