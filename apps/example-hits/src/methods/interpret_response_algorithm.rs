use serde_json::{json, Value};

use metilplugins_plugin_sdk::Failure;

/// Decides what a wrapped service's answer means. This is the plugin's job: the host never reads a
/// result to find out whether it failed.
///
/// - An answer with `"error": true` is a failure even when its `status` is 200, as some services
///   do. The failure keeps the service's own `code` (a name) when it gave one, and the whole
///   answer as its details.
/// - A `status` of 400 or more is a failure whose code is that status: the host knows that 429 and
///   503 can be tried again and that 400 and 404 cannot, so the plugin says nothing more.
/// - `null` is a failure too, as a 502: the service sits behind this plugin and answered nothing.
/// - Anything else is a success, and the answer is the result.
///
/// A service that says whether a retry could work (`retryable`) is believed over its status.
pub fn interpret_response(response: &Value) -> Result<Value, Failure> {
    if response.is_null() {
        return Err(Failure::status(502, "the service answered nothing"));
    }
    let status = response["status"].as_u64().unwrap_or(200);
    let flagged = response["error"].as_bool().unwrap_or(false);
    if !flagged && status < 400 {
        return Ok(response.clone());
    }

    let code = match response["code"].as_str() {
        Some(name) => json!(name),
        None if status >= 400 => json!(status),
        None => json!("SERVICE_ERROR"),
    };
    Err(Failure {
        message: response["message"]
            .as_str()
            .map_or_else(|| format!("the service answered {status}"), str::to_string),
        code: Some(code),
        retryable: response["retryable"].as_bool(),
        details: Some(response.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_200_with_error_true_is_a_failure_that_keeps_what_the_service_said() {
        let response = json!({ "status": 200, "error": true, "message": "quota", "code": "QUOTA" });
        let failure = interpret_response(&response).unwrap_err();
        assert_eq!(failure.message, "quota");
        assert_eq!(failure.code, Some(json!("QUOTA")));
        assert_eq!(failure.retryable, None);
        assert_eq!(failure.details, Some(response));

        // With no code of its own, a flagged answer gets a name; it is never passed off as a 200.
        let unnamed = interpret_response(&json!({ "status": 200, "error": true })).unwrap_err();
        assert_eq!(unnamed.code, Some(json!("SERVICE_ERROR")));
    }

    #[test]
    fn an_error_status_is_the_code_and_the_host_knows_what_it_means() {
        let busy = interpret_response(&json!({ "status": 429 })).unwrap_err();
        assert_eq!(busy.code, Some(json!(429)));
        assert_eq!(busy.retryable, None);
        let bad = interpret_response(&json!({ "status": 400 })).unwrap_err();
        assert_eq!(bad.code, Some(json!(400)));
        assert_eq!(bad.message, "the service answered 400");
    }

    #[test]
    fn the_service_saying_whether_to_retry_is_passed_on() {
        let hopeless =
            interpret_response(&json!({ "status": 503, "retryable": false })).unwrap_err();
        assert_eq!(hopeless.retryable, Some(false));
    }

    #[test]
    fn null_is_a_failure_and_a_normal_answer_is_the_result() {
        let empty = interpret_response(&Value::Null).unwrap_err();
        assert_eq!(empty.code, Some(json!(502)));
        let fine = json!({ "status": 200, "hits": [1] });
        assert_eq!(interpret_response(&fine).unwrap(), fine);
    }
}
