---
title: End-to-end Rust rewrite for tq
date_created: 2026-03-05
---

# Implementation Plan: End-to-end Rust rewrite for tq

`tq` already has strong contracts (strict config loading, deterministic rule output, stable rule IDs, explicit CLI UX), but it is currently implemented in Python with Python-centric toolchains in CI and release automation. This plan rewrites the product end-to-end in Rust using Ruff/Ty-inspired ergonomics: fast default execution, strict validation, deterministic diagnostics, and a small, explicit operator surface (`tq check`). Because `tq` is pre-release and has no production adoption, this rewrite treats current contracts as a high-value baseline, not a hard constraint. Design-forward improvements and intentional breaking changes are acceptable where they improve long-term architecture, ergonomics, and performance.

## Architecture and design

### Primary design goals

- Keep current contract surfaces explicit and strict:
  - CLI behavior (`tq check`, flags, exit codes)
  - config schema and precedence semantics
  - rule IDs/severity vocabulary and deterministic ordering
  - JSON output schema
- Improve performance and scalability in large codebases with Rust-native filesystem and analysis pipelines.
- Preserve maintainability through narrow crate boundaries, explicit dependency injection at composition root, and pure domain logic.
- Prioritize design excellence over compatibility preservation; avoid legacy adapters and migration coercion in runtime code.

### Proposed Rust workspace topology

- `crates/tq-cli` (binary)
  - Composition root and CLI parsing.
  - Builds runtime graph explicitly from config, planner, rule registry, and reporters.
- `crates/tq-config` (library)
  - Strict config decoding, precedence, validation, and materialization into runtime config.
  - No filesystem scanning or reporting logic.
- `crates/tq-discovery` (library)
  - Python source/test discovery and immutable analysis index construction.
- `crates/tq-engine` (library)
  - Rule orchestration, deterministic finding sort/aggregation, target-scoped planning.
- `crates/tq-rules` (library)
  - Built-in rules (`mapping-missing-test`, `structure-mismatch`, `test-file-too-large`, `orphaned-test`) and shared qualifier policy.
- `crates/tq-reporting` (library)
  - Text and JSON reporting adapters.
- `crates/tq-release` (binary or lib)
  - Replaces release artifact policy checks currently implemented in `scripts/release/verify_artifact_contents.py`.
- `crates/tq-docsgen` (binary)
  - Replaces docs generation scripts under `scripts/docs/` and regenerates manifest-backed docs artifacts.

### Contract and change policy

The current contract does not need to be preserved where it no longer serves the design goals or where Rust-native patterns improve clarity, correctness, and ergonomics.

This tool is pre-release, and the rewrite is an opportunity to improve the design and contract before adoption. Breaking changes are allowed.

Distribution intent is still Python-native even after the runtime rewrite:

- PyPI remains the canonical distribution channel.
- `uv` remains a first-class install and execution path.
- The Rust rewrite changes implementation language, not the package ecosystem the tool belongs to.
- Standalone binaries may exist as supplemental artifacts, but they are not the primary distribution contract.

Default starting points (baseline references, not constraints):

- CLI UX command and flag set documented in `docs/reference/cli.md` and `docs/reference/cli/options-manifest.json`.
- Config semantics in `src/tq/config/loader.py`:
  - unknown keys fail fast
  - explicit precedence order is preserved
  - target roots are resolved relative to the defining config file directory
- Rule identity and severity vocabulary parity from `docs/reference/rules/manifest.json`.
- Deterministic ordering parity with existing engine sort behavior in `src/tq/engine/runner.py`.
- JSON payload shape parity with `src/tq/reporting/json.py`.
- Exit code semantics parity from `docs/reference/exit-codes.md` and CLI tests.

### Migration strategy

