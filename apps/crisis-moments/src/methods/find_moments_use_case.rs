use metilplugins_plugin_sdk::Failure;
use serde_json::{json, Value};

use super::read_find_options_mapper::read_find_options;
use super::read_timed_list_mapper::{read_timed_list, Needs};
use crate::moments::{
    label_signals, merge_signals, text_signals, volume_signals, Moment, WordList,
};

/// How strong a hint of words of distress is: one word such as "help" is often only a word.
const DISTRESS: (f64, f64) = (0.45, 0.15);
/// How strong a hint of words about harming oneself is: one phrase is enough to look.
const SELF_HARM: (f64, f64) = (0.7, 0.1);
/// How strong a hint of aggressive words is.
const AGGRESSION: (f64, f64) = (0.45, 0.15);

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

/// The `find` action: moments in a video where a person may be in distress or in conflict, for a
/// person to look at. It says what was heard, never what it means.
///
/// Params: `text` (sentences with times), `labels` (sounds found, with times), `volume` (the level
/// over time; it only supports the other two, a loud sound alone is not a moment), and the
/// settings `languages`, `distressWords`, `selfHarmWords`, `aggressionWords`, `soundNames`,
/// `wordsWindowMs`, `loudnessJumpDb`, `mergeGapMs`, `minMs` and `threshold`. At least one of the three lists must be
/// given. A list that is given but empty is fine: it says there was nothing to see, and the result
/// is empty.
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

    let lists = [
        WordList {
            reason: "words of distress",
            words: &options.distress_words,
            one_word: DISTRESS.0,
            each_more: DISTRESS.1,
        },
        WordList {
            reason: "words of distress",
            words: &options.self_harm_words,
            one_word: SELF_HARM.0,
            each_more: SELF_HARM.1,
        },
        WordList {
            reason: "aggressive words",
            words: &options.aggression_words,
            one_word: AGGRESSION.0,
            each_more: AGGRESSION.1,
        },
    ];
    let mut signals = Vec::new();
    signals.extend(text_signals(
        text.as_deref().unwrap_or_default(),
        &lists,
        options.words_window_ms,
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

    fn said(text: &str) -> Value {
        json!({ "text": [{ "startMs": 0, "endMs": 1_000, "text": text }] })
    }

    fn count(params: &Value) -> u64 {
        find_moments(params).unwrap()["count"].as_u64().unwrap()
    }

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
        assert_eq!(moment["label"], "screaming");
        assert_eq!(moment["sources"], json!(["labels", "volume"]));
        assert!(moment["confidence"].as_f64().unwrap() > 0.8);
    }

    #[test]
    fn a_loud_jump_alone_is_never_a_moment() {
        let params = json!({ "volume": [
            { "startMs": 0, "endMs": 1_000, "value": -50 },
            { "startMs": 1_000, "endMs": 2_000, "value": -10 },
            { "startMs": 2_000, "endMs": 3_000, "value": -50 },
        ] });
        assert_eq!(count(&params), 0);
    }

    #[test]
    fn an_alarm_a_siren_or_a_gunshot_is_not_what_it_looks_for() {
        let params = json!({ "labels": [
            { "startMs": 0, "endMs": 2_000, "label": "Fire alarm", "confidence": 0.95 },
            { "startMs": 3_000, "endMs": 5_000, "label": "Siren", "confidence": 0.95 },
            { "startMs": 6_000, "endMs": 7_000, "label": "Gunshot, gunfire", "confidence": 0.95 },
            { "startMs": 8_000, "endMs": 9_000, "label": "Explosion", "confidence": 0.95 },
        ] });
        assert_eq!(count(&params), 0);
    }

    #[test]
    fn one_word_of_distress_stays_under_the_default_threshold_and_two_pass_it() {
        // One word such as "help" is often only a word ("can you help me carry this"): it needs
        // another input, or a lower threshold, to count.
        let one = said("Can you help me carry this");
        assert_eq!(count(&one), 0);
        let mut lenient = one.clone();
        lenient["threshold"] = json!(0.4);
        assert_eq!(count(&lenient), 1);

        let two = said("Help, let go of me");
        assert_eq!(count(&two), 1);
        let mut strict = two.clone();
        strict["threshold"] = json!(0.7);
        assert_eq!(count(&strict), 0);
    }

    #[test]
    fn a_phrase_about_harming_oneself_passes_the_default_threshold_alone() {
        let result = find_moments(&said("I just want to die")).unwrap();
        assert_eq!(result["count"], 1);
        assert_eq!(result["moments"][0]["label"], "words of distress");
        assert!((result["moments"][0]["confidence"].as_f64().unwrap() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn threats_and_insults_make_a_moment_named_aggressive_words() {
        let result = find_moments(&said("Shut up or I will kill you")).unwrap();
        assert_eq!(result["count"], 1);
        assert_eq!(result["moments"][0]["label"], "aggressive words");
        assert_eq!(count(&said("Please shut up for a minute")), 0);
    }

    #[test]
    fn hitting_sounds_alone_do_not_pass_but_with_shouting_they_do() {
        let hit = json!({ "labels": [
            { "startMs": 0, "endMs": 1_000, "label": "Slap, smack", "confidence": 0.8 }
        ] });
        assert_eq!(count(&hit), 0);
        let both = json!({ "labels": [
            { "startMs": 0, "endMs": 1_000, "label": "Slap, smack", "confidence": 0.8 },
            { "startMs": 500, "endMs": 1_500, "label": "Shout", "confidence": 0.7 },
        ] });
        let result = find_moments(&both).unwrap();
        assert_eq!(result["count"], 1);
        assert_eq!(result["moments"][0]["label"], "shouting");
    }

    #[test]
    fn crying_and_fast_breathing_are_found_from_the_sounds() {
        let params = json!({ "labels": [
            { "startMs": 0, "endMs": 4_000, "label": "Crying, sobbing", "confidence": 0.9 },
            { "startMs": 20_000, "endMs": 24_000, "label": "Gasp", "confidence": 0.95 },
        ] });
        let result = find_moments(&params).unwrap();
        assert_eq!(result["moments"][0]["label"], "crying");
        // A gasp keeps 70% of its confidence: 0.665, over the default threshold.
        assert_eq!(result["moments"][1]["label"], "fast breathing");
    }

    #[test]
    fn a_word_and_a_sound_that_agree_pass_and_the_evidence_says_what_was_seen() {
        let params = json!({
            "text": [{ "startMs": 0, "endMs": 2_000, "text": "Help!" }],
            "labels": [{ "startMs": 500, "endMs": 1_500, "label": "Screaming", "confidence": 0.8 }],
        });
        let result = find_moments(&params).unwrap();
        assert_eq!(result["count"], 1);
        let evidence = result["moments"][0]["evidence"].as_array().unwrap();
        let details: Vec<&str> = evidence
            .iter()
            .map(|item| item["detail"].as_str().unwrap())
            .collect();
        assert!(details.contains(&"said \"help\""), "{details:?}");
        assert!(details.contains(&"heard Screaming"), "{details:?}");
    }

    #[test]
    fn words_in_another_language_are_found_only_when_that_language_is_asked_for() {
        let sentence = said("Socorro, não consigo respirar");
        assert_eq!(count(&sentence), 0);
        let mut portuguese = sentence.clone();
        portuguese["languages"] = json!(["pt"]);
        assert_eq!(count(&portuguese), 1);
    }
}
