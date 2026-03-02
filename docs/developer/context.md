# tq context

This document explains the design context for `tq`.

## Problem

Test quality standards drift over time, especially in fast-moving codebases and agentic workflows. Monolithic tests, orphaned tests, and structure drift make failures less actionable and refactors harder.

`tq` exists to enforce stable quality contracts so test suites remain navigable and maintainable.

## Goals

`tq` keeps tests:

- discoverable,
- focused,
- actionable,
- maintainable.

These map directly to the testing standards in [docs/developer/standards/tests.md](./standards/tests.md).

## Design stance

`tq` follows a `ruff`/`ty` style operator surface:

- subcommand-first CLI (`tq check`),
- deterministic diagnostics,
- stable rule IDs,
- strict configuration and precedence.

## Rule scope

The initial built-in rules cover:

- source-to-test mapping,
- structure alignment,
- max test file size,
- orphaned tests.

See [docs/developer/tools/rules.md](./tools/rules.md) for canonical rule IDs and behavior.

## Current non-goals

The first public release does not yet include:

- semantic misnaming detection,
- cross-module coupling detection,
- redundant-by-semantics detection,
- vacuous-test detection,
- auto-fix mode.
