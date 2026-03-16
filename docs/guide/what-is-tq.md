# What is tq?

`tq` enforces test quality contracts for Python repositories.

It inspects how source modules map to tests and reports deterministic findings with stable rule IDs. Use it to keep test suites discoverable, focused, actionable, and maintainable as your codebase evolves.

## Why use tq?

- Keep test structure healthy through refactors.
- Catch missing, orphaned, and mismatched tests early.
- Feed stable, machine-readable output into CI automation.
- Apply explicit quality contracts with `ruff`/`ty`-style ergonomics.

## What tq checks

Built-in rules cover:

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

Go to [Quickstart](./quickstart.md) to install and run `tq`.
