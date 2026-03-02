# Copilot Instructions

Description: tq inspects a codebase's tests and enforces quality rules so tests remain discoverable, focused, actionable, and maintainable.

Repository: stelewis/tq

## Development Commands

Project uses `uv`.

### Code Quality

- Format: `uv run ruff format`
- Lint: `uv run ruff check --fix`
- Type check: `uv run ty check`
- Tests: `uv run pytest`

### Common Tool Calls

- Python: `uv run python <args>`
- Project: `uv run tq <args>`
- File system operations: `git mv`, `git rm`, `mv`, `rm`

### Multiline Command Line Issues

- Terminal wrapper may not manage multi-line strings correctly.
- You can create a temporary script file in `tmp/` to run complex commands.
- You do not need to quality gate these scripts.

### Full Validation

Before committing, run all checks:

```bash
uv run ruff format && uv run ruff check --fix && uv run ty check && uv run tq check && uv run pytest -q
```

## Guidelines

- MUST NOT assume existing design, architecture or code is correct.
- MUST NOT implement legacy or backward compatibility code:
  - MUST remove outdated modules, APIs and functions.
  - MUST refactor old code to align with current architecture and design.
  - MUST NOT preserve database or API schemas or implement migrations for legacy support.
- MUST NOT constrain new designs by trying to maintain compatibility or avoid breaking changes.
  - MUST strive for architectural excellence even if it requires significant changes.
  - MUST NOT take a convenience driven approach that compromises design quality.
- MUST ensure that test modules are properly refactored when source code changes (split, merge, replace, delete).
- MUST develop clean, maintainable, well factored, and elegant code.
- MUST keep package `__init__.py` files lightweight:
  - MUST NOT use `__init__.py` as a re-export hub (`__all__` barrels).
  - MUST prefer importing exact module paths.

**Correctness first, design forward.**
