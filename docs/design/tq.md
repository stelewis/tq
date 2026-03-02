# tq – Test Quality Toolkit

This document provides the design context and rationale for the `tq` test quality toolkit.

## Context

The goal of this project is to build a tool that inspects a codebase's tests and enforces quality rules so tests remain discoverable, focused, actionable, and maintainable.

The genesis of this project is the realization that test quality standards are hard to maintain. When working inside agentic workflows, monolithic and orphaned tests became increasingly common, and it was clear that the test suite needed to be treated as a first-class part of the codebase, with strict standards of excellence.

`ruff` and `ty` demonstrated the power of automated linting for acting as rails to keep code quality high and consistent. The goal with `tq` is to bring that same level of rigor to test quality.

The goal is to keep tests:

- **Discoverable**: easy to find the test for a module.
- **Focused**: small surface area, minimal cross-module coupling.
- **Actionable**: failures point to one contract, not "the system".
- **Maintainable**: tests refactor with the code (SOLID applies here too).

While avoiding the following antipatterns:

- **Monolithic unit tests**: one test module covering many unrelated modules.
- **Cross-module unit tests**: tests with no clear single target.
- **Duplicated coverage**: multiple suites asserting the same contract.
- **Redundant tests**: re-testing behavior already validated elsewhere.
- **Structure mismatches**: test location doesn’t mirror the source module.
- **Misnamed tests**: name implies a different target than what it covers.
- **Orphaned tests**: tests primarily covering code that no longer exists.
- **Vacuous tests**: tests that pass without meaningfully exercising behavior.
- **Very large test modules**: tests that try to cover too much in one suite.

The test standards are outlined in [docs/developer/standards/tests.md](./developer/standards/tests.md).

Code standards are in [docs/developer/standards/code.md](./developer/standards/code.md).

## Test Quality Checker

The original `check_test_quality` tool was built in a specific codebase and validated:

1. **Mapping**: Each source module has at least one test file
2. **Structure**: Test files are in the correct directory structure
3. **Size**: Test files don't exceed the configured maximum lines (default: 600)
4. **Orphans**: Test files correspond to existing source modules

*Cross-module tests, duplicated coverage, misnamed-by-semantics, redundant-by-semantics, and vacuous tests were not included due to noisy heuristics.*

That legacy implementation has now been fully removed in favor of the native
`tq check` architecture and command surface.

## Problem

`check_test_quality` was a great start and provided immediate value, but it was built essentially as a developer tooling script for a specific codebase. It wasn't designed for reuse or extensibility, which made reuse across projects unsustainable.

## Goal

Instead of porting the old scripts across to each project, the goal with `tq` is to provide these test lints/checks as a fully open source, reliable tool that anyone can use and expand.

In terms of design aesthetic and ergonomics, `ruff` and `ty` serve as inspiration. `tq` should follow the same design pattern wherever possible.
