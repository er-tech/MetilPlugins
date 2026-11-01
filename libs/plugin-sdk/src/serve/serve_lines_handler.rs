use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use serde_json::{json, Value};

use super::cancel_token_model::Cancel;
use super::failure_error::Failure;

/// The protocol version this SDK speaks.
const PROTOCOL: u64 = 2;

/// What a plugin is: a function of the action called, its params and a way to learn it was cancelled.
pub trait Handler:
    Fn(&str, &Value, &Cancel) -> Result<Value, Failure> + Send + Sync + 'static
{
}
impl<T> Handler for T where
    T: Fn(&str, &Value, &Cancel) -> Result<Value, Failure> + Send + Sync + 'static
{
}

/// Serves the protocol on standard input and output until the host says `shutdown` or closes the
/// input. This is what a plugin's `main` calls.
pub fn serve_stdio(handler: impl Handler) {
    let input = std::io::stdin();
    serve_lines(input.lock(), std::io::stdout(), handler);
}

/// Reads one JSON message per line from `input` and answers on `output`, until `shutdown` or the
/// end of the input. Lines that are not JSON, and messages of a type it does not know, are skipped.
///
/// Every call runs on its own thread. At the end of the input the calls still running are waited
/// for; at `shutdown` they are abandoned.
pub fn serve_lines(
    input: impl BufRead,
    output: impl Write + Send + 'static,
    handler: impl Handler,
) {
    let output = Arc::new(Mutex::new(output));
    let handler = Arc::new(handler);
    let running: Arc<Mutex<HashMap<u64, Cancel>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut calls = Vec::new();
    // The folder the host gave this plugin to write media in, from `hello`.
    let mut work_dir: Option<std::path::PathBuf> = None;
    // The device the host grants for models, from the `gpu` capability of `hello`.
    let mut gpu_device: Option<String> = None;

    for line in input.lines().map_while(Result::ok) {
        let Ok(message) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        match message["type"].as_str() {
            Some("hello") => {
                work_dir = message["workDir"].as_str().map(std::path::PathBuf::from);
                gpu_device = message["capabilities"]["gpu"]["device"]
                    .as_str()
                    .map(str::to_string);
                if !send(&output, &ready()) {
                    return;
                }
            }
            Some("call") => {
                // Registered here, in message order, so a cancel that follows cannot be missed.
                let id = message["id"].clone();
                let cancel = {
                    let output = Arc::clone(&output);
                    Cancel::for_call(
                        move |note| {
                            send(
                                &output,
                                &json!({ "type": "progress", "id": id, "message": note }),
                            );
                        },
                        work_dir.clone(),
                        gpu_device.clone(),
                    )
                };
                if let Some(id) = message["id"].as_u64() {
                    lock(&running).insert(id, cancel.clone());
                }
                let (output, handler, running) = (
                    Arc::clone(&output),
                    Arc::clone(&handler),
                    Arc::clone(&running),
                );
                calls.push(thread::spawn(move || {
                    let reply = answer(&message, &cancel, handler.as_ref());
                    if let Some(id) = message["id"].as_u64() {
                        lock(&running).remove(&id);
                    }
                    send(&output, &reply);
                }));
            }
            Some("cancel") => {
                if let Some(cancel) = message["id"]
                    .as_u64()
                    .and_then(|id| lock(&running).get(&id).cloned())
                {
                    cancel.cancel();
                }
            }
            Some("shutdown") => return,
            _ => {}
        }
    }
    for call in calls {
        let _ = call.join();
    }
}

/// The answer to `hello`: the protocol and what the SDK can do for a plugin, which is to read the
/// sound and the pictures the host decodes (`wants`), to say a call is alive (`progress`) and to
/// take the graphics card the host grants (`gpu`).
fn ready() -> Value {
    json!({
        "type": "ready",
        "protocol": PROTOCOL,
        "capabilities": { "wants": { "media": ["audio", "video"] }, "progress": {}, "gpu": {} }
    })
}

fn answer(call: &Value, cancel: &Cancel, handler: &impl Handler) -> Value {
    let id = call["id"].clone();
    let action = call["method"].as_str().unwrap_or_default();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        handler(action, &call["params"], cancel)
    }));
    match outcome {
        Ok(Ok(result)) => json!({ "type": "result", "id": id, "result": result }),
        Ok(Err(failure)) => failure.to_message(&id),
        // A bug in the plugin is a 500, not a plugin that vanishes.
        Err(panic) => {
            let reason = panic
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown".to_string());
            Failure::status(
                500,
                format!("the plugin broke while running {action}: {reason}"),
            )
            .to_message(&id)
        }
    }
}

