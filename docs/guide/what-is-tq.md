# What is tq?

`tq` is a test-quality checker that enforces test quality contracts for Python repositories.

It inspects the relationship between source modules and test modules, then reports deterministic findings with stable rule IDs. The goal is simple: keep test suites discoverable, focused, actionable, and maintainable as your codebase evolves.

## Why use tq?

- Protect long-term test structure quality during refactors.
- Catch missing, orphaned, or structurally inconsistent tests early.
- Keep CI output machine-readable and stable for automation.
- Apply explicit quality contracts inspired by `ruff` and `ty`.

## What tq checks

Built-in rules currently cover:

- source-to-test mapping
- structure alignment
- max test file size
- orphaned tests

See the full [Rules Index](../reference/rules/index.md) for rule IDs and behavior.

## Design stance

`tq` follows a strict operator surface:

- subcommand-first CLI (`tq check`)
- deterministic diagnostics
- stable rule IDs
- strict configuration and precedence

## Next step

Go to [QuickStart](./quickstart.md) to install and run `tq` in a project.
