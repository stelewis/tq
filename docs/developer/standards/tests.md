# Testing Standards

Treat the test suite as a first-class part of the codebase.

Keep tests:

- **Discoverable**: easy to find the test for a module.
- **Focused**: small surface area and one clear reason to fail.
- **Actionable**: failures point to one contract, not "the system".
- **Deterministic**: no dependency on timing, network, or incidental ordering.
- **Maintainable**: tests refactor with the code.

## Test Layers

### Inline unit tests

- Put narrow invariant tests in the owning module with `#[cfg(test)]` when they need private access or validate local helper behavior.
- Keep these tests close to the implementation and avoid turning them into mini integration suites.

### Crate contract tests

- Put public-surface, composition, and contract tests in `crates/<crate>/tests/`.
- Name each file for the contract it owns, such as `loader_contract.rs`, `reporting_contract.rs`, or `mapping_missing_test_rule_tests.rs`.
- Prefer one contract per test module. If a suite grows too large, split it by stable concern rather than building a grab-bag file.

### Cross-crate workflows

- Put end-to-end or composition-root workflows in the crate that owns that workflow, usually `tq-cli`, `tq-docsgen`, or `tq-release`.
- Do not create repository-root test grab bags that bypass crate ownership.

## Structure Rules

- Test the narrowest owned contract.
- Use inline tests for private invariants and crate tests for public behavior.
- Do not force 1:1 source-file mirroring when it fights Rust module boundaries or visibility.
- Keep helper code local to the owning suite. Share helpers only when the contract is genuinely reused, and prefer `tests/support.rs` or a small local helper module.
- Prefer temporary directories and constructed inputs over large checked-in fixtures.
- Add checked-in fixtures only when they encode a stable external contract, and keep them small and reviewable.
- Golden or snapshot tests are allowed for correctness-critical output, but the asserted surface should stay small and intentional.
- Use `expect` messages in test setup when they clarify the invariant being established.

## Test Quality Standards

### Avoid these anti-patterns

- **Monolithic contract suites**: one file testing many unrelated behaviors.
- **Duplicated coverage**: asserting the same contract in multiple layers without a reason.
- **Incidental assertions**: locking tests to private implementation details when a public contract would suffice.
- **Nondeterministic expectations**: relying on hash iteration, filesystem traversal order, or wall-clock behavior.
- **Opaque snapshots**: asserting giant blobs when a narrower contract would be clearer.
- **Fixture drift**: helpers or checked-in data that outlive the contract they were meant to validate.
- **Cross-crate testing by convenience**: testing a lower-level crate indirectly from a higher-level suite when the lower-level crate can own the contract directly.

Use this practical refactor rule:

- If a source contract splits, split or rewrite the tests so each file still owns one coherent contract. Do not keep historical grab-bag suites just because they already exist.

## Determinism Rules

- Tests must run fully offline.
- Do not depend on wall-clock time, random ordering, or machine-local state.
- If ordering matters, make it explicit in the implementation and assert it directly.
- Prefer `tempfile` workspaces and explicit fixture construction over shared mutable test directories.

## Workflow

- Full suite: `cargo test --workspace --locked`
- Fast crate loop: `cargo test -p <crate> --locked`
- Targeted contract test: `cargo test -p tq-rules --test mapping_missing_test_rule_tests --locked`

Run the full workspace suite before merge. Add narrower commands to your local loop, but do not treat a crate-only pass as a substitute for the workspace gate when shared contracts changed.
