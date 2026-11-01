"""Listens to real speech with a real model. Skipped unless `METILPLUGINS_SPEECH_SAMPLE` names a
video or sound with English speech in it: the model (about 75 MB for `tiny`) is downloaded on first
use, so this is not part of an ordinary run."""

import os
import threading
from pathlib import Path

import pytest
from metilplugins_speech_to_text.actions import run_action

SAMPLE = os.environ.get("METILPLUGINS_SPEECH_SAMPLE")


@pytest.mark.skipif(not SAMPLE or not Path(SAMPLE).is_file(), reason="no sample with speech given")
def test_a_real_recording_gives_sentences_in_order_with_times_inside_its_length() -> None:
    assert SAMPLE is not None
    result = run_action(
        "transcribe", {"media": SAMPLE, "model": "tiny", "language": "en"}, threading.Event()
    )
    sentences = result["sentences"]
    assert sentences, "the recording has speech, so there should be sentences"
    starts = [sentence["startMs"] for sentence in sentences]
    assert starts == sorted(starts)
    assert all(0 <= s["startMs"] < s["endMs"] <= result["durationMs"] + 1000 for s in sentences)
    assert result["text"].strip()
