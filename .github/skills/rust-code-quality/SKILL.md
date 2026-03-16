---
name: rust-code-quality
description: Improves Rust code quality by fixing architectural refactor smells such as stringly typed IDs, weak boundary validation, broad re-export barrels, hidden IO in domain code, and catch-all error handling. Use when a Rust task involves hardening crate or module boundaries, replacing raw strings with enums or newtypes, tightening error surfaces, or simplifying internal APIs after a refactor.
argument-hint: describe the Rust code smell, affected crates or modules, and the invariant you want after the refactor
user-invocable: true
disable-model-invocation: false
---

# Rust Code Quality

Use this skill for Rust refactors that need structural cleanup rather than cosmetic lint fixes.

## When To Use It

- The task is to harden Rust code, refactor internals, or improve crate and module boundaries.
- The smell involves raw string IDs, hidden IO, weak validation, broad `lib.rs` barrels, or opaque error handling.
- The change needs coordinated updates across source, call sites, and tests.

## Workflow

1. Identify the boundary that is wrong: parsing, validation, dependency wiring, module ownership, or error surface.
2. Fix the boundary first. Prefer typed models, strict parsing, composition-root construction, and direct module ownership over local patches.
3. Update every implicated caller and test. Do not leave legacy parallel APIs or re-export shims behind.
4. Validate with the standard Rust quality gates for the affected scope.

## Pattern Guides

Read the reference that matches the dominant smell unless the task spans multiple problems.

- Strong internal types: [references/strong-types.md](references/strong-types.md)
- Boundary validation: [references/boundary-validation.md](references/boundary-validation.md)
- Typed errors: [references/typed-errors.md](references/typed-errors.md)
- Module ownership: [references/module-boundaries.md](references/module-boundaries.md)

## Execution Rules

- Parse and validate once at the boundary; keep core logic typed and explicit.
- Keep domain crates pure. Push filesystem, environment, and process reads to adapters and composition roots.
- Keep `lib.rs` surfaces narrow; import exact modules instead of growing barrel layers.
- Treat unexpected branches as typed failures with context, not silent fallbacks.
- Prefer a small dedicated type or error module per concept when that improves ownership and clarity.
- Keep the response focused on findings, code changes, updated tests, and validation status.

## Validation

Use the standard Rust gate order for the relevant scope:

- cargo fmt --all --check
- cargo clippy --workspace --all-targets --locked -- -D warnings
- cargo test --workspace --locked
