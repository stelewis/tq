---
agent: agent
---

# Fix Heavyweight `__init__.py` Imports

In Python, importing `pkg.submodule` executes `pkg/__init__.py` first. Keep `__init__.py` cheap and never use it as an API barrel.

## Objective

- Minimal `__init__.py` files with no heavyweight imports.
- No re-export hubs; import exact module paths everywhere.
- Fewer circular imports and clearer dependency direction.

## Non-negotiables

- `__init__.py` must not import wiring/services/clients/CLI.
- Avoid `__all__`-style barrels and re-exports.
- No compatibility scaffolding: update all imports and tests.

## What to search for

- In `__init__.py`: `from .x import ...`, `import ...`, `__all__ = [...]`.
- Imports that look “package-level”: `from pkg import thing` where `thing` actually lives in a submodule.
- Side-effect imports (env/logging setup, network clients, global singletons).

## Refactor pattern

1. Inventory each `__init__.py` and classify imports as lightweight vs heavyweight.
2. Make `__init__.py` minimal:
   - keep only docstring, `from __future__ import annotations`, tiny constants/types.
3. Replace all package-level imports with direct module imports.
4. If a stable facade is truly needed, create an explicit module (e.g. `api.py`) and import that directly.
5. Run tests to ensure import-time cycles are gone.

## Deliverables

- Findings list: each heavyweight `__init__` and what it imported.
- Refactor: minimal `__init__.py` plus updated call sites/tests.

## Local validation

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
