---
agent: agent
---

# Fix Stringly-Typed IDs and Vocabularies

Replace raw strings used as identifiers or closed vocabularies with strong internal types. Parse at boundaries; use typed values internally; serialize back to primitives at boundaries.

## Objective

- Parse/validate inputs immediately at ingestion (wire/DB/env/CLI).
- Core logic uses strong types (no raw strings).
- Boundaries serialize to primitives (`.value` / `str(x)`), and hashes/signatures only see JSON primitives.
- Tests create values via the same parsers used in production.

## Type choices

- Use `StrEnum` for closed vocabularies.
- Use `NewType` for string IDs that must not be mixed.
- Use dataclasses or pydantic for structured payloads with `from_*` / `to_*` boundary helpers.

## Non-negotiables

- Parse once at the boundary; do not let raw strings leak into core logic.
- Do not keep parallel “str and typed” APIs; update all call sites.
- Avoid `cast()` for untrusted data.

## Refactor checklist

1. Inventory stringly usage:
   - repeated literals, `x == "..."`, dict-key vocabularies, and ID-shaped strings.
2. Introduce canonical types in a small dedicated module (one per concept).
3. Add strict parsers: `parse_<thing>(raw: str) -> <Type>`.
4. Move parsing to ingestion points (wire decode, DB read, CLI/env parsing).
5. Update internal models and logic to use the new types.
6. Fix serialization boundaries (DB write, wire encode, logging, signatures).
7. Update tests to construct values via the new parsers and assert on typed values.

## Minimal templates

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
    except ValueError as e:
        raise ValueError(f"unknown side: {value!r}") from e
```

## Local validation

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
