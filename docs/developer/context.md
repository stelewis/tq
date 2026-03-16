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

These goals define the product quality model and inform both the built-in rules and the repository's own testing standards.

## Quality model

`tq` is implemented in Rust, but the product contract it enforces is a structural quality model for Python test suites.

That model assumes:

- each source module should have direct mirrored unit-test coverage,
- unit tests should stay focused on one module or contract,
- stable qualifiers may split large suites by concern without losing discoverability,
- workflow-spanning coverage belongs in integration or e2e paths rather than unit-test layouts,
- orphaned, misplaced, and oversized unit tests are design problems worth surfacing early.

These expectations are the rationale behind the built-in rules and user-facing guidance.

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

See the [Rules reference](../reference/rules/index.md) for canonical rule IDs and behavior.

## Current non-goals

The first public release does not include:

- semantic misnaming detection,
- cross-module coupling detection,
- redundant-by-semantics detection,
- vacuous-test detection,
- auto-fix mode.

These are on the roadmap for future release.
