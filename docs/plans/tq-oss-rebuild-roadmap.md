---
title: tq OSS rebuild roadmap
date_created: 2026-03-02
---

# Implementation Plan: tq OSS rebuild roadmap

Rebuild `tq` as a general-purpose open source test quality linter with a `ruff`/`ty` style operator surface (`tq check`) and a strict, extensible architecture. The imported `test_quality` implementation is treated as discovery input only. The new implementation should be package-agnostic, rule-driven, and designed for incremental expansion (new checks without runner churn). Once parity for core checks is achieved, remove the imported legacy modules and keep only the new architecture.

## Architecture and design

## Product shape

- Primary interface: `tq check`.
- Exit semantics:
  - `0`: no error-level findings.
  - `1`: one or more error-level findings.
  - `2`: invalid invocation or configuration.

## Core boundaries

- Composition root in CLI layer only.
- Pure domain layer for rule evaluation and finding modeling.
- Discovery/index layer for filesystem traversal and module-test mapping.
- Config adapter layer for `pyproject.toml` parsing and validation.
- Reporter layer for console output and machine-readable formats.

## Domain model

- Introduce stable finding model fields for long-term tooling integration:
  - `rule_id`, `severity`, `message`, `path`, `line`, `suggestion`.
  - Align with <https://docs.astral.sh/ruff/> and <https://docs.astral.sh/ty/> conventions where possible.
  - See repos at `astral-sh/ruff` and `astral-sh/ty` for reference implementations.
- Define rule contract (protocol/ABC) that receives immutable analysis context and returns findings.
- Define canonical severity taxonomy and central exit policy mapping.

## Configuration model

- Move from `[tool.test_quality]` to `[tool.tq]`.
- Keep explicit check settings (no silent coercion).
- Include target roots and package mapping in config instead of hard-coded `src/tq` and `tests`.
- Add strict validation with actionable errors when config is incomplete or contradictory.

## Rule system

- Re-implement initial built-in rules under new IDs:
  - mapping coverage.
  - structure alignment.
  - max test file size.
  - orphaned tests.
- Keep rule implementations independent and stateless.
- Provide rule selection controls (`--select`, `--ignore`) with deterministic resolution.

## Package and module structure

- Proposed target layout:
  - `src/tq/cli/` for command parsing and composition root.
  - `src/tq/config/` for schema and pyproject adapter.
  - `src/tq/discovery/` for filesystem walking and index creation.
  - `src/tq/rules/` for rule contracts and built-ins.
  - `src/tq/engine/` for orchestration and result aggregation.
  - `src/tq/reporting/` for console and future machine output.
- Keep package `__init__.py` files minimal (no re-export hubs).

## Tasks

## Phase 1: Define and freeze the external contract

- Add architecture note in `docs/adr/` documenting CLI contract, exit codes, and configuration namespace.
- Specify v1 rule IDs and severity policy in developer docs.
- Define migration policy from legacy `check_test_quality` script to `tq check`.

## Phase 2: Build a clean engine and rule API

- Implement analysis context and immutable index model.
- Implement rule interface and orchestration engine.
- Implement finding aggregation, summary, and deterministic sorting.
- Add unit tests for engine behavior, including no-rules and multi-rule scenarios.

## Phase 3: Re-implement v1 built-in rules

- Build mapping rule against new index model.
- Build structure rule against new index model.
- Build size rule with explicit counting policy.
- Build orphan rule with explicit qualifier strategy.
- Add focused tests per rule and golden-style integration tests for representative trees.

## Phase 4: Deliver CLI surface and configuration

- Implement `tq check` command and argument parsing.
- Implement strict config loading from `pyproject.toml` `[tool.tq]`.
- Implement override precedence: CLI flags over config values.
- Provide readable terminal output and concise summaries.
- Update `pyproject.toml` scripts to expose `tq` command.

## Phase 5: Packaging, migration, and cleanup

- Update `README.md` and `docs/developer/tools/tq_check.md` to reflect command and config contract.
- Add migration notes for users moving from `[tool.test_quality]` and `check_test_quality`.
- Remove imported legacy modules under `src/tq/tools/test_quality/` after parity verification.
- Refactor tests to target only new architecture and delete legacy-specific test fixtures.

## Phase 6: Hardening and expansion hooks

- Add machine-readable reporter format (JSON first, SARIF later if needed).
- Add stable rule docs page listing rule IDs, examples, and fix guidance.
- Add CI matrix checks to ensure deterministic outputs across Python versions.

## Verification

- Quality gates for each phase:
  - `uv run ruff format`
  - `uv run ruff check --fix`
  - `uv run ty check`
  - `uv run pytest -q`
- Add end-to-end CLI tests that execute `tq check` against fixture repos.
- Validate deterministic output ordering and exit code behavior.
- Validate migration path with one legacy-style fixture repo and one greenfield fixture repo.

## Out of scope for first release

- Semantic misnaming detection.
- Cross-module coupling detection.
- Redundant-by-semantics and vacuous-test detection.
- Auto-fix mode.
