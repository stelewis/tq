---
agent: agent
---

# Fix Catch-All Error Handling

Eliminate error-handling patterns that hide failures by silently dropping or “best-effort” accepting unexpected inputs.

## Objective

- No silent loss of unexpected inputs in critical loops.
- Structured, typed errors with bounded context.
- Tests updated so failures are observable.

## Non-negotiables

- Do not use bare `except:` or `except BaseException:`.
- In async code: always re-raise `asyncio.CancelledError`.
- Unknown payload shapes are errors (validate at boundaries).
- No compatibility scaffolding: update all call sites/tests.

## What to search for

- Broad handling: `except Exception`, bare `except`, `return_exceptions=True` without inspection.
- Silent drop: `else: continue`, `pass` on unknown types, “ignore unknown message”.
- Masking defaults: `payload.get("k", "")` / `get(..., 0)` where the default is treated as valid.
- Loops: `while True` / `async for` with `try/except` that continues.

## Refactor pattern

1. Classify each location: critical loop (must surface) vs tooling (may be less strict).
2. Replace catch-all logic with explicit handling:
	 - Handle known cases via `match`/`if` branches.
	 - For unknown cases, raise a dedicated error (or produce a typed error object).
3. If the loop must keep running:
	 - Emit the structured error to the existing error surface (health/metrics/callback/event bus), then continue.
4. Bound context on errors:
	 - include a subsystem identifier, optional message kind, bounded ID list, and a raw sample capped at ~2KB.

## Deliverables

- Findings list: file + symbol + risk + fix.
- New/updated error types with a stable shape.
- Updated/added tests that assert unknown inputs surface as structured errors.

## Local validation

- `uv run ruff format`
- `uv run ruff check --fix`
- `uv run ty check`
- `uv run tq check`
- `uv run pytest -q`
