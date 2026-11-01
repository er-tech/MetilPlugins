use serde_json::Value;

use crate::serve::Failure;

/// A whole number param in `min..=max`, or `default` when it is not given (or `null`).
pub fn whole_number(
    params: &Value,
    key: &str,
    default: u64,
    min: u64,
    max: u64,
) -> Result<u64, Failure> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(value) => value
            .as_u64()
            .filter(|number| (min..=max).contains(number))
            .ok_or_else(|| {
                Failure::status(
                    400,
                    format!("{key} must be a whole number from {min} to {max}, not {value}"),
                )
            }),
    }
}

/// A number param in `min..=max`, or `default` when it is not given.
pub fn number(params: &Value, key: &str, default: f64, min: f64, max: f64) -> Result<f64, Failure> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(value) => value
            .as_f64()
            .filter(|number| (min..=max).contains(number))
            .ok_or_else(|| {
                Failure::status(
                    400,
                    format!("{key} must be a number from {min} to {max}, not {value}"),
                )
            }),
    }
}

/// A true-or-false param, or `default` when it is not given.
pub fn flag(params: &Value, key: &str, default: bool) -> Result<bool, Failure> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(other) => Err(Failure::status(
            400,
            format!("{key} must be true or false, not {other}"),
        )),
    }
}

/// A text param of at most `max_chars` characters, or `default` when it is not given.
pub fn text(
    params: &Value,
    key: &str,
    default: Option<&str>,
    max_chars: usize,
) -> Result<Option<String>, Failure> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(default.map(str::to_string)),
        Some(Value::String(value)) if value.chars().count() <= max_chars => Ok(Some(value.clone())),
        Some(other) => Err(Failure::status(
            400,
            format!("{key} must be a text of at most {max_chars} characters, not {other}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_param_not_given_is_its_default_and_one_outside_its_range_is_a_400() {
        let params = json!({ "windowMs": 500, "bad": 5, "text": "x", "on": true, "level": -30.5, "title": "Hi" });
        assert_eq!(
            whole_number(&params, "windowMs", 1000, 10, 60_000).unwrap(),
            500
        );
        assert_eq!(
            whole_number(&params, "missing", 1000, 10, 60_000).unwrap(),
            1000
        );
        for key in ["bad", "text"] {
            let failure = whole_number(&params, key, 1, 10, 20).unwrap_err();
            assert_eq!(failure.code, Some(json!(400)), "{key}");
            assert!(failure
                .message
                .starts_with(&format!("{key} must be a whole number from 10 to 20")));
        }
        assert_eq!(number(&params, "level", -50.0, -100.0, 0.0).unwrap(), -30.5);
        assert_eq!(number(&params, "none", -50.0, -100.0, 0.0).unwrap(), -50.0);
        assert!(number(&params, "bad", 0.0, -100.0, 0.0).is_err());
        assert!(flag(&params, "on", false).unwrap());
        assert!(flag(&params, "off", true).unwrap());
        assert!(flag(&params, "text", true).is_err());
        assert_eq!(
            text(&params, "title", None, 10).unwrap().as_deref(),
            Some("Hi")
        );
        assert_eq!(
            text(&params, "none", Some("fallback"), 10)
                .unwrap()
                .as_deref(),
            Some("fallback")
        );
        assert_eq!(text(&params, "none", None, 10).unwrap(), None);
        assert!(text(&params, "title", None, 1).is_err());
        assert!(text(&params, "on", None, 10).is_err());
    }
}
