"""Listens to the first seconds of a video or a sound and says which language is spoken."""

import threading
from collections.abc import Callable
from importlib import import_module
from pathlib import Path
from typing import Any

import numpy as np

from ..device import CPU, Placement, load_placed_model
from ..serve import Failure
from ..transcription.speech_model_store import get_model

#: How many languages are listed as candidates besides the answer.
CANDIDATES = 5


def detect_language(
    media: Path | np.ndarray,
    model_name: str,
    cancel: threading.Event,
    load_model: Callable[[str, Placement], Any] = get_model,
    placement: Placement = CPU,
) -> dict[str, Any]:
    """Gives `{ language, languageProbability, candidates, model, device }`.

    `candidates` is the most likely languages, each `{ language, probability }`, best first.
    `media` is a file to decode here, or samples (mono, 16000 a second) the core already decoded.
    `load_model` is how a model is got by its name (the recognition can be swapped in tests). A
    sound with no speech still gets an answer, with a low probability; the caller decides how
    sure is sure enough. A call that is a `Call` says it is alive as the work goes.
    """
    progress = getattr(cancel, "progress", None)
    if progress is not None:
        progress("loading the model")
    try:
        model, placement = load_placed_model(load_model, model_name, placement)
    except Exception as error:
        raise Failure.status(
            503, f"the model {model_name!r} could not be loaded: {error}"
        ) from error
    if cancel.is_set():
        raise Failure("cancelled", code="CANCELLED")

    if progress is not None:
        progress("listening")
    try:
        samples = media if isinstance(media, np.ndarray) else _decode(media)
        language, probability, ranked = model.detect_language(audio=samples)
    except Failure:
        raise
    except Exception as error:
        name = "the audio" if isinstance(media, np.ndarray) else media.name
        raise Failure.status(422, f"cannot read speech from {name}: {error}") from error

    return {
        "language": language,
        "languageProbability": round(float(probability), 3),
        "candidates": [
            {"language": code, "probability": round(float(chance), 3)}
            for code, chance in sorted(ranked, key=lambda pair: pair[1], reverse=True)[:CANDIDATES]
        ],
        "model": model_name,
        "device": placement.report(),
    }


def _decode(path: Path) -> np.ndarray:
    """The samples of a file, mono and 16000 a second, as the recognition wants them."""
    decode_audio = import_module("faster_whisper").decode_audio
    samples: np.ndarray = decode_audio(str(path), sampling_rate=16000)
    return samples
