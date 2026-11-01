"""The speech recognition models that are loaded: a model is loaded once, whoever uses it."""

import threading
from importlib import import_module
from typing import Any

_models: dict[str, Any] = {}
_lock = threading.Lock()


def get_model(name: str) -> Any:
    """The faster-whisper model with this name, loaded on first use.

    A name like `tiny` or `base.en` is downloaded once from Hugging Face into its cache folder; a
    folder on disk also works. It runs on the processor with 8-bit numbers, which is slower than a
    graphics card but needs nothing installed.
    """
    with _lock:
        if name not in _models:
            # Heavy, so loaded on first use; imported by name because the package has stubs on some installs only.
            whisper_model = import_module("faster_whisper").WhisperModel
            _models[name] = whisper_model(name, device="cpu", compute_type="int8")
        return _models[name]
