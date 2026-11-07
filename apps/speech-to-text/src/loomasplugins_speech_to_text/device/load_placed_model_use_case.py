"""Loads a speech model on the chosen device, and falls back to the processor when it cannot."""

from collections.abc import Callable
from typing import Any

from .placement_contract import Placement


def load_placed_model(
    load_model: Callable[[str, Placement], Any], name: str, placement: Placement
) -> tuple[Any, Placement]:
    """The model and the placement it really got.

    A model that cannot start on a card (a driver or a CUDA library that is missing, not enough
    memory) is loaded on the processor, and the placement that comes back says why. A model that
    cannot load on the processor either raises the error of that load.
    """
    if not placement.uses_card:
        return load_model(name, placement), placement
    try:
        return load_model(name, placement), placement
    except Exception as error:  # whatever the card stack says: the processor still works
        fallback = placement.on_cpu(f"the card could not be used ({error}); the processor is used")
        return load_model(name, fallback), fallback
