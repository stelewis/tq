---
id: 0003
title: "Adopt a design-forward Rust rewrite and cutover policy"
status: accepted
date: 2026-03-05
tags:
  - architecture
  - rust
  - rewrite
  - governance
supersedes: null
superseded_by: null
---

## Context

`tq` is currently implemented in Python, with Python-first runtime, quality gates, release checks, and docs generation scripts. The project design goals and operator ergonomics are inspired by Ruff and Ty, both Rust-native tools focused on speed, strictness, and deterministic behavior.

The project is pre-release and has no adoption constraints requiring backward compatibility. Existing contracts are useful baselines, but we should not carry legacy constraints or compatibility layers when they conflict with better architecture and long-term maintainability.

## Decision

`tq` will execute a full end-to-end Rust rewrite with design excellence and performance as first-order priorities.

### Scope and policy

- Rust becomes the canonical implementation language for runtime, CI-quality checks, release validation logic, and docs generation tooling.
- Breaking changes are allowed when they improve architecture, ergonomics, correctness, or performance.
- Runtime backward compatibility adapters, schema coercions, and legacy migrations are disallowed in core Rust runtime paths.

### Contract source-of-truth lock

The following are locked as baseline contract references for rewrite planning and conformance review:

- CLI options baseline:
  - `docs/reference/cli.md`
  - `docs/reference/cli/options-manifest.yaml`
- Rule identity and severity baseline:
  - `docs/reference/rules/manifest.yaml`
- Versioning/release-impact policy baseline:
  - `docs/developer/versioning.md`

These references are baselines, not immutability constraints. Intentional changes are allowed, but must be documented in the same PR that implements runtime behavior changes.

### Developer tooling documentation

`docs/developer/tools/index.md` is established as the canonical entrypoint for Rust developer tooling and command documentation, and must be kept aligned with CI and pre-commit hook behavior.

### Cutover criteria

Cutover from Python runtime to Rust runtime requires all of the following:

- Rust CLI provides canonical `tq` behavior for `check` and documented operator workflows.
- Conformance harness is green, with remaining deltas explicitly classified as intentional design changes, not regressions.
- Rust quality, test, and security gates are green in CI.
- Release workflow and artifact policy checks run on Rust artifacts and pass attestation verification policy.
- Docs generation contracts pass and developer docs reflect Rust-first workflows.
- Python runtime implementation is removed from production execution paths.

### Rollback policy

- Before cutover: continue iterating on Rust while Python remains the released runtime.
- During cutover window: if required cutover criteria fail, do not promote Rust as canonical; fix forward and retry.
- After cutover: rollback means releasing a corrective Rust patch; do not reintroduce dual-runtime compatibility architecture.

## Consequences

- The project can optimize architecture and UX without carrying legacy baggage; codebase remains clean, maintainable, strict, and performant.
- Rewrite velocity improves because compatibility-preservation work is intentionally excluded.
- Docs and governance become clearer: one canonical toolchain and one canonical tooling documentation location.
- Contributors must align changes with explicit contract-source updates and cutover gates.

## Alternatives considered

### Preserve full Python contract compatibility

Rejected. The project is pre-release and this would add cost and complexity without user benefit, while blocking design improvements.

### Partial rewrite (Rust core with permanent Python orchestration)

Rejected as the end-state. It reduces immediate migration cost but leaves long-term split ownership, operational complexity, and technical debt.

### Greenfield Rust rewrite without contract baselines

Rejected. Baselines are still valuable for conformance checks, regression awareness, and disciplined intentional change tracking.

## Related

- [Project context](../developer/context.md)
- [Code standards](../developer/standards/code.md)
- [Versioning policy](../developer/versioning.md)
