"""Listens to a video or a sound and writes down the sentences, with their times."""

import threading
from collections.abc import Callable
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any

import numpy as np

from ..serve import Failure
from .sentence_contract import Sentence, Word
from .speech_model_store import get_model
from .split_sentences_algorithm import split_sentences


@dataclass(frozen=True)
class TranscribeOptions:
    #: The model: `tiny`, `base`, `small`, `medium`, `large-v3`, with `.en` for English only.
    model: str = "tiny"
    #: The language as a code such as `en`; `None` to find it out from the first seconds.
    language: str | None = None
    #: Skip the stretches with no speech. It stops the model from inventing words in silence, but
    #: it also drops speech that sits over loud music, so it is off unless asked for.
    skip_silence: bool = False
    beam_size: int = 5
    #: Give each sentence its words, with their own times.
    words: bool = False


def transcribe_media(
    media: Path | np.ndarray,
    options: TranscribeOptions,
    cancel: threading.Event,
    load_model: Callable[[str], Any] = get_model,
    offset_ms: int = 0,
) -> dict[str, Any]:
    """Gives `{ language, languageProbability, durationMs, model, text, sentences }`.

    `sentences` is a list of `{ startMs, endMs, text, confidence }`, with `words` (the same, one per
    word) in each when `options.words` is set. A video or sound with no speech
    in it is a success with no sentences. `media` is a file to decode here, or samples (mono, 16000
    a second) the core already decoded. `load_model` is how a model is got by its name (the
    recognition itself can be swapped in tests). `offset_ms` is where `media` starts in the source,
    when it is a stretch the host cut out: the times of the sentences are in the time of the source.
    A call that is a `Call` says it is alive as the recognition goes.
    """
    progress = getattr(cancel, "progress", None)
    if progress is not None:
        progress("loading the model")
    try:
        model = load_model(options.model)
    except Exception as error:
        # Usually no network the first time, or a name that is not a model.
        raise Failure.status(
            503, f"the model {options.model!r} could not be loaded: {error}"
        ) from error

    if progress is not None:
        progress("listening")
    words: list[Word] = []
    try:
        segments, info = model.transcribe(
            media if isinstance(media, np.ndarray) else str(media),
            language=options.language,
            beam_size=options.beam_size,
            word_timestamps=True,
            vad_filter=options.skip_silence,
        )
        for segment in segments:  # the recognition runs as this is read
            if cancel.is_set():
                raise Failure("cancelled", code="CANCELLED")
            if progress is not None:
                progress(f"{len(words)} words so far")
            for word in segment.words or []:
                words.append(Word(word.start, word.end, word.word, word.probability))
    except Failure:
        raise
    except Exception as error:
        name = "the audio" if isinstance(media, np.ndarray) else media.name
        raise Failure.status(422, f"cannot read speech from {name}: {error}") from error

    sentences = [_in_source_time(sentence, offset_ms) for sentence in split_sentences(words)]
    return {
        "language": info.language,
        "languageProbability": round(float(info.language_probability), 3),
        "durationMs": round(float(info.duration) * 1000),
        "model": options.model,
        "text": " ".join(sentence.text for sentence in sentences),
        "sentences": [sentence.to_dict(options.words) for sentence in sentences],
    }


def _in_source_time(sentence: Sentence, offset_ms: int) -> Sentence:
    """The sentence and its words moved by `offset_ms`: from stretch time to source time."""
    return replace(
        sentence,
        start_ms=sentence.start_ms + offset_ms,
        end_ms=sentence.end_ms + offset_ms,
        words=tuple(
            replace(word, start_ms=word.start_ms + offset_ms, end_ms=word.end_ms + offset_ms)
            for word in sentence.words
        ),
    )
