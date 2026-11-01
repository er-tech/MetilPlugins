"""Starts the plugin: `python -m videoplugins_speech_to_text`. VideoCraw does this itself."""

# Loaded here, on the main thread, before the first call: importing these native libraries from
# the thread that serves a call hung on Windows while the main thread waited on the pipe.
import faster_whisper  # noqa: F401  # type: ignore[import-untyped]

from .actions import run_action
from .serve import serve_stdio

serve_stdio(run_action)
