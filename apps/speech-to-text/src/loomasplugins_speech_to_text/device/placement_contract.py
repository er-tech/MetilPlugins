"""Where a speech model runs, and the report of what really ran."""

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class Placement:
    """The device a speech model is loaded on.

    `device` is what CTranslate2 takes (`cpu` or `cuda`), `index` the card number for `cuda`.
    `fallback` says why a card was not used when one was asked for or granted.
    """

    device: str = "cpu"
    index: int = 0
    #: 8-bit numbers on the processor, 16-bit floats on a card.
    compute_type: str = "int8"
    #: The name of the card, when one is used and its name is known.
    name: str | None = None
    fallback: str | None = None

    @property
    def uses_card(self) -> bool:
        return self.device == "cuda"

    def report(self) -> dict[str, Any]:
        """The `device` field of a result: `{ device, name, fallback }`, like the other plugins."""
        return {
            "device": f"cuda:{self.index}" if self.uses_card else "cpu",
            "name": self.name,
            "fallback": self.fallback,
        }

    def on_cpu(self, reason: str) -> "Placement":
        """The same request, run on the processor because of `reason`."""
        return Placement(fallback=reason)


#: The processor, with nothing to explain.
CPU = Placement()
