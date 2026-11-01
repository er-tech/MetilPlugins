use serde_json::{json, Value};
use metilplugins_plugin_sdk::Failure;

use super::read_find_options_mapper::read_find_options;
use super::read_timed_list_mapper::{read_timed_list, Needs};
use crate::moments::{label_signals, merge_signals, text_signals, volume_signals, Moment};

/// One moment as the result lists it: a `timedLabel` item with the evidence added.
fn moment_value(moment: &Moment) -> Value {
    let mut sources: Vec<&str> = moment
        .signals
        .iter()
        .map(|signal| signal.source.name())
        .collect();
    sources.sort_unstable();
    sources.dedup();
    json!({
        "startMs": moment.start_ms,
        "endMs": moment.end_ms,
        "label": moment.reason,
        "confidence": moment.confidence,
        "sources": sources,
        "evidence": moment.signals.iter().map(|signal| json!({
            "source": signal.source.name(),
            "detail": signal.detail,
        })).collect::<Vec<_>>(),
    })
}

/// The `find` action: moments in a video that a person should look at.
///
/// Params: `text` (sentences with times), `labels` (sounds or things found, with times), `volume`
/// (the level over time), and the settings `keywords`, `soundNames`, `loudnessJumpDb`,
/// `mergeGapMs`, `minMs` and `threshold`. At least one of the three lists must be given. A list
/// that is given but empty is fine: it says there was nothing to see, and the result is empty.
///
/// # Errors
/// A 400 failure when no list is given, or a param is wrong.
pub fn find_moments(params: &Value) -> Result<Value, Failure> {
    let options = read_find_options(params)?;
    let text = read_timed_list(params, "text", Needs::Text)?;
    let labels = read_timed_list(params, "labels", Needs::Label)?;
    let volume = read_timed_list(params, "volume", Needs::Value)?;
    if text.is_none() && labels.is_none() && volume.is_none() {
        return Err(Failure::status(
            400,
            "give at least one of text, labels or volume (an empty list is fine)",
        ));
    }

    let mut signals = Vec::new();
    signals.extend(text_signals(
        text.as_deref().unwrap_or_default(),
        &options.keywords,
    ));
    signals.extend(label_signals(
        labels.as_deref().unwrap_or_default(),
        &options.sound_names,
    ));
    signals.extend(volume_signals(
        volume.as_deref().unwrap_or_default(),
        options.loudness_jump_db,
    ));

    let moments = merge_signals(
        signals,
        options.merge_gap_ms,
        options.min_ms,
        options.threshold,
    );
    Ok(json!({
        "count": moments.len(),
        "moments": moments.iter().map(moment_value).collect::<Vec<_>>(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_list_at_all_is_a_400_but_empty_lists_give_an_empty_result() {
        let failure = find_moments(&json!({})).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let result = find_moments(&json!({ "text": [], "labels": [] })).unwrap();
        assert_eq!(result["count"], 0);
    }

    #[test]
    fn a_scream_and_a_loud_jump_at_the_same_time_make_one_strong_moment() {
        let params = json!({
            "labels": [{ "startMs": 10_000, "endMs": 12_000, "label": "Screaming", "confidence": 0.8 }],
            "volume": [
                { "startMs": 0, "endMs": 1_000, "value": -50 },
                { "startMs": 1_000, "endMs": 2_000, "value": -50 },
                { "startMs": 10_000, "endMs": 11_000, "value": -20 },
                { "startMs": 11_000, "endMs": 12_000, "value": -50 },
            ],
        });
        let result = find_moments(&params).unwrap();
        assert_eq!(result["count"], 1);
        let moment = &result["moments"][0];
        assert_eq!(moment["label"], "shouting");
        assert_eq!(moment["sources"], json!(["labels", "volume"]));
        assert!(moment["confidence"].as_f64().unwrap() > 0.8);
    }

    #[test]
    fn one_keyword_alone_passes_the_default_threshold_but_not_a_stricter_one() {
        let params =
            json!({ "text": [{ "startMs": 0, "endMs": 1_000, "text": "call the police" }] });
        assert_eq!(find_moments(&params).unwrap()["count"], 1);
        let mut strict = params.clone();
        strict["threshold"] = json!(0.7);
        assert_eq!(find_moments(&strict).unwrap()["count"], 0);
    }

    #[test]
    fn the_evidence_says_what_was_seen() {
        let params = json!({ "text": [{ "startMs": 0, "endMs": 1_000, "text": "Help!" }] });
        let result = find_moments(&params).unwrap();
        assert_eq!(
            result["moments"][0]["evidence"][0]["detail"],
            "said \"help\""
        );
    }
}
