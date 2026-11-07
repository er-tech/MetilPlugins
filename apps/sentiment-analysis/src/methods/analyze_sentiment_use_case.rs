use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use serde_json::{json, Value};
use tokenizers::Tokenizer;

use crate::models::locate_models;
use crate::reading::{read_segments, token_batches};
use crate::scoring::{overall, score_logits, Scored};
use crate::session::{classify, device_report};

use super::read_analyze_options_mapper::read_analyze_options;

/// The `analyze` action: how positive or negative each stretch of speech is.
///
/// Params: `segments` (a list of `{ startMs, endMs, text }`, as `speech-to-text` gives its
/// sentences), `model` and `device` (see `read_analyze_options`). It gives
/// `{ segments, overall, skipped, model, device }`. Each segment is the one given with `label`
/// (`positive`, `neutral` or `negative`), `confidence`, `score` (-1 to 1) and the three
/// `probabilities`. A segment with no words is skipped and counted. No segments is a success with
/// an empty list.
pub fn analyze_sentiment(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_analyze_options(params)?;
    // A call that asked for `auto` runs where the host grants; the default stays the processor.
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
    let (segments, skipped) = read_segments(params)?;
    if segments.is_empty() {
        return Ok(json!({
            "segments": [],
            "overall": overall(&[]),
            "skipped": skipped,
            "model": options.model.name,
            "device": Value::Null,
        }));
    }
    let paths = locate_models(options.model, cancel)?;
    let tokenizer = Tokenizer::from_file(&paths.tokenizer).map_err(|error| {
        Failure::status(
            422,
            format!(
                "the tokenizer cannot be read from {}: {error}",
                paths.tokenizer.display()
            ),
        )
    })?;
    let texts: Vec<&str> = segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect();
    let batches =
        token_batches(&tokenizer, &texts).map_err(|message| Failure::status(422, message))?;

    let mut scored: Vec<Scored> = Vec::with_capacity(segments.len());
    for batch in &batches {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!(
            "{} of {} texts read",
            scored.len(),
            segments.len()
        ));
        let logits = classify(&paths.model, options.device, batch)?;
        scored.extend(score_logits(&logits));
    }
    let rated: Vec<(u64, Scored)> = segments
        .iter()
        .zip(&scored)
        .map(|(segment, scored)| (segment.end_ms - segment.start_ms, *scored))
        .collect();
    let items: Vec<Value> = segments
        .iter()
        .zip(&scored)
        .map(|(segment, scored)| {
            let mut item = json!({
                "startMs": segment.start_ms,
                "endMs": segment.end_ms,
                "text": segment.text,
            });
            if let (Some(item), Some(extra)) = (item.as_object_mut(), scored.to_value().as_object())
            {
                item.extend(extra.clone());
            }
            item
        })
        .collect();
    Ok(json!({
        "segments": items,
        "overall": overall(&rated),
        "skipped": skipped,
        "model": options.model.name,
        "device": device_report().map(|report| report.to_value()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_segments_is_a_success_with_an_empty_list_and_nothing_downloaded() {
        let result = analyze_sentiment(&json!({ "segments": [] }), &Cancel::default()).unwrap();
        assert_eq!(result["segments"], json!([]));
        assert_eq!(result["overall"]["segments"], 0);
    }

    #[test]
    fn a_wrong_param_is_refused_before_anything_is_downloaded() {
        let failure = analyze_sentiment(
            &json!({ "segments": [{ "startMs": 0, "endMs": 1, "text": "a" }], "model": "unicorn" }),
            &Cancel::default(),
        )
        .unwrap_err();
        assert!(failure.message.contains("unicorn"), "{}", failure.message);
    }
}
