use serde_json::Value;
use metilplugins_plugin_sdk::Failure;

use crate::moments::TimedItem;

/// The field an input list must have in every item besides `startMs` and `endMs`.
#[derive(Debug, Clone, Copy)]
pub enum Needs {
    /// `text`, for a `timedText` list.
    Text,
    /// `label`, for a `timedLabel` list.
    Label,
    /// `value`, for a `series` list.
    Value,
}

impl Needs {
    fn key(self) -> &'static str {
        match self {
            Needs::Text => "text",
            Needs::Label => "label",
            Needs::Value => "value",
        }
    }
}

fn whole_ms(item: &Value, input: &str, index: usize, key: &str) -> Result<u64, Failure> {
    item.get(key).and_then(Value::as_u64).ok_or_else(|| {
        Failure::status(
            400,
            format!("{input}[{index}] needs {key} as a whole number of milliseconds"),
        )
    })
}

/// Reads one input list into items. Returns `None` when the input is missing or `null`.
///
/// # Errors
/// A 400 failure that names the input and the position of the item when the input is not a list,
/// or an item is not an object, or an item lacks `startMs`, `endMs` or the field it needs.
pub fn read_timed_list(
    params: &Value,
    input: &str,
    needs: Needs,
) -> Result<Option<Vec<TimedItem>>, Failure> {
    let items = match params.get(input) {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::Array(items)) => items,
        Some(other) => {
            return Err(Failure::status(
                400,
                format!("{input} must be a list, not {other}"),
            ))
        }
    };
    let key = needs.key();

    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let start_ms = whole_ms(item, input, index, "startMs")?;
            let end_ms = whole_ms(item, input, index, "endMs")?;
            let field = item.get(key);
            let text_of = |value: Option<&Value>| value.and_then(Value::as_str).map(str::to_owned);
            let valid = match needs {
                Needs::Text | Needs::Label => field.is_some_and(Value::is_string),
                Needs::Value => field.is_some_and(Value::is_number),
            };
            if !valid {
                return Err(Failure::status(
                    400,
                    format!("{input}[{index}] needs {key}"),
                ));
            }
            Ok(TimedItem {
                start_ms,
                end_ms,
                text: if matches!(needs, Needs::Text) {
                    text_of(field)
                } else {
                    None
                },
                label: if matches!(needs, Needs::Label) {
                    text_of(field)
                } else {
                    None
                },
                confidence: item.get("confidence").and_then(Value::as_f64),
                value: if matches!(needs, Needs::Value) {
                    field.and_then(Value::as_f64)
                } else {
                    None
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_missing_input_is_none_and_an_empty_list_is_some() {
        assert_eq!(
            read_timed_list(&json!({}), "text", Needs::Text).unwrap(),
            None
        );
        assert_eq!(
            read_timed_list(&json!({ "text": null }), "text", Needs::Text).unwrap(),
            None
        );
        assert_eq!(
            read_timed_list(&json!({ "text": [] }), "text", Needs::Text).unwrap(),
            Some(vec![])
        );
    }

    #[test]
    fn items_keep_their_times_and_the_field_their_type_adds() {
        let params = json!({ "labels": [{ "startMs": 1, "endMs": 2, "label": "Siren", "confidence": 0.9 }] });
        let items = read_timed_list(&params, "labels", Needs::Label)
            .unwrap()
            .unwrap();
        assert_eq!(items[0].label.as_deref(), Some("Siren"));
        assert_eq!(items[0].confidence, Some(0.9));
        assert_eq!((items[0].start_ms, items[0].end_ms), (1, 2));
    }

    #[test]
    fn a_bad_item_is_a_400_that_names_the_input_and_the_position() {
        let params = json!({ "volume": [{ "startMs": 0, "endMs": 1, "value": -20 }, { "startMs": 1, "endMs": 2 }] });
        let failure = read_timed_list(&params, "volume", Needs::Value).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        assert!(failure.message.contains("volume[1]"), "{}", failure.message);
    }

    #[test]
    fn an_input_that_is_not_a_list_is_a_400() {
        let failure =
            read_timed_list(&json!({ "text": "hello" }), "text", Needs::Text).unwrap_err();
        assert!(
            failure.message.contains("must be a list"),
            "{}",
            failure.message
        );
    }
}
