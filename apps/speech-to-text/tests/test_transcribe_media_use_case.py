import threading
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pytest
from loomasplugins_speech_to_text.actions import run_action
from loomasplugins_speech_to_text.serve import Failure
from loomasplugins_speech_to_text.transcription import transcribe_media
from loomasplugins_speech_to_text.transcription.transcribe_media_use_case import TranscribeOptions


@dataclass
class FakeWord:
    start: float
    end: float
    word: str
    probability: float = 0.9


@dataclass
class FakeSegment:
    words: list[FakeWord] | None


@dataclass
class FakeInfo:
    language: str = "en"
    language_probability: float = 0.987654
    duration: float = 6.5


@dataclass
class FakeModel:
    """Stands in for the speech recognition: gives the segments it was made with."""

    segments: list[FakeSegment]
    calls: list[tuple[str, dict[str, Any]]] = field(default_factory=list)

    def transcribe(self, path: str, **options: Any) -> tuple[Any, FakeInfo]:
        self.calls.append((path, options))
        return iter(self.segments), FakeInfo()


SPEECH = [
    FakeSegment(
        [FakeWord(0.2, 0.6, " Hello"), FakeWord(0.6, 1.1, " world.")],
    ),
    FakeSegment(
        [FakeWord(2.0, 2.4, " Second"), FakeWord(2.4, 3.0, " one!")],
    ),
]


def test_a_stretch_cut_out_of_the_source_gives_times_in_the_source() -> None:
    model = FakeModel(SPEECH)
    result = transcribe_media(
        Path("talk.mp4"),
        TranscribeOptions(),
        threading.Event(),
        lambda _name, _placement: model,
        offset_ms=60_000,
    )
    assert result["sentences"][0]["startMs"] == 60_200
    assert result["sentences"][1]["endMs"] == 63_000


def test_gives_the_language_the_length_and_the_sentences_with_their_times() -> None:
    model = FakeModel(SPEECH)
    result = transcribe_media(
        Path("talk.mp4"),
        TranscribeOptions(model="base"),
        threading.Event(),
        lambda _name, _placement: model,
    )
    assert result["language"] == "en"
    assert result["languageProbability"] == 0.988
    assert result["durationMs"] == 6500
    assert result["model"] == "base"
    assert result["text"] == "Hello world. Second one!"
    assert [(s["startMs"], s["endMs"], s["text"]) for s in result["sentences"]] == [
        (200, 1100, "Hello world."),
        (2000, 3000, "Second one!"),
    ]


def test_passes_the_options_to_the_recognition_and_asks_for_the_time_of_each_word() -> None:
    model = FakeModel(SPEECH)
    options = TranscribeOptions(language="pt", skip_silence=False, beam_size=2)
    transcribe_media(Path("talk.mp4"), options, threading.Event(), lambda _name, _placement: model)
    ((path, sent),) = model.calls
    assert path == "talk.mp4"
    assert sent == {
        "language": "pt",
        "beam_size": 2,
        "word_timestamps": True,
        "vad_filter": False,
    }


def test_no_speech_is_a_success_with_no_sentences() -> None:
    model = FakeModel([FakeSegment(None), FakeSegment([])])
    result = transcribe_media(
        Path("x.mp3"), TranscribeOptions(), threading.Event(), lambda _n, _placement: model
    )
    assert result["sentences"] == [] and result["text"] == ""


def test_a_model_that_cannot_be_loaded_is_a_503_and_media_that_cannot_be_read_is_a_422() -> None:
    def no_model(_name: str, _placement: object) -> Any:
        raise OSError("no network")

    with pytest.raises(Failure) as unavailable:
        transcribe_media(Path("x.mp3"), TranscribeOptions(model="big"), threading.Event(), no_model)
    assert unavailable.value.code == 503
    assert "'big' could not be loaded: no network" in unavailable.value.message

    class Broken:
        def transcribe(self, _path: str, **_options: Any) -> Any:
            raise RuntimeError("invalid data")

    with pytest.raises(Failure) as unreadable:
        transcribe_media(
            Path("x.mp3"), TranscribeOptions(), threading.Event(), lambda _n, _placement: Broken()
        )
    assert unreadable.value.code == 422
    assert "cannot read speech from x.mp3: invalid data" in unreadable.value.message


def test_a_cancel_stops_the_work_between_segments() -> None:
    cancel = threading.Event()

    class Cancelling:
        def transcribe(self, _path: str, **_options: Any) -> Any:
            def segments() -> Any:
                yield SPEECH[0]
                cancel.set()
                yield SPEECH[1]

            return segments(), FakeInfo()

    with pytest.raises(Failure) as stopped:
        transcribe_media(
            Path("x.mp3"), TranscribeOptions(), cancel, lambda _n, _placement: Cancelling()
        )
    assert stopped.value.message == "cancelled" and stopped.value.code == "CANCELLED"


def test_the_action_reads_a_stored_file_a_path_and_fails_with_the_status_that_fits(
    tmp_path: Path,
) -> None:
    sound = tmp_path / "talk.mp3"
    sound.write_bytes(b"not really a sound")
    cancel = threading.Event()

    def code_of(action: str, params: dict[str, Any]) -> Any:
        with pytest.raises(Failure) as failure:
            run_action(action, params, cancel)
        return failure.value.code

    assert code_of("transcribe", {}) == 400
    assert code_of("transcribe", {"media": 5}) == 400
    assert code_of("transcribe", {"media": {"name": "no file"}}) == 400
    assert code_of("transcribe", {"media": str(tmp_path / "missing.mp3")}) == 404
    assert code_of("transcribe", {"media": {"file": str(tmp_path / "missing.mp3")}}) == 404
    bad_options: list[dict[str, Any]] = [
        {"model": 3},
        {"model": "bad name; rm -rf"},
        {"language": "English"},
        {"skipSilence": "yes"},
        {"words": "yes"},
        {"beamSize": 0},
        {"beamSize": True},
    ]
    for bad in bad_options:
        assert code_of("transcribe", {"media": str(sound), **bad}) == 400, bad
    assert code_of("sing", {"media": str(sound)}) == 404


def test_words_are_given_only_when_asked_for_and_are_in_the_time_of_the_source() -> None:
    model = FakeModel(SPEECH)
    plain = transcribe_media(
        Path("talk.mp4"), TranscribeOptions(), threading.Event(), lambda _name, _placement: model
    )
    assert all("words" not in sentence for sentence in plain["sentences"])

    with_words = transcribe_media(
        Path("talk.mp4"),
        TranscribeOptions(words=True),
        threading.Event(),
        lambda _name, _placement: FakeModel(SPEECH),
        offset_ms=60_000,
    )
    first = with_words["sentences"][0]
    assert first["words"] == [
        {"startMs": 60_200, "endMs": 60_600, "text": "Hello", "confidence": 0.9},
        {"startMs": 60_600, "endMs": 61_100, "text": "world.", "confidence": 0.9},
    ]
    # The sentence spans its words.
    assert first["startMs"] == first["words"][0]["startMs"]
    assert first["endMs"] == first["words"][-1]["endMs"]
