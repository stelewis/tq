---
name: python-code-quality
description: Improves Python code quality by fixing architectural refactor smells such as stringly typed IDs and vocabularies, risky signature defaults, heavyweight __init__.py imports, and catch-all error handling. Use whenever a Python task involves hardening modules, refactoring internal APIs, cleaning up import-time side effects, removing hidden defaults, surfacing swallowed failures, or replacing weak internal typing.
argument-hint: describe the Python code smell, affected files, and the invariant you want after the refactor
user-invocable: true
disable-model-invocation: false
---

# Python Code Quality

Use this skill for Python refactors that need architectural cleanup rather than cosmetic lint fixes.

## When To Use It

- The task is to harden Python code, refactor internals, or improve module boundaries.
- The code smell involves raw string IDs or vocabularies, risky defaults, heavyweight package imports, or broad exception handling.
- The change needs coordinated updates across source, call sites, and tests.

## Workflow

1. Inventory the exact smell and identify the architectural seam it crosses: input parsing, dependency wiring, import graph, or error surface.
2. Fix the seam first. Prefer boundary parsing, composition-root injection, direct module imports, and explicit error types over local patches.
3. Update every implicated call site and test. Do not leave legacy parallel APIs or compatibility shims behind.
4. Validate with the project's standard Python checks after the refactor lands.

## Pattern Guides

Choose the playbook that matches the dominant smell. Read the relevant file unless the task spans multiple problems.

- Strong internal types: [references/strong-types.md](references/strong-types.md)
- Risky signature defaults: [references/signature-defaults.md](references/signature-defaults.md)
- Heavyweight package imports: [references/init-imports.md](references/init-imports.md)
- Catch-all error handling: [references/catch-all-errors.md](references/catch-all-errors.md)

## Execution Rules

- Parse once at boundaries; keep core logic typed and explicit.
- Keep __init__.py cheap; import exact modules instead of package barrels.
- Make critical inputs required in core code; defaults belong at boundaries.
- Treat unknown payloads and unexpected branches as observable failures, not silent drops.
- Prefer a small dedicated type or error module per concept when that improves ownership and clarity.
- Keep the response focused on findings, code changes, updated tests, and validation status.

## Validation

Use the standard quality-gate order:

- uv run ruff format
- uv run ruff check --fix
- uv run ty check
- uv run tq check
- uv run pytest -q
