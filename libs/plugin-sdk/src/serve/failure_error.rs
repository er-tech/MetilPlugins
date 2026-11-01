use serde_json::Value;

/// A call that failed, as the plugin decides. Only the message is required; the code, the retry
/// hint and the details are sent to the host when there are any.
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    pub message: String,
    /// What kind of failure: a number with the meaning of the HTTP status code of that number
    /// (400, 404, 429, 503), or a name of the plugin's own (`MODEL_MISSING`).
    pub code: Option<Value>,
    /// Whether trying again could work. Leave it out when unknown: the host reads the code.
    pub retryable: Option<bool>,
    /// Anything else worth keeping, such as what a wrapped service answered.
    pub details: Option<Value>,
}

impl Failure {
    /// A failure with a message and nothing else.
    pub fn new(message: impl Into<String>) -> Self {
        Failure {
            message: message.into(),
            code: None,
            retryable: None,
            details: None,
        }
    }

    /// A failure with the meaning of an HTTP status code. The host knows what the standard codes
    /// mean (whether a retry could work, for one), so nothing more needs saying.
    pub fn status(code: u16, message: impl Into<String>) -> Self {
        Failure {
            code: Some(Value::from(code)),
            ..Failure::new(message)
        }
    }

    /// A failure with a name of the plugin's own, such as `MODEL_MISSING`.
    pub fn named(code: &str, message: impl Into<String>) -> Self {
        Failure {
            code: Some(Value::from(code)),
            ..Failure::new(message)
        }
    }

    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = Some(retryable);
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// The `error` message for the call with this id.
    pub(crate) fn to_message(&self, id: &Value) -> Value {
        let mut reply = serde_json::json!({ "type": "error", "id": id, "message": self.message });
        if let Some(code) = &self.code {
            reply["code"] = code.clone();
        }
        if let Some(retryable) = self.retryable {
            reply["retryable"] = Value::from(retryable);
        }
        if let Some(details) = &self.details {
            reply["details"] = details.clone();
        }
        reply
    }
}

impl From<String> for Failure {
    fn from(message: String) -> Self {
        Failure::new(message)
    }
}

impl From<&str> for Failure {
    fn from(message: &str) -> Self {
        Failure::new(message)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_failure_sends_only_what_it_was_given() {
        assert_eq!(
            Failure::new("no").to_message(&json!(3)),
            json!({ "type": "error", "id": 3, "message": "no" })
        );
        let full = Failure::status(429, "busy")
            .retryable(true)
            .with_details(json!({ "after": 5 }));
        assert_eq!(
            full.to_message(&json!(1)),
            json!({ "type": "error", "id": 1, "message": "busy", "code": 429,
                    "retryable": true, "details": { "after": 5 } })
        );
        assert_eq!(
            Failure::named("QUOTA", "used up").code,
            Some(json!("QUOTA"))
        );
    }

    #[test]
    fn a_text_is_a_failure_with_a_message() {
        let failure: Failure = "plain".into();
        assert_eq!(failure, Failure::new("plain"));
        assert_eq!(Failure::from(String::from("owned")).message, "owned");
    }
}
