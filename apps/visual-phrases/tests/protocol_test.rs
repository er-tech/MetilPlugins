//! The plugin as Metil starts it: the handshake, a call that is wrong (which must come back as
//! an error and not a result), an action it does not have, and the end. Looking at pictures needs
//! the model, which is downloaded on first use: that is tried by hand and by Metil's own tests.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn talk(messages: &[Value]) -> Vec<Value> {
    let mut plugin = Command::new(env!("CARGO_BIN_EXE_metilplugins-visual-phrases"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = plugin.stdin.take().unwrap();
    let mut output = BufReader::new(plugin.stdout.take().unwrap());
    let mut answers = Vec::new();
    for message in messages {
        writeln!(input, "{message}").unwrap();
        if message["type"] != "shutdown" {
            let mut line = String::new();
            output.read_line(&mut line).unwrap();
            answers.push(serde_json::from_str(&line).unwrap());
        }
    }
    drop(input);
    plugin.wait().unwrap();
    answers
}

#[test]
fn it_answers_hello_refuses_a_wrong_call_with_an_error_and_stops_on_shutdown() {
    let answers = talk(&[
        json!({ "type": "hello", "protocol": 2, "capabilities": {} }),
        json!({ "type": "call", "id": 1, "method": "find", "params": { "media": "a.mp4", "phrases": ["a cat"] } }),
        json!({ "type": "call", "id": 2, "method": "find",
                "params": { "media": "a.mp4", "phrases": [] } }),
        json!({ "type": "call", "id": 3, "method": "count", "params": {} }),
        json!({ "type": "shutdown" }),
    ]);
    assert_eq!(answers[0]["type"], "ready");
    assert_eq!(answers[0]["protocol"], 2);
    for (answer, code) in answers[1..].iter().zip([400, 400, 404]) {
        assert_eq!(answer["type"], "error", "{answer}");
        assert_eq!(answer["code"], code, "{answer}");
    }
    assert!(answers[2]["message"].as_str().unwrap().contains("phrases"));
}
