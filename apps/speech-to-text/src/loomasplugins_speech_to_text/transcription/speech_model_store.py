"""The speech models that are loaded: a model is loaded once for each device, whoever uses it."""

import threading
from importlib import import_module
from typing import Any

from ..device import Placement

_models: dict[tuple[str, str, int], Any] = {}
_lock = threading.Lock()


def get_model(name: str, placement: Placement) -> Any:
    """The faster-whisper model with this name on this device, loaded on first use.

    A name like `tiny` or `base.en` is downloaded once from Hugging Face into its cache folder; a
    folder on disk also works. On the processor it uses 8-bit numbers, which is slower than a
    graphics card but needs nothing installed; on an NVIDIA card it uses 16-bit floats.
    """
    key = (name, placement.device, placement.index)
    with _lock:
        if key not in _models:
            # Heavy, so loaded on first use. Imported by name: stubs exist on some installs only.
            whisper_model = import_module("faster_whisper").WhisperModel
            _models[key] = whisper_model(
                name,
                device=placement.device,
                device_index=placement.index,
                compute_type=placement.compute_type,
            )
        return _models[key]
