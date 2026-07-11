# Local Workflows

Core contributor commands for day-to-day work.

## Workspace loop

- `cargo check --workspace --all-targets --locked`
- `cargo run -p tq-cli --locked -- check --help`

Use `cargo check` as the fast compile, type, trait, and borrow-check loop before running stricter gates.

## Quality gates

- `cargo dev check routine --repo-root .`
- `cargo dev check all --repo-root .`
- `cargo dev check --profile full all --repo-root .`

`cargo dev check routine` runs the fast daily Rust gate: format, clippy, and tests. `cargo dev check all` adds generated-doc sync and release-policy validation. `cargo dev check --profile full all` adds the local release artifact build. The release build validates the source distribution plus a wheel for the current host platform; the full publishable artifact matrix is validated in CI.

## Combined local check

- `cargo dev check routine --repo-root .`

## Security and dependency audit

- `cargo audit`
- `cargo deny check`
- `cargo outdated --workspace --root-deps-only`

Secret scanning and commit policy remain part of the standard workflow through `gitleaks`, `detect-secrets`, and `commitizen`.

These checks are only the baseline. Before adding or upgrading external dependencies, follow the broader [Security standards](../standards/security.md) guidance and the dependency review bar in [Supply-chain security standards](../standards/supply-chain-security.md).

## Pre-commit hooks

The language-specific pre-commit hooks are Rust-native:

- `cargo fmt --all` on `pre-commit`
- `cargo clippy --workspace --all-targets --locked -- -D warnings` on `pre-push`
- `cargo test --workspace --locked` on `pre-push`
