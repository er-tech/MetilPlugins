"""Where a speech model runs: what the call asks, what Loomas grants, what the library has."""

import json
import threading
from typing import Any

import pytest
from loomasplugins_speech_to_text.actions.run_action_use_case import run_action
from loomasplugins_speech_to_text.device import (
    CPU,
    Placement,
    choose_placement,
    load_placed_model,
    read_device,
    resolve_device,
)
from loomasplugins_speech_to_text.serve import Call, Failure, serve_lines


def test_the_device_words_are_read_in_any_case_and_anything_else_is_a_400() -> None:
    assert read_device(None) == "auto"
    assert read_device(" GPU:1 ") == "gpu:1"
    for wrong in ("cuda", "cpu:1", "gpu:x", 3):
        with pytest.raises(Failure) as refused:
            read_device(wrong)
        assert refused.value.message.startswith("device must be")


def test_a_call_that_left_it_to_auto_takes_the_grant_and_a_named_device_wins() -> None:
    assert resolve_device("auto", "gpu:0") == "gpu:0"
    assert resolve_device("auto", None) == "cpu"
    assert resolve_device("auto", "nonsense") == "cpu"
    assert resolve_device("cpu", "gpu:0") == "cpu"
    assert resolve_device("gpu", "cpu") == "gpu"


def test_the_processor_is_used_with_nothing_to_explain_when_that_is_what_was_asked() -> None:
    assert choose_placement("cpu", "gpu", lambda: 2) == CPU
    assert choose_placement("auto", None, lambda: 2) == CPU


def test_a_card_is_used_when_the_library_has_one_and_the_number_asked_exists() -> None:
    first = choose_placement("auto", "gpu", lambda: 1)
    assert (first.device, first.index, first.compute_type) == ("cuda", 0, "float16")
    assert first.report() == {"device": "cuda:0", "name": None, "fallback": None}
    assert choose_placement("gpu:1", None, lambda: 2).index == 1


def test_no_card_for_the_library_is_the_processor_with_the_reason_and_not_a_failure() -> None:
    placed = choose_placement("auto", "gpu", lambda: 0)
    assert placed.device == "cpu"
    assert placed.report()["device"] == "cpu"
    assert "NVIDIA" in (placed.fallback or "")
    assert "card 3" in (choose_placement("gpu:3", None, lambda: 1).fallback or "")


def test_a_model_that_cannot_start_on_the_card_is_loaded_on_the_processor_and_says_why() -> None:
    tried: list[str] = []

    def load(name: str, placement: Placement) -> str:
        tried.append(placement.device)
        if placement.uses_card:
            raise OSError("cudnn is missing")
        return f"{name} on the processor"

    model, used = load_placed_model(load, "tiny", Placement(device="cuda", compute_type="float16"))
    assert (model, tried) == ("tiny on the processor", ["cuda", "cpu"])
    assert "cudnn is missing" in (used.fallback or "")
    assert used.report()["device"] == "cpu"


def test_a_model_that_cannot_load_on_the_processor_either_raises() -> None:
    def load(name: str, placement: Placement) -> str:
        raise OSError("no network")

    with pytest.raises(OSError):
        load_placed_model(load, "tiny", CPU)


def test_hello_grants_the_device_to_the_calls_and_the_plugin_offers_gpu() -> None:
    seen: list[Any] = []
    out: list[str] = []

    def handler(action: str, params: dict[str, Any], cancel: threading.Event) -> dict[str, Any]:
        seen.append(getattr(cancel, "granted_device", "missing"))
        return {}

    hello = {"type": "hello", "capabilities": {"gpu": {"device": "gpu:0", "devices": []}}}
    call = {"type": "call", "id": 1, "method": "transcribe", "params": {}}
    serve_lines([json.dumps(hello), json.dumps(call)], out.append, handler)
    ready = json.loads(out[0])
    assert ready["capabilities"]["gpu"] == {}
    assert seen == ["gpu:0"]


def test_a_hello_without_a_grant_leaves_the_device_unset() -> None:
    assert Call().granted_device is None


def test_an_action_reads_the_device_of_the_call_and_the_grant_of_loomas(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    chosen: list[Placement] = []

    def fake_transcribe(*args: Any, **kwargs: Any) -> dict[str, Any]:
        chosen.append(kwargs["placement"])
        return {}

    import loomasplugins_speech_to_text.actions.run_action_use_case as module

    monkeypatch.setattr(module, "transcribe_media", fake_transcribe)
    monkeypatch.setattr(module, "read_media", lambda params: ("x.mp3", 0))
    monkeypatch.setattr(
        module, "choose_placement", lambda asked, granted: Placement(fallback=f"{asked}/{granted}")
    )
    run_action("transcribe", {"media": "x.mp3", "device": "gpu:1"}, Call(granted_device="cpu"))
    run_action("transcribe", {"media": "x.mp3"}, Call(granted_device="gpu"))
    assert [p.fallback for p in chosen] == ["gpu:1/cpu", "auto/gpu"]
