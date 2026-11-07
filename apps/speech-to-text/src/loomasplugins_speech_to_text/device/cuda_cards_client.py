"""Asks CTranslate2, the library under faster-whisper, how many NVIDIA cards it can use."""

from importlib import import_module


def count_cuda_cards() -> int:
    """How many CUDA cards CTranslate2 can use here; 0 when none, or when the library cannot say.

    CTranslate2 runs on the processor and on NVIDIA cards (CUDA). It has no route for AMD or Intel
    cards, which is why a card that is granted can still be left unused.
    """
    try:
        count = import_module("ctranslate2").get_cuda_device_count()
    except Exception:  # no library, a driver that is missing or broken: there is no card to use
        return 0
    return int(count)