- Use a staged replacement with conformance fixtures and golden JSON outputs.
- Keep a temporary comparison harness that runs Python and Rust binaries on the same fixtures and classifies deltas as intentional design changes versus regressions.
- Remove the Python runtime once parity and performance gates pass; do not keep runtime dual paths or any compatibility code long-term.
- Treat docs/release helper scripts as first-class migration scope (end-to-end rewrite means no required Python runtime for core development, CI, release, or docs generation).

### Tooling strategy aligned to Ruff/Ty ergonomics

- Single fast binary (`tq`) with deterministic output and clear diagnostics.
- Rust-first quality stack:
  - formatting: `cargo fmt --check`
  - linting: `cargo clippy --workspace --all-targets -- -D warnings`
  - tests: `cargo test --workspace`
  - security/dependency checks: `cargo audit` and `cargo deny`
- Keep pre-commit hygiene hooks; replace Python-only hooks with Rust equivalents where applicable.

## Tasks

### Phase 0: ADR and scope lock

Status: Completed (2026-03-05).

- Add and accept an ADR defining rewrite scope, cutover rules, and non-goals.
- Lock contract sources of truth:
  - CLI/options manifest
  - rules manifest
  - versioning policy
- Define explicit cutover criteria and rollback plan.
- Create the missing developer tools docs entrypoint at `docs/developer/tools/index.md` and establish it as the canonical location for Rust stack developer tooling and commands as phases land.

### Phase 1: Rust workspace bootstrap

Status: Completed (2026-03-05). Audit Completed.

- Create Cargo workspace and crates listed above.
- Implement `tq-cli` command surface (`check`, help, output-format) with strict argument parsing.
- Set MSRV policy and rustfmt/clippy config committed at repo root.
- Add foundational error model (`thiserror`) with actionable boundary errors.

### Phase 2: Config subsystem port (`tq-config`)

Status: Completed (2026-03-05). Audit Completed.

- Port config models and strict loader semantics from `src/tq/config/models.py` and `src/tq/config/loader.py`.
- Implement exact precedence and isolation behavior (`--config`, `--isolated`, project discovery).
- Enforce strict unknown-key and type validation.
- Port target materialization and validation invariants (including duplicate target and duplicate source package root checks).

### Phase 3: Discovery and engine core (`tq-discovery`, `tq-engine`)

Status: Completed (2026-03-05). Audit Completed.

- Port filesystem discovery semantics from `src/tq/discovery/filesystem.py`.
- Implement immutable analysis index and context models.
- Port planning and deterministic aggregation semantics from:
  - `src/tq/engine/planner.py`
  - `src/tq/engine/runner.py`
  - `src/tq/engine/models.py`

### Phase 4: Rule system port (`tq-rules`)

Status: Completed (2026-03-05). Audit Completed.

- Port qualifier policy and all built-in rules with contract-conformance intent and explicit allowance for design improvements:
  - `mapping-missing-test`
  - `structure-mismatch`
  - `test-file-too-large`
  - `orphaned-test`
- Keep stable rule IDs and severity defaults aligned with `docs/reference/rules/manifest.json`.
- Port rule selection/ignore behavior and unknown rule ID errors.

### Phase 5: Reporting adapters (`tq-reporting`)

Status: Completed (2026-03-10).

- Port terminal and JSON reporting behavior from:
  - `src/tq/reporting/terminal.py`
  - `src/tq/reporting/json.py`
- Preserve deterministic JSON field ordering and text output structure.
- Preserve optional suggestions rendering behavior.

### Phase 6: End-to-end conformance harness

Status: Completed (2026-03-10).

- Add fixture projects and expected outputs for all contract categories:
  - clean run
  - each rule trigger
  - invalid config and unknown keys
  - target scoping and target selection
  - deterministic ordering edge cases
- Build a conformance test command that compares Python baseline output with Rust output until cutover and emits an intentional-delta report.

### Phase 7: CI and quality hook migration

Status: Completed (2026-03-10).

- Replace Python-focused CI jobs with Rust-native jobs while preserving governance intent:
  - lint/type/format gates
  - deterministic output checks
  - package/build checks
  - docs-sync contract checks
  - security scans
