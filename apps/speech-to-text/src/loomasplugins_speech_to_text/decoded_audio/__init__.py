"""Sound the core decoded once and hands over as a region of samples in a file."""

from .read_audio_region_use_case import SAMPLE_RATE, AudioRegion, read_audio_region

__all__ = ["SAMPLE_RATE", "AudioRegion", "read_audio_region"]
