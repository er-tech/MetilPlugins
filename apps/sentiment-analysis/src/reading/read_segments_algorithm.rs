use loomasplugins_plugin_sdk::Failure;
use serde_json::Value;

/// One stretch of speech to rate: where it is and what was said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// The `segments` param: a list of `{ startMs, endMs, text }`, as the speech plugin gives its
/// sentences. A segment with no words in its text is left out and counted. A segment with no time
/// or no text is a 400 that names it. Returns the segments to rate and how many were left out.
pub fn read_segments(params: &Value) -> Result<(Vec<Segment>, usize), Failure> {
    let list = match params.get("segments") {
        Some(Value::Array(list)) => list,
        // A step that did not run gives nothing to rate; a call that leaves it out is a mistake.
        Some(Value::Null) => return Ok((Vec::new(), 0)),
        None => {
            return Err(Failure::status(
                400,
                "segments is missing: give the sentences to rate",
            ))
        }
        Some(other) => {
            return Err(Failure::status(
                400,
                format!("segments must be a list, not {other}"),
            ))
        }
    };
    let mut segments = Vec::with_capacity(list.len());
    let mut left_out = 0;
    for (place, item) in list.iter().enumerate() {
        let time = |field: &str| {
            item.get(field).and_then(Value::as_u64).ok_or_else(|| {
                Failure::status(
                    400,
                    format!("segments[{place}]: {field} is not a time in milliseconds"),
                )
            })
        };
        let (start_ms, end_ms) = (time("startMs")?, time("endMs")?);
        let text = item
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| Failure::status(400, format!("segments[{place}]: text is missing")))?;
        if end_ms < start_ms {
            return Err(Failure::status(
                400,
                format!("segments[{place}]: endMs ({end_ms}) is before startMs ({start_ms})"),
            ));
        }
        if text.trim().is_empty() {
            left_out += 1;
            continue;
        }
        segments.push(Segment {
            start_ms,
            end_ms,
            text: text.trim().to_string(),
        });
    }
    Ok((segments, left_out))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn sentences_are_read_in_order_and_blank_ones_are_counted_and_left_out() {
        let (segments, left_out) = read_segments(&json!({ "segments": [
            { "startMs": 0, "endMs": 900, "text": " I love it. ", "confidence": 0.9 },
            { "startMs": 900, "endMs": 1000, "text": "   " },
            { "startMs": 1000, "endMs": 2000, "text": "Terrible." },
        ] }))
        .unwrap();
        assert_eq!(left_out, 1);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "I love it.");
        assert_eq!((segments[1].start_ms, segments[1].end_ms), (1000, 2000));
    }

    #[test]
    fn nothing_is_an_empty_list_and_a_wrong_segment_is_a_400_that_names_it() {
        assert_eq!(
            read_segments(&json!({ "segments": null })).unwrap().0.len(),
            0
        );
        assert!(read_segments(&json!({}))
            .unwrap_err()
            .message
            .contains("missing"));
        for bad in [
            json!({ "segments": "hello" }),
            json!({ "segments": [{ "startMs": 0, "text": "a" }] }),
            json!({ "segments": [{ "startMs": 0, "endMs": 1 }] }),
            json!({ "segments": [{ "startMs": 5, "endMs": 1, "text": "a" }] }),
        ] {
            let failure = read_segments(&bad).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)), "{bad}");
        }
        let failure =
            read_segments(&json!({ "segments": [{ "startMs": 0, "endMs": 1 }] })).unwrap_err();
        assert!(
            failure.message.contains("segments[0]"),
            "{}",
            failure.message
        );
    }
}
