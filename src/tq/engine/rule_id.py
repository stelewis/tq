"""Rule identifier value object for stable diagnostic identity."""

from __future__ import annotations

import re
from dataclasses import dataclass

RULE_ID_PATTERN = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")


@dataclass(frozen=True, slots=True)
class RuleId:
    """Stable identifier for a rule.

    Rule identifiers use kebab-case.
    """

    value: str

    def __post_init__(self) -> None:
        """Validate identifier shape.

        Raises:
            ValueError: If the identifier is blank or not valid kebab-case.
        """
        if not self.value:
            msg = "RuleId must be non-empty"
            raise ValueError(msg)

        if not RULE_ID_PATTERN.fullmatch(self.value):
            msg = "RuleId must be kebab-case, e.g. mapping-missing-test"
            raise ValueError(msg)

    def __str__(self) -> str:
        """Return identifier string form."""
        return self.value
