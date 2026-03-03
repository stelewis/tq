---
title: Multi-target project scope for tq check
date_created: 2026-03-03
---

# Implementation Plan: Multi-target project scope for tq check

`tq` currently models one package/source/test scope per run, so default project configuration can miss first-party Python modules outside the main package path (for example `scripts/`). The long-term fix is to make scope explicit and first-class: define one or more analysis targets in config, then run `tq check` across all targets by default with deterministic ordering and strict validation. This aligns with Ruff and Ty ergonomics: explicit target selection, strict config, predictable precedence, and CLI overrides that do not silently change meaning.

## Architecture and design

## Design goals

- Keep configuration strict and fail-fast.
- Keep operator UX explicit and low-surprise.
- Keep rule engine pure and target-agnostic.
- Avoid compatibility shims and legacy coercion.

## Proposed model

- Replace single-scope runtime config with a target-based model.
- Add required `[[tool.tq.targets]]` entries.
- Each target is an independent analysis unit with explicit boundaries.

Required target fields:

- `name`: stable target identifier (kebab-case, unique per project)
- `package`: import package to analyze
- `source_root`: source root for that package
- `test_root`: test root for that package

Optional target fields (override project defaults):

- `ignore_init_modules`
- `max_test_file_non_blank_lines`
- `qualifier_strategy`
- `allowed_qualifiers`
- `select`
- `ignore`

Project-level `[tool.tq]` keeps shared defaults and operational flags; targets provide strict per-scope boundaries.

## CLI surface

- `tq check` runs all configured targets.
- `tq check --target <name>` runs one target (repeatable).
- `tq check --config ...` and `--isolated` preserve existing precedence semantics.
- Exit code is computed from aggregate findings using current severity policy.

## Why this is the best-practice direction

- Mirrors Ruff and Ty path/scope ergonomics by making analysis scope explicit.
- Removes hidden assumptions that all first-party Python lives under one package root.
- Keeps contracts strict: no heuristic discovery of “extra code trees.”
- Scales to future repository structure changes without reworking rule logic.

## Tasks

## Phase 1: Config and models

- Introduce target dataclass(es) in config models.
- Refactor loader to parse and validate `[[tool.tq.targets]]` strictly.
- Remove single-scope-only materialization code.
- Add strict invariants:
  - at least one target required,
  - unique target names,
  - non-empty required fields,
  - no duplicate effective source package roots.

## Phase 2: Engine orchestration

- Introduce a run planner that materializes one `AnalysisContext` per target.
- Execute built-in rules per target and merge findings.
- Preserve deterministic target and finding ordering.
- Include target name in machine-readable output payloads.

## Phase 3: CLI and reporting

- Add `--target` filtering.
- Update terminal and JSON reporters to show target context.
- Keep stable rule IDs and existing severity vocabulary unchanged.

## Phase 4: Tests and fixtures

- Refactor config tests to validate strict target parsing and precedence.
- Add CLI tests for default all-target execution and `--target` filtering.
- Add integration test with at least two targets (`tq` and `scripts`).
- Ensure one test module per source module for any newly introduced modules.

## Phase 5: Documentation and migration

- Update configuration reference with target schema and examples.
- Update CLI reference with `--target` semantics and precedence.
- Add migration notes for replacing single-scope config with target entries.
- Regenerate any docs artifacts that are source-of-truth generated.

## Phase 6: Additional polish and verification

- Add stricter `package` validation beyond non-empty strings.
  - Require dotted Python identifier segments.
  - Fail fast for invalid import package syntax.
- Improve target-entry error precision for arrays.
  - Include target index context where possible (for example `tool.tq.targets[1].name`) to speed troubleshooting.
- Add explicit duplicate-entry policy for list-valued keys.
  - Consider failing on duplicate `allowed_qualifiers` and duplicate rule IDs in `select` / `ignore` rather than silently normalizing.
- Introduce an explicit run-planner unit as a separable orchestration boundary.
  - Keep current behavior identical.
  - Isolate planning from execution to reduce future change surface.

## Verification

- `uv run ruff format`
- `uv run ruff check --fix`
- `uv run ty check`
- `uv run tq check`
- `uv run pytest -q`

Functional acceptance criteria:

- A config with both `tq` and `scripts` targets reports missing tests in either target.
- `tq check --target scripts` scopes findings to `scripts` only.
- Unknown target names fail fast with an actionable error.
- Invalid target entries fail fast with precise key-level errors.
