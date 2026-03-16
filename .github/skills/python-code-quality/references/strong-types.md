# Strong Internal Types

Replace raw strings used as identifiers or closed vocabularies with strong internal types. Parse at boundaries, use typed values internally, then serialize back to primitives at boundaries.

## Objective

- Parse and validate inputs immediately at ingestion from wire, database, environment, or CLI boundaries.
- Keep core logic on strong internal types instead of raw strings.
- Serialize only at boundaries, and ensure hashes or signatures see JSON primitives rather than internal wrapper objects.
- Build test values through the same parsers used in production.

## Type Choices

- Use StrEnum for closed vocabularies.
- Use NewType for string IDs that must not be mixed.
- Use dataclasses or pydantic models for structured payloads with explicit `from_*` and `to_*` helpers.

## Non-Negotiables

- Parse once at the boundary; do not let raw strings leak into core logic.
- Do not keep parallel string and typed APIs; update all call sites.
- Avoid cast() for untrusted data.

## Refactor Checklist

1. Inventory repeated literals, equality checks on string vocabularies, dictionary-key vocabularies, and ID-shaped strings.
2. Introduce canonical types in a small dedicated module, typically one module per concept.
3. Add strict parsers such as parse_order_id(raw: str) -> OrderId.
4. Move parsing to ingestion points: wire decode, database read, CLI parsing, or environment parsing.
5. Update internal models and logic to use the new types.
6. Fix serialization boundaries for database writes, wire encoding, logging, and signatures.
7. Update tests to construct values via the new parsers and assert on typed behavior.

## Minimal Template

```py
from enum import StrEnum
from typing import NewType

OrderId = NewType("OrderId", str)


def parse_order_id(value: str) -> OrderId:
    value = value.strip()
    if not value:
        raise ValueError("order_id must be non-empty")
    return OrderId(value)


class Side(StrEnum):
    BUY = "buy"
    SELL = "sell"


def parse_side(value: str) -> Side:
    value = value.strip().lower()
    try:
        return Side(value)
    except ValueError as error:
        raise ValueError(f"unknown side: {value!r}") from error
```
