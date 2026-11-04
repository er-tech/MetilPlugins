"""Runs an action of the plugin."""

import threading
from typing import Any

from ..language_detection import detect_language
from ..serve import Failure
from ..transcription import transcribe_media
from ..transcription.transcribe_media_use_case import TranscribeOptions
from .read_params_mapper import read_media, read_model, read_options


def run_action(action: str, params: dict[str, Any], cancel: threading.Event) -> dict[str, Any]:
    """Runs `transcribe`: the sentences said in a video or a sound, with their times.

    Params: `media` (a video or a sound), `model` (default `tiny`), `language` (default: found out),
    `skipSilence` (default false) and `beamSize` (default 5). Runs `detectLanguage`: which language
    is spoken, from `media` and `model` (default `tiny`).
    """
    if action == "transcribe":
        media, offset_ms = read_media(params)
        return transcribe_media(media, read_options(params), cancel, offset_ms=offset_ms)
    if action == "detectLanguage":
        media, _ = read_media(params)
        return detect_language(media, read_model(params, TranscribeOptions().model), cancel)
    raise Failure.status(
        404, f"this plugin has no action {action!r} (it has transcribe and detectLanguage)"
    )
