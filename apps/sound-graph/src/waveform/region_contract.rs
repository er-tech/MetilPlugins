use serde_json::Value;
use loomasplugins_plugin_sdk::Failure;

/// A stretch of the sound to mark on the picture: a sentence that was said, a loud moment.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub start_ms: u64,
    pub end_ms: u64,
    pub label: Option<String>,
    pub color: Option<String>,
}

/// Reads the `regions` param: a list of `{ startMs, endMs, label?, color? }`. The `text` of a
/// sentence (as the speech plugin gives it) is taken as the label when there is no `label`, so the
/// sentences of a transcript can be given as they are.
pub fn read_regions(params: &Value) -> Result<Vec<Region>, Failure> {
    let list = match params.get("regions") {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(list)) => list,
        Some(other) => {
            return Err(Failure::status(
                400,
                format!("regions must be a list, not {other}"),
            ))
        }
    };
    if list.len() > 5_000 {
        return Err(Failure::status(
            400,
            format!(
                "regions has {} items; at most 5000 can be drawn",
                list.len()
            ),
        ));
    }
    list.iter()
        .enumerate()
        .map(|(position, item)| {
            let time = |key: &str| {
                item.get(key).and_then(Value::as_u64).ok_or_else(|| {
                    Failure::status(
                        400,
                        format!("regions[{position}].{key} must be a whole number of milliseconds"),
                    )
                })
            };
            let (start_ms, end_ms) = (time("startMs")?, time("endMs")?);
            if end_ms < start_ms {
                return Err(Failure::status(
                    400,
                    format!("regions[{position}] ends ({end_ms}) before it starts ({start_ms})"),
                ));
            }
            let text = |key: &str| item.get(key).and_then(Value::as_str).map(str::to_string);
            Ok(Region {
                start_ms,
                end_ms,
                label: text("label").or_else(|| text("text")),
                color: text("color"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn reads_regions_and_takes_the_text_of_a_sentence_as_its_label() {
        let regions = read_regions(&json!({ "regions": [
            { "startMs": 0, "endMs": 1500, "label": "intro", "color": "#e53e3e" },
            { "startMs": 2000, "endMs": 3000, "text": "A sentence." }
        ] }))
        .unwrap();
        assert_eq!(regions.len(), 2);
        assert_eq!(
            regions[0],
            Region {
                start_ms: 0,
                end_ms: 1500,
                label: Some("intro".into()),
                color: Some("#e53e3e".into())
            }
        );
        assert_eq!(regions[1].label.as_deref(), Some("A sentence."));
        assert!(read_regions(&json!({})).unwrap().is_empty());
        assert!(read_regions(&json!({ "regions": null }))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn mistakes_name_the_item_and_are_a_400() {
        for (regions, wanted) in [
            (json!(5), "regions must be a list"),
            (json!([{ "startMs": 0 }]), "regions[0].endMs"),
            (
                json!([{ "startMs": "a", "endMs": 1 }]),
                "regions[0].startMs",
            ),
            (
                json!([{ "startMs": 5, "endMs": 1 }]),
                "ends (1) before it starts (5)",
            ),
        ] {
            let failure = read_regions(&json!({ "regions": regions })).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)));
            assert!(failure.message.contains(wanted), "{}", failure.message);
        }
    }
}
