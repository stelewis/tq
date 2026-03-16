# Local Workflows

Core contributor commands for day-to-day work.

## Workspace loop

- `cargo check --workspace --all-targets --locked`
- `cargo run -p tq-cli --locked -- check --help`

Use `cargo check` as the fast compile, type, trait, and borrow-check loop before running stricter gates.

## Quality gates

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `cargo build --workspace --locked`
- `cargo build -p tq-cli --release --locked`
- `cargo package --workspace --locked`
- `uv build`

## Combined local check

- `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked`

## Security and dependency audit

- `cargo audit`
- `cargo deny check`
- `cargo outdated --workspace --root-deps-only`

Secret scanning and commit policy remain part of the standard workflow through `gitleaks`, `detect-secrets`, and `commitizen`.

## Pre-commit hooks

The language-specific pre-commit hooks are Rust-native:

- `cargo fmt --all` on `pre-commit`
- `cargo clippy --workspace --all-targets --locked -- -D warnings` on `pre-push`
- `cargo test --workspace --locked` on `pre-push`
