"""Reads the sound VideoCraw decoded for this plugin."""

import os
from dataclasses import dataclass
from typing import Any

import numpy as np

from ..serve import Failure

#: Speech models listen at this rate; the manifest asks the core for it.
SAMPLE_RATE = 16000


@dataclass(frozen=True)
class AudioRegion:
    """The samples a region holds and where its first one is in the source, in milliseconds."""

    samples: np.ndarray
    start_ms: int


def read_audio_region(value: Any) -> AudioRegion | None:
    """The samples a region describes, or `None` when `value` is not a region (a path or a file).

    A region is what VideoCraw hands over for an input the manifest lists under `wants`: raw
    little-endian 32-bit float samples in a file the core decoded once, described by `file`,
    `format`, `sampleRate`, `channels`, `sampleCount`, `startMs` and `offsetBytes`. The file is
    mapped read-only (the operating system shares its pages with every process that reads it) and
    checked against the description first. The result is mono at 16000 samples a second, as the
    models listen.
    """
    if not isinstance(value, dict) or value.get("kind") != "audio":
        return None
    if value.get("format") != "f32le":
        raise Failure.status(400, "the audio region is not in f32le, the only format read")
    file = value.get("file")
    if not isinstance(file, str) or not file:
        raise Failure.status(400, "the audio region has no file")
    numbers = {}
    for key in ("sampleRate", "channels", "sampleCount", "startMs", "offsetBytes"):
        number = value.get(key)
        if isinstance(number, bool) or not isinstance(number, int) or number < 0:
            raise Failure.status(400, f"the audio region has no usable {key}")
        numbers[key] = number
    if numbers["sampleRate"] != SAMPLE_RATE or numbers["channels"] not in (1, 2):
        raise Failure.status(
            400,
            f"this plugin listens to {SAMPLE_RATE} Hz in 1 or 2 channels, "
            f"not {numbers['sampleRate']} Hz in {numbers['channels']}",
        )
    channels, frames, offset = numbers["channels"], numbers["sampleCount"], numbers["offsetBytes"]
    start_ms = numbers["startMs"]
    try:
        length = os.path.getsize(file)
    except OSError as error:
        raise Failure.status(404, f"{file}: the file is not there") from error
    wanted_bytes = offset + frames * channels * 4
    if wanted_bytes > length:
        raise Failure.status(
            400, f"the audio region says {wanted_bytes} bytes, but {file} has {length}"
        )
    if frames == 0:
        return AudioRegion(np.zeros(0, dtype=np.float32), start_ms)
    samples = np.memmap(file, dtype="<f4", mode="r", offset=offset, shape=(frames, channels))
    if channels == 1:
        return AudioRegion(samples[:, 0], start_ms)
    return AudioRegion(np.asarray(samples.mean(axis=1), dtype=np.float32), start_ms)
