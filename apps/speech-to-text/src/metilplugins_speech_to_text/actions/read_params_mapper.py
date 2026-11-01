"""Reads the params of the `transcribe` action, failing with a 400 that says what was wrong."""

import re
from pathlib import Path
from typing import Any

import numpy as np

from ..decoded_audio import read_audio_region
from ..serve import Failure
from ..transcription.transcribe_media_use_case import TranscribeOptions

_MODEL = re.compile(r"^[A-Za-z0-9_.\-/\\: ]{1,200}$")
_LANGUAGE = re.compile(r"^[a-z]{2,3}(-[A-Za-z]{2,4})?$")


def read_media(params: dict[str, Any]) -> tuple[Path | np.ndarray, int]:
    """What to listen to, and where it starts in the source in milliseconds: the samples VideoCraw
    decoded (a region, which may be a stretch of the source), or else the file `media` names, as a
    stored file or a video (an object with an absolute `file`) or a path in text, from 0. A file
    that is not there is a 404."""
    media = params.get("media")
    decoded = read_audio_region(media)
    if decoded is not None:
        return decoded.samples, decoded.start_ms
    if isinstance(media, str) and media:
        path = Path(media)
    elif isinstance(media, dict) and isinstance(media.get("file"), str):
        path = Path(media["file"])
    elif media is None:
        raise Failure.status(
            400, "media is missing: give a video or a sound (an object with file) or a path"
        )
    else:
        raise Failure.status(400, f"media must be a stored file or a path in text, not {media!r}")
    if not path.is_file():
        raise Failure.status(404, f"{path}: the file is not there")
    return path, 0


def read_options(params: dict[str, Any]) -> TranscribeOptions:
    """The options of the action; what is not given has the default of `TranscribeOptions`."""
    defaults = TranscribeOptions()

    model = params.get("model", defaults.model)
    if not isinstance(model, str) or not _MODEL.match(model):
        raise Failure.status(
            400, f"model must be the name of a model such as tiny or base.en, not {model!r}"
        )

    language = params.get("language", defaults.language)
    if language is not None and not (isinstance(language, str) and _LANGUAGE.match(language)):
        raise Failure.status(
            400, f"language must be a code such as en or pt-BR (or left out), not {language!r}"
        )

    skip_silence = params.get("skipSilence", defaults.skip_silence)
    if not isinstance(skip_silence, bool):
        raise Failure.status(400, f"skipSilence must be true or false, not {skip_silence!r}")

    beam_size = params.get("beamSize", defaults.beam_size)
    if isinstance(beam_size, bool) or not isinstance(beam_size, int) or not 1 <= beam_size <= 10:
        raise Failure.status(
            400, f"beamSize must be a whole number from 1 to 10, not {beam_size!r}"
        )

    words = params.get("words", defaults.words)
    if not isinstance(words, bool):
        raise Failure.status(400, f"words must be true or false, not {words!r}")

    return TranscribeOptions(
        model=model, language=language, skip_silence=skip_silence, beam_size=beam_size, words=words
    )
