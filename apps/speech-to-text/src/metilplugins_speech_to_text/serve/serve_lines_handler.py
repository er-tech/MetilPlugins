"""Serves the plugin protocol: reads a JSON message per line, answers a JSON message per line."""

import io
import json
import sys
import threading
from collections.abc import Callable, Iterable
from typing import Any, BinaryIO

from .failure_error import Failure

PROTOCOL = 2

#: What this SDK can do for a plugin: read the sound and the pictures the host decodes, and say a
#: call is alive.
CAPABILITIES: dict[str, Any] = {"wants": {"media": ["audio", "video"]}, "progress": {}}


class Call(threading.Event):
    """What an action holds of its call: an event that is set when the host cancels it, and a way
    to say the call is alive (`progress`). A plugin whose manifest sets `stallSeconds` must do this
    more often than that, or the host stops it."""

    def __init__(self, notify: Callable[[str], None] | None = None) -> None:
        super().__init__()
        self._notify = notify

    def progress(self, message: str = "") -> None:
        if self._notify is not None:
            self._notify(message)


#: What a plugin is: a function of the action called, its params and an event that is set when
#: the host cancels the call. It returns the result, or raises `Failure` to fail.
Handler = Callable[[str, dict[str, Any], threading.Event], dict[str, Any]]


def serve_stdio(handler: Handler) -> None:
    """Serves the protocol on standard input and output until `shutdown` or the end of the input.

    Anything else that writes to standard output (a library printing a progress bar) is sent to
    standard error instead, so it cannot break the protocol.
    """
    out: BinaryIO = sys.stdout.buffer
    sys.stdout = sys.stderr
    lines = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8")

    write_lock = threading.Lock()

    def write(line: str) -> None:
        with write_lock:
            out.write((line + "\n").encode("utf-8"))
            out.flush()

    serve_lines(lines, write, handler)


def serve_lines(lines: Iterable[str], write: Callable[[str], None], handler: Handler) -> None:
    """Reads one JSON message per line from `lines` and answers with `write`, one call per thread.

    Lines that are not JSON, and messages of a type it does not know, are skipped. At the end of the
    input the calls still running are waited for; at `shutdown` they are abandoned.
    """
    cancels: dict[int, threading.Event] = {}
    cancels_lock = threading.Lock()
    calls: list[threading.Thread] = []

    for line in lines:
        message = _parse(line)
        if message is None:
            continue
        kind = message.get("type")
        if kind == "hello":
            write(_dump({"type": "ready", "protocol": PROTOCOL, "capabilities": CAPABILITIES}))
        elif kind == "call":
            call_id = message.get("id")
            cancel = Call(_progress_sender(call_id, write))
            if isinstance(call_id, int):
                with cancels_lock:
                    cancels[call_id] = cancel
            thread = threading.Thread(
                target=_run_call,
                args=(message, cancel, handler, write, cancels, cancels_lock),
                daemon=True,
            )
            thread.start()
            calls.append(thread)
        elif kind == "cancel":
            target = message.get("id")
            with cancels_lock:
                event = cancels.get(target) if isinstance(target, int) else None
            if event is not None:
                event.set()
        elif kind == "shutdown":
            return

    for thread in calls:
        thread.join()


def _progress_sender(call_id: Any, write: Callable[[str], None]) -> Callable[[str], None]:
    def send(note: str) -> None:
        write(_dump({"type": "progress", "id": call_id, "message": note}))

    return send


def _run_call(
    message: dict[str, Any],
    cancel: threading.Event,
    handler: Handler,
    write: Callable[[str], None],
    cancels: dict[int, threading.Event],
    cancels_lock: threading.Lock,
) -> None:
    call_id = message.get("id")
    action = str(message.get("method", ""))
    params = message.get("params")
    try:
        result = handler(action, params if isinstance(params, dict) else {}, cancel)
        reply: dict[str, Any] = {"type": "result", "id": call_id, "result": result}
    except Failure as failure:
        reply = failure.to_message(call_id)
    except Exception as error:  # a bug in the plugin is a 500, not a plugin that vanishes
        reply = Failure.status(500, f"the plugin broke while running {action}: {error}").to_message(
            call_id
        )
    with cancels_lock:
        if isinstance(call_id, int):
            cancels.pop(call_id, None)
    write(_dump(reply))


def _parse(line: str) -> dict[str, Any] | None:
    try:
        message = json.loads(line)
    except ValueError:
        return None
    return message if isinstance(message, dict) else None


def _dump(message: dict[str, Any]) -> str:
    return json.dumps(message, ensure_ascii=False, separators=(",", ":"))
