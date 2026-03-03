# Context

Design context for `tq`.

## Problem

Test quality standards drift over time, especially in fast-moving codebases and agentic workflows. Monolithic tests, orphaned tests, and structure drift make failures less actionable and refactors harder.

`tq` enforces stable quality contracts so test suites stay navigable and maintainable.

## Goals

`tq` keeps tests:

- discoverable,
- focused,
- actionable,
- maintainable.

These goals map directly to the [testing standards](./standards/tests.md) in this project.

## Design stance

`tq` follows a `ruff`/`ty` style operator surface:

- subcommand-first CLI (`tq check`),
- deterministic diagnostics,
- stable rule IDs,
- strict configuration and precedence.

## Rule scope

Built-in rules cover:

- source-to-test mapping,
- structure alignment,
- max test file size,
- orphaned tests.

See [docs/reference/rules/index.md](../reference/rules/index.md) for canonical rule IDs and behavior.

## Current non-goals

The first public release does not include:

- semantic misnaming detection,
- cross-module coupling detection,
- redundant-by-semantics detection,
- vacuous-test detection,
- auto-fix mode.

These are on the roadmap for future release.
