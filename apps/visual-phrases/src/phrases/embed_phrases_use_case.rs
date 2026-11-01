use tokenizers::Tokenizer;
use metilplugins_plugin_sdk::Failure;

use crate::matching::normalize;
use crate::models::ModelPaths;
use crate::session::{embed_texts, DIMENSIONS};

use super::token_ids_algorithm::{token_ids, LENGTH};

/// The numbers of each phrase, scaled to length 1, one after another (`DIMENSIONS` for each).
pub fn embed_phrases(paths: &ModelPaths, phrases: &[String]) -> Result<Vec<f32>, Failure> {
    let tokenizer = Tokenizer::from_file(&paths.tokenizer).map_err(|error| {
        Failure::status(
            422,
            format!(
                "the tokenizer cannot be read from {}: {error}",
                paths.tokenizer.display()
            ),
        )
    })?;
    let mut ids = Vec::with_capacity(phrases.len() * LENGTH);
    for phrase in phrases {
        ids.extend(token_ids(&tokenizer, phrase).map_err(|message| Failure::status(422, message))?);
    }
    let mut numbers = embed_texts(&paths.text, ids, phrases.len(), LENGTH)?;
    numbers.chunks_mut(DIMENSIONS).for_each(normalize);
    Ok(numbers)
}
