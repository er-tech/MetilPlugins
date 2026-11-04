use serde_json::{json, Value};

use super::find_hits_algorithm::find_hits;
use super::interpret_response_algorithm::interpret_response;
use super::measure_file_use_case::measure_file;
use super::wait_use_case::wait;
use loomasplugins_plugin_sdk::Failure;

/// Runs a method of this plugin and gives its result, or the reason it failed.
///
/// - `find`: `{ durationMs, everyMs?, lengthMs?, label? }` gives `{ hits: [{ startMs, endMs, label }] }`.
/// - `size`: `{ audio }` gives the size of the stored file.
/// - `sleep`: `{ ms }` waits that long, or ends with the error `cancelled` as soon as the call is
///   cancelled. `pid` gives the number of the program's process, so a host can tell copies apart.
/// - `interpret`: `{ response }` decides what a service's answer means (see
///   `interpret_response`): an answer flagged `error`, a status of 400 or more, or `null` is a
///   failure with a code and a retry hint; anything else is the result.
/// - `fail`: always an error, with the code `ALWAYS_FAILS` (a name of the plugin's own). `crash`: ends the program without answering. `hang`: never answers
///   and ignores a cancel. The last three exist so a host can test how it copes with a
///   misbehaving plugin.
///
/// `is_cancelled` says whether the host cancelled this call.
pub fn run_method(
    method: &str,
    params: &Value,
    is_cancelled: &dyn Fn() -> bool,
) -> Result<Value, Failure> {
    match method {
        "find" => {
            let hits = find_hits(
                params["durationMs"].as_u64().unwrap_or(0),
                params["everyMs"].as_u64().unwrap_or(1_000),
                params["lengthMs"].as_u64().unwrap_or(500),
                params["label"].as_str().unwrap_or("hit"),
            );
            Ok(json!({ "hits": hits }))
        }
        "size" => measure_file(params),
        "sleep" => wait(params, is_cancelled).map_err(Failure::from),
        "interpret" => interpret_response(&params["response"]),
        "pid" => Ok(json!({ "pid": std::process::id() })),
        "fail" => Err(Failure {
            message: "this plugin always fails".to_string(),
            code: Some(json!("ALWAYS_FAILS")),
            retryable: Some(false),
            details: None,
        }),
        "crash" => std::process::exit(3),
        "hang" => {
            std::thread::sleep(std::time::Duration::from_secs(60));
            Ok(json!({}))
        }
        other => Err(Failure::status(404, format!("no method {other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_uses_defaults_and_unknown_methods_are_errors() {
        let result = run_method("find", &json!({ "durationMs": 1_200 }), &|| false).unwrap();
        assert_eq!(result["hits"].as_array().unwrap().len(), 2);
        assert_eq!(result["hits"][0]["label"], "hit");

        assert_eq!(
            run_method("nope", &json!({}), &|| false)
                .unwrap_err()
                .message,
            "no method \"nope\""
        );
        let failure = run_method("fail", &json!({}), &|| false).unwrap_err();
        assert_eq!(failure.message, "this plugin always fails");
        assert_eq!(failure.code, Some(json!("ALWAYS_FAILS")));
    }
}