/// Writes one line and flushes: the host is waiting for it. False when the host is gone.
fn send<W: Write>(output: &Arc<Mutex<W>>, reply: &Value) -> bool {
    let mut output = lock(output);
    writeln!(output, "{reply}")
        .and_then(|()| output.flush())
        .is_ok()
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    /// A writer the test can read after the plugin has used it.
    #[derive(Clone, Default)]
    struct Shared(Arc<Mutex<Vec<u8>>>);

    impl Write for Shared {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            lock(&self.0).extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// A plugin with a few actions: `echo`, `fail`, `sleep` (watching for a cancel) and `panic`.
    fn plugin(action: &str, params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
        match action {
            "echo" => Ok(params.clone()),
            "where" => Ok(json!({ "workDir": cancel.work_dir().map(|dir| dir.to_string_lossy()) })),
            "fail" => Err(Failure::status(429, "busy").with_details(json!({ "after": 1 }))),
            "sleep" => {
                let end =
                    Instant::now() + Duration::from_millis(params["ms"].as_u64().unwrap_or(0));
                while Instant::now() < end {
                    if cancel.is_cancelled() {
                        return Err(Failure::new("cancelled"));
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Ok(json!({ "slept": true }))
            }
            "progress" => {
                cancel.progress("half way");
                Ok(json!({ "done": true }))
            }
            "panic" => panic!("on purpose"),
            other => Err(Failure::status(404, format!("no action {other:?}"))),
        }
    }

    fn talk(lines: &[&str]) -> Vec<Value> {
        let output = Shared::default();
        serve_lines(lines.join("\n").as_bytes(), output.clone(), plugin);
        let text = String::from_utf8(lock(&output.0).clone()).unwrap();
        text.lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn with_id(replies: &[Value], id: u64) -> &Value {
        replies
            .iter()
            .find(|reply| reply["id"] == id)
            .unwrap_or_else(|| panic!("no reply for call {id}"))
    }

    #[test]
    fn answers_hello_with_ready_and_a_call_with_its_result_under_the_same_id() {
        let replies = talk(&[
            r#"{"type":"hello","protocol":2,"capabilities":{}}"#,
            r#"{"type":"call","id":7,"method":"echo","params":{"a":1}}"#,
        ]);
        assert_eq!(replies[0]["type"], "ready");
        assert_eq!(replies[0]["protocol"], 2);
        assert!(replies[0]["capabilities"]["wants"].is_object());
        assert!(replies[0]["capabilities"]["progress"].is_object());
        assert_eq!(
            replies[1],
            json!({ "type": "result", "id": 7, "result": { "a": 1 } })
        );
    }

    #[test]
    fn a_failure_is_an_error_with_its_code_and_details_and_the_plugin_goes_on() {
        let replies = talk(&[
            r#"{"type":"call","id":1,"method":"fail","params":{}}"#,
            r#"{"type":"call","id":2,"method":"echo","params":1}"#,
            r#"{"type":"call","id":3,"method":"nope","params":{}}"#,
        ]);
        assert_eq!(
            with_id(&replies, 1),
            &json!({ "type": "error", "id": 1, "message": "busy", "code": 429, "details": { "after": 1 } })
        );
        assert_eq!(with_id(&replies, 2)["type"], "result");
        assert_eq!(with_id(&replies, 3)["code"], 404);
    }

    #[test]
    fn the_work_folder_of_hello_reaches_the_calls() {
        let replies = talk(&[
            r#"{"type":"hello","protocol":2,"workDir":"/w/p"}"#,
            r#"{"type":"call","id":1,"method":"where","params":{}}"#,
        ]);
        assert_eq!(with_id(&replies, 1)["result"]["workDir"], "/w/p");
        let none = talk(&[r#"{"type":"call","id":1,"method":"where","params":{}}"#]);
        assert_eq!(
            with_id(&none, 1)["result"]["workDir"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn an_action_can_say_it_is_alive_and_the_message_carries_the_call_id() {
        let replies = talk(&[r#"{"type":"call","id":4,"method":"progress","params":{}}"#]);
        let progress = replies
            .iter()
            .find(|reply| reply["type"] == "progress")
            .expect("a progress message");
        assert_eq!(progress["id"], 4);
        assert_eq!(progress["message"], "half way");
        assert!(replies
            .iter()
            .any(|reply| reply["type"] == "result" && reply["id"] == 4));
    }

    #[test]
    fn a_panic_is_a_500_that_says_what_broke_and_the_plugin_goes_on() {
        let replies = talk(&[
            r#"{"type":"call","id":1,"method":"panic","params":{}}"#,
            r#"{"type":"call","id":2,"method":"echo","params":{}}"#,
        ]);
        let broken = with_id(&replies, 1);
        assert_eq!(broken["code"], 500);
        assert!(
            broken["message"]
                .as_str()
                .unwrap()
                .contains("while running panic: on purpose"),
            "{broken}"
        );
        assert_eq!(with_id(&replies, 2)["type"], "result");
    }

    #[test]
    fn calls_run_at_the_same_time_and_a_cancel_stops_only_its_call() {
        let started = Instant::now();
        let replies = talk(&[
            r#"{"type":"call","id":1,"method":"sleep","params":{"ms":300}}"#,
            r#"{"type":"call","id":2,"method":"sleep","params":{"ms":300}}"#,
            r#"{"type":"call","id":3,"method":"sleep","params":{"ms":30000}}"#,
            r#"{"type":"cancel","id":3}"#,
        ]);
        assert!(started.elapsed() < Duration::from_secs(5));
        assert_eq!(with_id(&replies, 1)["type"], "result");
        assert_eq!(with_id(&replies, 2)["type"], "result");
        assert_eq!(with_id(&replies, 3)["message"], "cancelled");
    }

    #[test]
    fn stops_at_shutdown_and_skips_what_it_does_not_understand() {
        let replies = talk(&[
            "not json",
            r#"{"type":"mystery"}"#,
            r#"{"type":"shutdown"}"#,
            r#"{"type":"hello","protocol":2}"#,
        ]);
        assert!(replies.is_empty());
    }
}
