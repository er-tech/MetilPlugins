import threading
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import numpy as np
import pytest
from metilplugins_speech_to_text.actions import run_action
from metilplugins_speech_to_text.language_detection import detect_language
from metilplugins_speech_to_text.serve import Failure

RANKED = [("es", 0.021), ("pt", 0.9712), ("en", 0.004), ("fr", 0.003), ("de", 0.002), ("it", 0.001)]


@dataclass
class FakeModel:
    """Stands in for the speech model: says what it was made to say about the audio it is given."""

    calls: list[Any] = field(default_factory=list)

    def detect_language(self, audio: Any = None) -> tuple[str, float, list[tuple[str, float]]]:
        self.calls.append(audio)
        return "pt", 0.9712, RANKED


def test_the_language_has_its_certainty_and_the_five_most_likely_candidates() -> None:
    samples = np.zeros(16000, dtype=np.float32)
    model = FakeModel()
    answer = detect_language(samples, "tiny", threading.Event(), lambda name: model)
    assert answer["language"] == "pt"
    assert answer["languageProbability"] == 0.971
    assert [c["language"] for c in answer["candidates"]] == ["pt", "es", "en", "fr", "de"]
    assert answer["candidates"][0] == {"language": "pt", "probability": 0.971}
    assert answer["model"] == "tiny"
    assert model.calls[0] is samples  # the samples Metil decoded are used as they are


def test_a_model_that_cannot_be_loaded_is_a_503_and_unreadable_audio_a_422() -> None:
    def no_model(name: str) -> Any:
        raise OSError("no network")

    with pytest.raises(Failure) as failure:
        detect_language(np.zeros(8, dtype=np.float32), "tiny", threading.Event(), no_model)
    assert failure.value.code == 503

    class Broken(FakeModel):
        def detect_language(self, audio: Any = None) -> tuple[str, float, list[tuple[str, float]]]:
            raise ValueError("not audio")

    with pytest.raises(Failure) as failure:
        detect_language(
            np.zeros(8, dtype=np.float32), "tiny", threading.Event(), lambda name: Broken()
        )
    assert failure.value.code == 422


def test_a_cancelled_call_stops_before_listening() -> None:
    cancel = threading.Event()
    cancel.set()
    with pytest.raises(Failure) as failure:
        detect_language(np.zeros(8, dtype=np.float32), "tiny", cancel, lambda name: FakeModel())
    assert failure.value.code == "CANCELLED"


def test_the_action_checks_its_params(tmp_path: Path) -> None:
    sound = tmp_path / "a.mp3"
    sound.write_bytes(b"x")

    def code_of(params: dict[str, Any]) -> Any:
        with pytest.raises(Failure) as failure:
            run_action("detectLanguage", params, threading.Event())
        return failure.value.code

    assert code_of({}) == 400
    assert code_of({"media": str(tmp_path / "missing.mp3")}) == 404
    assert code_of({"media": str(sound), "model": "bad name; rm -rf"}) == 400
    assert code_of({"media": str(sound), "model": 3}) == 400
