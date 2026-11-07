"""Chooses where a speech model runs from what the call asked and what Loomas grants."""

import re
from collections.abc import Callable

from ..serve import Failure
from .cuda_cards_client import count_cuda_cards
from .placement_contract import CPU, Placement

_DEVICE = re.compile(r"^(auto|cpu|gpu)(:(\d+))?$", re.IGNORECASE)

NO_CARD_ROUTE = (
    "the speech library (CTranslate2) runs on NVIDIA cards only, and no NVIDIA card was found; "
    "the processor is used"
)


def read_device(value: object) -> str:
    """The `device` param: `auto`, `cpu`, `gpu` or `gpu:N`, in any case. A 400 for anything else."""
    if value is None:
        return "auto"
    if isinstance(value, str):
        match = _DEVICE.match(value.strip())
        if match and (match.group(3) is None or match.group(1).lower() == "gpu"):
            return value.strip().lower()
    raise Failure.status(400, f"device must be auto, cpu, gpu or gpu:N, not {value!r}")


def resolve_device(asked: str, granted: str | None) -> str:
    """What to run on when the call asked for `asked` and Loomas grants `granted`: a call that left
    it to `auto` takes the grant, and a call that named a device keeps it. A grant that is not a
    device word is ignored, and with no grant `auto` is the processor."""
    if asked != "auto":
        return asked
    if granted is not None and _DEVICE.match(granted.strip()):
        return granted.strip().lower()
    return "cpu"


def choose_placement(
    asked: str,
    granted: str | None,
    count_cards: Callable[[], int] = count_cuda_cards,
) -> Placement:
    """Where the model should be loaded.

    The processor when that is what was asked or granted. For a card: the NVIDIA card with the
    number asked (`gpu:N`, else the first), on 16-bit floats. When the library has no card to use,
    the processor with the reason in `fallback`: the call does not fail.
    """
    device = resolve_device(asked, granted)
    if device == "cpu":
        return CPU
    cards = count_cards()
    if cards == 0:
        return CPU.on_cpu(NO_CARD_ROUTE)
    number = int(device.split(":", 1)[1]) if ":" in device else 0
    if number >= cards:
        return CPU.on_cpu(
            f"card {number} was asked for and only {cards} can be used; the processor is used"
        )
    return Placement(device="cuda", index=number, compute_type="float16")
