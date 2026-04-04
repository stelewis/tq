---
title: Thinking-in-Rust Domain Hardening
date_created: 2026-04-04
---

# Implementation Plan: Thinking-in-Rust Domain Hardening

Refactor the Rust workspace to replace stringly and boolean-heavy runtime modeling with a small set of shared domain types and typed validation errors, while preserving the existing CLI and rule behavior. The main change is to parse and validate target identity, package naming, and target-relative paths at the config boundary, then carry those validated types through `tq-config`, `tq-engine`, `tq-rules`, and `tq-reporting` instead of repeatedly converting between `String`, `PathBuf`, and ad hoc booleans. This plan is downstream of the design-forward rewrite policy in [ADR 0003](../adr/0003-rust-rewrite-design-forward-cutover.md) and the architecture boundaries in [developer architecture](../developer/architecture.md).

## Architecture and design

The goal is not to add more abstractions. The goal is to move existing invariants into types so the compiler enforces them and the crate boundaries become cleaner.

The highest-value changes are:

- Introduce shared validated types for target names and package-relative paths in `tq-core` so they can be reused across config loading, engine planning, finding rendering, and built-in rules.
- Keep `InitModulesMode` as an enum end to end instead of collapsing it to a `bool` in the CLI-to-rules pipeline.
- Replace generic `Validation { message: String }` error variants in library crates with typed error variants that describe concrete failure modes.
- Narrow public APIs to borrowed path views and domain types instead of exposing implementation-oriented storage types like `&PathBuf`.

The plan should not change the user-facing CLI contract unless a stronger internal model makes a documented contract adjustment clearly better. CLI flags such as `--isolated` and `--show-suggestions` remain normal command-surface booleans. Partial config parsing types may continue to use `Option` at the boundary because they represent incomplete user input, not validated runtime state.

The plan should not force a closed-set rule engine if the project still wants an open rule trait boundary. Trait-object dispatch is not a required part of this refactor.

## Scope and non-goals

In scope:

- `crates/tq-core`
- `crates/tq-config`
- `crates/tq-engine`
- `crates/tq-rules`
- `crates/tq-reporting`
- Targeted docs and tests that describe or lock the new runtime modeling

Out of scope:

- Reworking CLI syntax or flag names
- Replacing trait-object rule dispatch solely for style reasons
- Large behavior changes to rule semantics
- Rewriting partial TOML parse models into fully typed runtime models before materialization

## Tasks

### 1. Define the shared domain model in `tq-core`

- Add a validated `TargetName` newtype for target identity.
- Add a validated package-domain type instead of passing package names and target package paths around as bare strings. Choose one clear model and use it consistently:
  - Either a `PackageName` newtype that understands dotted Python package syntax and can derive a relative path.
  - Or a `RelativePackagePath` newtype if path semantics are the true canonical representation.
- Add any small helper types needed to avoid mixing display-only strings with semantic paths. Prefer one semantic path type plus formatting helpers over a separate display-string field.
- Keep fields private and require parsing or constructor functions so invalid values cannot be reintroduced downstream.
- Derive only the traits needed by the current crates and tests.

### 2. Move parsing and validation to the config boundary

- Refactor `tq-config` materialization to produce validated runtime types instead of raw `String` and ad hoc path/string mixtures.
- Update `TqTargetConfig` to store `TargetName` and the chosen package-domain type.
- Replace `source_root()` and `test_root()` accessors that expose `&PathBuf` with borrowed `&Path` views.
- Keep partial config structs permissive, but ensure materialization is the single place where runtime invariants become guaranteed.
- Remove duplicate downstream checks that become impossible once construction uses validated types.

### 3. Propagate validated types through engine planning and findings

- Refactor `TargetContext` and `TargetPlanInput` in `tq-engine` to use the new target and package types rather than raw strings.
- Remove redundant string validation in constructors once callers can only pass validated types.
- Update `Finding` to carry a validated target identity type instead of `Option<String>`.
- Review finding sort keys and rendering helpers so they operate on the new types without lossy conversions.
- Keep output formatting behavior stable unless a bug is exposed by stronger typing.

### 4. Preserve enum meaning end to end for init-module behavior

- Change `BuiltinRuleOptions` and `MappingMissingTestRule` to store `InitModulesMode` rather than `ignore_init_modules: bool`.
- Remove `should_ignore()` if it becomes unnecessary, or keep it only as a local helper where it does not erase the main API meaning.
- Update the CLI composition root to pass the enum through directly.
- Re-check tests for rule construction and mapping behavior so the refactor preserves current semantics.

### 5. Replace catch-all validation strings with typed library errors

- Introduce concrete error variants in `tq-config`, `tq-engine`, `tq-rules`, and `tq-discovery` for the currently observed invariant failures.
- Prefer variants that carry structured fields where useful, such as the offending path, numeric value, or target name.
- Keep `thiserror` messages concise and operator-facing, but do not use a generic string bucket for domain validation once the failure shapes are known.
- Update callers and tests to match on variants where appropriate instead of asserting only on formatted strings.
- Preserve source chaining for I/O and parse failures.

### 6. Simplify rules and helpers after the type migration

- Remove helper functions in `tq-rules` that only exist to translate stringly context into paths if that translation can now happen once at construction time.
- Revisit `package_path_from_context`, `known_target_package_paths_from_context`, and `test_root_display_from_context` and either delete them or reduce them to thin, typed accessors.
- Replace repeated trimming and normalization of qualifier lists or target values only when the stronger boundary model makes that cleanup obviously safe.
- Keep rule behavior deterministic and output ordering unchanged.

### 7. Update documentation and contract references

- Document the new runtime domain model briefly in developer docs if the new types materially clarify crate boundaries.
- Update any docs or contract fixtures that currently describe raw string expectations if those expectations move behind typed runtime APIs.
- If any user-visible behavior changes intentionally, document it in the same change set per ADR 0003.

## Execution order

Implement this in narrow, reviewable slices rather than a single sweep.

1. Add the new shared types and tests in `tq-core`.
2. Migrate `tq-config` materialization and `TqTargetConfig` to the new types.
3. Migrate `tq-engine` planning and finding models.
4. Migrate `tq-rules` option passing and context helpers.
5. Replace generic validation error variants with typed variants as the old constructors disappear.
6. Update reporting and docs last, once the core types settle.

Each slice should compile and pass its crate-level tests before moving on.

## Verification

- Run `cargo fmt --all --check`.
- Run `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Run `cargo test --workspace --locked`.
- Run `cargo run -p tq-docsgen --locked -- generate all` if docs or contract outputs change.
- Review public APIs in touched crates to confirm they now expose domain types or borrowed views instead of raw storage types where practical.
- Add or update tests that prove invalid target names, invalid package values, invalid relative package paths, and invalid finding metadata fail at construction with typed errors.
- Add regression tests that preserve current mapping, orphaned-test, and output-reporting behavior.

## Success criteria

- Runtime target and package concepts are no longer passed across crate boundaries as unconstrained strings.
- `InitModulesMode` survives from config resolution to rule execution without being collapsed to a boolean API.
- Library validation failures are represented by typed error variants rather than generic string buckets.
- Public runtime APIs no longer expose `&PathBuf` where `&Path` or a domain type is sufficient.
- Existing CLI behavior and rule results remain stable unless an intentional, documented improvement is made.
