"""Where the speech models run: the processor, or an NVIDIA card when Loomas grants one."""

from .choose_placement_algorithm import choose_placement, read_device, resolve_device
from .load_placed_model_use_case import load_placed_model
from .placement_contract import CPU, Placement

__all__ = [
    "CPU",
    "Placement",
    "choose_placement",
    "load_placed_model",
    "read_device",
    "resolve_device",
]
