---
agent: agent
---

# Fix Risky Signature Defaults

Eliminate default values in function/method signatures that hide dependencies, mask missing inputs, or change semantics.

## Objective

- Explicit contracts: core logic requires critical inputs.
- No hidden dependency creation: dependencies injected from composition roots.
- No mutable defaults.
- Tests updated to match the new contracts.

## Non-negotiables

- Do not instantiate objects in default arguments.
- Do not use mutable defaults (`[]`, `{}`).
- Core/service code must not silently pick defaults for critical knobs/IDs/timeouts.
- Boundary layers (CLI/config/wiring) may provide defaults explicitly.

## What to search for

- Dependency defaults: `client=...()`, `session=...()`, `now=datetime.now()`.
- Mutable defaults: `items=[]`, `cache={}`.
- “Optional-by-default” fallbacks: `x: T | None = None` then `if x is None: x = ...`.
- Critical parameter defaults: empty string / `0` / `None` for IDs, limits, risk knobs, timeouts.

## Refactor pattern

1. Inventory each risky signature and classify: dependency, mutable default, critical knob, or optional-by-default.
2. Apply one of these fixes:
   - Make the parameter required.
   - Inject the dependency from the composition root.
   - Move defaults into a config object created at the boundary.
   - Split into two functions: a strict core function and a boundary wrapper that supplies defaults.
3. Update all call sites (no legacy overloads).
4. Add/update tests:
   - missing required inputs fail fast (assert the exception and message).
   - no hidden dependency instantiation.

## Deliverables

- Findings list: file + symbol + risk + fix chosen.
- Refactor changes + updated call sites.
- Tests updated/added to cover missing-input paths.

## Local validation

- `uv run ruff format`
- `uv run ruff check --fix`
- `uv run ty check`
- `uv run pytest -q`
