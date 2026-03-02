# Test Quality Toolkit

tq inspects a codebase's tests and enforces quality rules so tests remain discoverable, focused, actionable, and maintainable.

## Getting Started

Install dependencies:

```sh
uv sync
```

Install pre-commit hooks (including `pre-commit`, `commit-msg`, and `pre-push`):

```sh
uv run prek install
```

Run tests:

```sh
uv run pytest -q
```
