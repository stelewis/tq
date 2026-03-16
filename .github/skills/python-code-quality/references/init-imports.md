# Heavyweight __init__.py Imports

In Python, importing pkg.submodule executes pkg/__init__.py first. Keep __init__.py cheap and never use it as an API barrel.

## Objective

- Keep __init__.py files minimal and free of heavyweight imports.
- Remove re-export hubs and import exact module paths instead.
- Reduce circular imports and clarify dependency direction.

## Non-Negotiables

- __init__.py must not import wiring, services, clients, or CLI modules.
- Avoid __all__ barrels and package-level re-exports.
- Do not preserve compatibility scaffolding; update all imports and tests.

## What To Search For

- from .x import ..., import ..., or __all__ declarations inside __init__.py.
- Package-level imports such as from pkg import thing where thing really lives in a submodule.
- Side-effect imports for environment setup, logging setup, network clients, or global singletons.

## Refactor Pattern

1. Inventory each __init__.py and classify its imports as lightweight or heavyweight.
2. Make __init__.py minimal. Keep only a docstring, future annotations, or tiny constants and type aliases when genuinely needed.
3. Replace package-level imports with direct module imports everywhere.
4. If a stable facade is truly necessary, create an explicit module such as api.py and import that module directly.
5. Run tests to confirm import-time cycles and side effects are gone.
