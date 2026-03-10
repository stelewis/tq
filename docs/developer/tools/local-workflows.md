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
- `cargo metadata --format-version 1 --locked > /dev/null`

## Combined local check

- `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked`

## Security and dependency audit

- `cargo audit`
- `cargo deny check`

Secret scanning and commit policy remain part of the standard workflow through `gitleaks`, `detect-secrets`, and `commitizen`.

## Conformance harness

- `cargo test -p tq-cli --test conformance_harness --locked -- --ignored --nocapture`

The conformance harness compares Rust output with the transitional Python baseline and checks repeat-run determinism.

If the baseline interpreter is not available at `.venv/bin/python`, set `TQ_CONFORMANCE_PYTHON` to the executable that should run `python -m tq.cli.main`.

## Pre-commit hooks

The language-specific pre-commit hooks are Rust-native:

- `cargo fmt --all` on `pre-commit`
- `cargo clippy --workspace --all-targets --locked -- -D warnings` on `pre-push`
- `cargo test --workspace --locked` on `pre-push`
