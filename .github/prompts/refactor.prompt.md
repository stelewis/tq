---
agent: agent
---

Your task is to support a ground up refactor of the codebase according to the
goals specified by the user. Think through design intent and context and make
source code changes to achieve our long term goals. You are empowered to remove
modules and tests entirely and start test files afresh as needed. Do not be
constrained by what already exists. Think as if you were building anew. If you
detect areas of improvement, logical flaws, or other issues, please address
them.

Refactor all code to align with the new goals outlined as part of the task.

## Refactor Implementation

- MUST NOT assume existing architecture or code is correct
- MUST NOT constrain new designs by trying to maintain compatibility or avoid breaking changes
- MUST NOT implement backward compatibility or legacy code
- MUST NOT write code or tests in a way that re-introduces the old architecture

- MUST strive for software engineering best practices and design principles (SOLID, etc.)
- MUST strive for architectural excellence even if it requires significant changes
- MUST adhere to the [Testing Standards](../../docs/developer/standards/tests.md)
- MUST ensure test modules are also properly refactored

## Outdated Tests

Do not leave monolithic test modules that test multiple source modules.

For each test module implicated in a refactor:

- **Rewrite** if it validates a real contract but is tied to old names or old signatures
- **Split** if it tests multiple source modules
- **Merge** if there is redundancy across multiple test modules
- **Fix** if it is logically correct and only needs mechanical updates
- **Delete** if it encodes old layering/semantics
- **Replace** if it is large, fragile, or asserts legacy constructs

I recognize this is a big refactor, but it is absolutely critical that we do it
right to ensure we have clean, maintainable, and elegant code that aligns with
our vision of the future.

## Quality Gates

Ensure all checks pass:

- `uv run ruff format`
- `uv run ruff check --fix`
- `uv run ty check`
- `uv run tq check`
- `uv run pytest -q`