- Update `.pre-commit-config.yaml` to Rust-oriented checks and keep or migrate hygiene/secret scanning/commit policy hooks.
- Replace CodeQL language configuration from Python to Rust.
- Complete end-to-end CI/CD workflow, actions, and job audit.
- Adopt Rust 1.94.0 as the product MSRV across workspace policy, tooling, and CI.
- Keep Rust security scanners on an explicitly documented stable-toolchain bootstrap until scanner installation can rely on released, policy-approved versions without manual pin review.

### Phase 8: Docs/release pipeline rewrite

Status: Completed (2026-03-10).

- Port docs generators in `scripts/docs/` to `tq-docsgen` and preserve generated artifact contracts:
  - `docs/reference/cli.md`
  - `docs/reference/configuration.md`
  - `docs/guide/quickstart.md`
  - `docs/.vitepress/generated/rules-sidebar.ts`
  - `docs/reference/rules/index.md` and rule pages
- Port release artifact policy checks from `scripts/release/verify_artifact_contents.py` to Rust.
- Rewrite developer tooling/tools/workflow/CI docs at `docs/developer/tools/index.md` (refactoring into relevant standalone pages) to reflect Rust toolchain and commands.
- Keep docs and release automation aligned with the Python-package distribution contract.

### Phase 9: Packaging and distribution cutover

Status: Completed (2026-03-10).

- Define distribution strategy:
  - PyPI remains the canonical distribution channel
  - `uv add`, `uvx`, and `uv tool install` are the install surfaces
  - standalone binaries are optional supplemental artifacts, not the primary contract
- Choose and implement a Rust-backed Python packaging approach that emits publishable wheel and sdist artifacts while preserving the `tqlint` PyPI distribution contract.
- Collapse the installed runtime surface to the single `tq` command even though the PyPI package name remains `tqlint`.
- Update install docs and release workflow accordingly.
- Replace the interim `cargo metadata` manifest-validation CI gate with a publish-ready `cargo package` or equivalent dry-run packaging validation once the workspace crates are intentionally prepared for distribution.

### Phase 10: Decommission and remove Python runtime

Status: Completed (2026-03-10).

- Remove `src/tq` runtime implementation, the transitional Python runtime dependencies that only support it, and obsolete Python CI paths.
- Remove conformance harness and fixtures once Python runtime is removed; keep only if needed for ongoing regression testing during Rust iteration.
- Keep only minimal Python packaging and integration files strictly needed to preserve the PyPI-and-`uv` distribution surface.
- Remove all legacy or compatibility code paths in Rust runtime.
- Update docs to reflect Rust architecture as canonical.
- Remove any transition-oriented language or docs drift.
- Update policy enforcement and docs to remove Python-runtime-specific policies such as `src/tq` exclusion from artifact content policy.

## Verification

### Contract parity gates

- [x] CLI acceptance tests for all existing flags and exit code semantics.
- [x] Config conformance tests for strict validation and precedence.
- [x] Rule behavior tests mapped 1:1 to rule manifest expectations.
- [x] JSON output schema and determinism tests across repeated runs and OS matrix.

### Quality gates

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo audit`
- [x] `cargo deny check`

### Performance gates

- [x] Ensure no regressions in deterministic output while optimizing traversal and evaluation.

### Release and supply-chain gates

- [x] Build reproducible release artifacts in CI.
- [x] Keep GitHub provenance attestation and verification policy equivalent to current publish workflow.
- [x] Smoke-test installed artifacts and command entrypoints before publish.

### Documentation gates

- [x] Regenerate docs artifacts via Rust docs generator and verify no diff drift in generated files.
- [x] Ensure architecture and contributor docs reflect Rust-first build, test, and release commands.
- [x] Ensure `docs/developer/tools/index.md` is complete and current for the post-cutover Rust toolchain.
- [x] Ensure no transition oriented language or docs drift remains after cutover.
