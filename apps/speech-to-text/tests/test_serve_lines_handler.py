import json
import threading
import time
from typing import Any

from loomasplugins_speech_to_text.serve import Call, Failure, serve_lines


def plugin(action: str, params: dict[str, Any], cancel: threading.Event) -> dict[str, Any]:
    """A plugin with a few actions: echo, fail, bug, sleep (watching for a cancel)."""
    if action == "echo":
        return params
    if action == "fail":
        raise Failure("busy", code=429, details={"after": 1})
    if action == "ping":
        assert isinstance(cancel, Call)
        cancel.progress("half way")
        return {"pinged": True}
    if action == "bug":
        raise ValueError("on purpose")
    if action == "sleep":
        end = time.monotonic() + params["ms"] / 1000
        while time.monotonic() < end:
            if cancel.wait(0.005):
                raise Failure("cancelled")
        return {"slept": True}
    raise Failure.status(404, f"no action {action!r}")


def talk(*messages: str) -> list[dict[str, Any]]:
    written: list[str] = []
    lock = threading.Lock()

    def write(line: str) -> None:
        with lock:
            written.append(line)

    serve_lines([message + "\n" for message in messages], write, plugin)
    return [json.loads(line) for line in written]


def with_id(replies: list[dict[str, Any]], call_id: int) -> dict[str, Any]:
    return next(reply for reply in replies if reply.get("id") == call_id)


def test_an_action_can_say_it_is_alive_and_the_message_carries_the_call_id() -> None:
    replies = talk('{"type":"call","id":4,"method":"ping","params":{}}')
    progress = next(reply for reply in replies if reply["type"] == "progress")
    assert progress["id"] == 4
    assert progress["message"] == "half way"
    assert any(reply["type"] == "result" and reply["id"] == 4 for reply in replies)


def test_answers_hello_with_ready_and_a_call_with_its_result_under_the_same_id() -> None:
    replies = talk(
        '{"type":"hello","protocol":2,"capabilities":{}}',
        '{"type":"call","id":7,"method":"echo","params":{"a":"Olá"}}',
    )
    assert replies[0]["type"] == "ready"
    assert replies[0]["protocol"] == 2
    assert replies[0]["capabilities"]["progress"] == {}
    assert "wants" in replies[0]["capabilities"]
    assert replies[1] == {"type": "result", "id": 7, "result": {"a": "Olá"}}


def test_a_failure_is_an_error_with_its_code_and_details_and_the_plugin_goes_on() -> None:
    replies = talk(
        '{"type":"call","id":1,"method":"fail","params":{}}',
        '{"type":"call","id":2,"method":"echo","params":{}}',
        '{"type":"call","id":3,"method":"nope","params":{}}',
    )
    assert with_id(replies, 1) == {
        "type": "error",
        "id": 1,
        "message": "busy",
        "code": 429,
        "details": {"after": 1},
    }
    assert with_id(replies, 2)["type"] == "result"
    assert with_id(replies, 3)["code"] == 404


def test_a_bug_in_the_plugin_is_a_500_that_says_what_broke_and_the_plugin_goes_on() -> None:
    replies = talk(
        '{"type":"call","id":1,"method":"bug","params":{}}',
        '{"type":"call","id":2,"method":"echo","params":{}}',
    )
    broken = with_id(replies, 1)
    assert broken["code"] == 500
    assert "while running bug: on purpose" in broken["message"]
    assert with_id(replies, 2)["type"] == "result"


def test_calls_run_at_the_same_time_and_a_cancel_stops_only_its_call() -> None:
    started = time.monotonic()
    replies = talk(
        '{"type":"call","id":1,"method":"sleep","params":{"ms":300}}',
        '{"type":"call","id":2,"method":"sleep","params":{"ms":300}}',
        '{"type":"call","id":3,"method":"sleep","params":{"ms":30000}}',
        '{"type":"cancel","id":3}',
    )
    assert time.monotonic() - started < 5
    assert with_id(replies, 1)["type"] == "result"
    assert with_id(replies, 2)["type"] == "result"
    assert with_id(replies, 3)["message"] == "cancelled"


def test_stops_at_shutdown_and_skips_what_it_does_not_understand() -> None:
    replies = talk(
        "not json",
        "[1, 2]",
        '{"type":"mystery"}',
        '{"type":"shutdown"}',
        '{"type":"hello","protocol":2,"capabilities":{}}',
    )
    assert replies == []


def test_a_failure_sends_only_what_it_was_given() -> None:
    assert Failure("no").to_message(3) == {"type": "error", "id": 3, "message": "no"}
    full = Failure("x", code="QUOTA", retryable=False, details=[1]).to_message(1)
    assert full == {
        "type": "error",
        "id": 1,
        "message": "x",
        "code": "QUOTA",
        "retryable": False,
        "details": [1],
    }
