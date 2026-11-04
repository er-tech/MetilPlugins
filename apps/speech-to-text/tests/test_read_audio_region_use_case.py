from pathlib import Path

import numpy as np
import pytest
from loomasplugins_speech_to_text.decoded_audio import read_audio_region
from loomasplugins_speech_to_text.serve import Failure


def region(file: Path, channels: int, frames: int, **changes: object) -> dict[str, object]:
    return {
        "kind": "audio",
        "file": str(file),
        "format": "f32le",
        "sampleRate": 16000,
        "channels": channels,
        "sampleCount": frames,
        "startMs": 0,
        "offsetBytes": 0,
    } | changes


def write(path: Path, samples: list[float]) -> Path:
    path.write_bytes(np.array(samples, dtype="<f4").tobytes())
    return path


def test_mono_samples_are_read_as_they_are(tmp_path: Path) -> None:
    file = write(tmp_path / "a.f32le", [0.5, -0.5, 0.25])
    samples = read_audio_region(region(file, 1, 3))
    assert samples is not None
    assert samples.samples.tolist() == [0.5, -0.5, 0.25]
    assert samples.start_ms == 0


def test_stereo_is_mixed_down_and_the_offset_skips_the_start(tmp_path: Path) -> None:
    file = write(tmp_path / "s.f32le", [9.0, 1.0, 0.0, 0.5, 0.5])
    mixed = read_audio_region(region(file, 2, 2, offsetBytes=4))
    assert mixed is not None
    assert mixed.samples.tolist() == [0.5, 0.5]


def test_a_path_or_a_stored_file_is_not_a_region() -> None:
    assert read_audio_region("a.wav") is None
    assert read_audio_region({"file": "a.wav"}) is None
    assert read_audio_region(None) is None


@pytest.mark.parametrize(
    "changes, code",
    [
        ({"format": "s16le"}, 400),
        ({"sampleRate": 44100}, 400),
        ({"channels": 6}, 400),
        ({"sampleCount": 99}, 400),
        ({"sampleCount": True}, 400),
        ({"file": "no-such-file.f32le"}, 404),
    ],
)
def test_a_description_that_does_not_fit_is_refused(
    tmp_path: Path, changes: dict[str, object], code: int
) -> None:
    file = write(tmp_path / "a.f32le", [0.0, 0.0])
    with pytest.raises(Failure) as refused:
        read_audio_region(region(file, 1, 2) | changes)
    assert refused.value.code == code
