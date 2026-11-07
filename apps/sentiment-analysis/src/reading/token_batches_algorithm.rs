use tokenizers::Tokenizer;

/// How many texts the model reads at once.
const BATCH_SIZE: usize = 16;
/// The longest text the model takes, in pieces; a longer one is cut (and still ends with its end
/// piece).
const MAX_LENGTH: usize = 512;

/// Some texts as the model takes them: the pieces of each, padded to the longest of the batch, and
/// a mask that is 1 where there is a piece and 0 where there is padding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub count: usize,
    pub length: usize,
    pub ids: Vec<i64>,
    pub mask: Vec<i64>,
}

/// Cuts each of `texts` into pieces and groups them in batches of `BATCH_SIZE`, in order. A text
/// that cannot be cut is an error that names it.
pub fn token_batches(tokenizer: &Tokenizer, texts: &[&str]) -> Result<Vec<Batch>, String> {
    let pad = i64::from(tokenizer.token_to_id("[PAD]").unwrap_or(0));
    let mut rows = Vec::with_capacity(texts.len());
    for text in texts {
        let encoded = tokenizer
            .encode(*text, true)
            .map_err(|error| format!("{text:?} could not be cut into pieces: {error}"))?;
        let mut ids: Vec<i64> = encoded.get_ids().iter().map(|id| i64::from(*id)).collect();
        if ids.len() > MAX_LENGTH {
            let end = ids[ids.len() - 1];
            ids.truncate(MAX_LENGTH - 1);
            ids.push(end);
        }
        rows.push(ids);
    }
    Ok(rows
        .chunks(BATCH_SIZE)
        .map(|chunk| pad_batch(chunk, pad))
        .collect())
}

fn pad_batch(rows: &[Vec<i64>], pad: i64) -> Batch {
    let length = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut ids = Vec::with_capacity(rows.len() * length);
    let mut mask = Vec::with_capacity(rows.len() * length);
    for row in rows {
        ids.extend(row);
        ids.extend(std::iter::repeat_n(pad, length - row.len()));
        mask.extend(std::iter::repeat_n(1, row.len()));
        mask.extend(std::iter::repeat_n(0, length - row.len()));
    }
    Batch {
        count: rows.len(),
        length,
        ids,
        mask,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn rows_of_different_length_are_padded_and_masked() {
        let batch = pad_batch(&[vec![101, 5, 102], vec![101, 102]], 0);
        assert_eq!((batch.count, batch.length), (2, 3));
        assert_eq!(batch.ids, [101, 5, 102, 101, 102, 0]);
        assert_eq!(batch.mask, [1, 1, 1, 1, 1, 0]);
    }

    /// The tokenizer, when it is already on this computer (the first real run downloads it).
    fn cached_tokenizer() -> Option<Tokenizer> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()?;
        Tokenizer::from_file(
            PathBuf::from(home).join(".loomas/models/sentiment-analysis/tokenizer.json"),
        )
        .ok()
    }

    #[test]
    fn real_texts_are_cut_batched_and_a_long_text_is_shortened() {
        let Some(tokenizer) = cached_tokenizer() else {
            eprintln!("skipped: the tokenizer is not downloaded yet");
            return;
        };
        let long = "muito bom ".repeat(400);
        let texts: Vec<&str> = std::iter::repeat_n("I love it", 20)
            .chain([long.as_str()])
            .collect();
        let batches = token_batches(&tokenizer, &texts).unwrap();
        assert_eq!(batches.iter().map(|b| b.count).collect::<Vec<_>>(), [16, 5]);
        assert_eq!(batches[1].length, MAX_LENGTH);
        assert_eq!(batches[1].ids.len(), 5 * MAX_LENGTH);
    }
}
