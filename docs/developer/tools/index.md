# Developer Tools

Developer tooling and commands used to work on `tq`.

## Status

Phase 7 (CI and quality hook migration) is complete. The canonical local toolchain is now Rust-first for formatting, linting, testing, build validation, and dependency security checks.

Python remains in the repository only for transition-era conformance, docs generation, and release helpers until later rewrite phases remove those paths.

The product MSRV is Rust 1.94.0. CI and local quality gates should use the workspace toolchain by default unless a workflow explicitly documents a different toolchain boundary.

## Workspace and toolchain

- `cargo --version`
- `cargo build --workspace`
- `cargo run -p tq-cli -- --help`

## Fast local loop

- `cargo check --workspace --all-targets --locked`

Use this as the quick compile, type, trait, and borrow-check pass during active development. It is the closest Rust-native equivalent to a fast static analysis loop before running the stricter `clippy` and `test` gates.

## Quality gates

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets --locked`
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

## Security toolchain policy

Security scanners are intentionally treated as CI tooling, not as part of the `tq` runtime contract.

The workspace and product commands run on the pinned MSRV from `rust-toolchain.toml`. The scanner bootstrap in CI installs and uses the stable toolchain for `cargo-audit` and `cargo-deny`.

The setup action force-reinstalls both scanners from the pinned sources so cache hits cannot silently keep an older binary than the reviewed pin.

This split exists for two reasons:

- security tools and the RustSec advisory database can move faster than the product MSRV
- we do not want to raise the product MSRV only to satisfy scanner installation or parser support churn

Manual review is required for the scanner installation pins in `.github/actions/setup-rust-security-tools/action.yml`. Dependabot updates the Rust workspace and toolchain, but it does not update versions embedded in shell bootstrap logic.

## Conformance harness

- `cargo test -p tq-cli --test conformance_harness --locked -- --ignored --nocapture`

The conformance harness runs fixture projects through both runtimes, enforces deterministic repeated output, and prints a parity report that distinguishes exact matches from documented intentional deltas.

If the baseline Python executable is not available at `.venv/bin/python`, set `TQ_CONFORMANCE_PYTHON` to the interpreter that should run `python -m tq.cli.main`.

## Transitional docs workflow

- `uv run python scripts/docs/generate_rules_docs.py`
- `uv run python scripts/docs/generate_cli_docs.py`
- `uv run python scripts/docs/generate_config_examples.py`
- `mise run docs-build`

These commands remain temporary until Phase 8 ports docs generation into `tq-docsgen`.

## Pre-commit hooks

The pre-commit surface keeps hygiene, secret scanning, and commit policy hooks, but the language-specific hooks are now Rust-native:

- `cargo fmt --all` on `pre-commit`
- `cargo clippy --workspace --all-targets --locked -- -D warnings` on `pre-push`
- `cargo test --workspace --locked` on `pre-push`

The cargo hooks are local on purpose. There is no canonical first-party pre-commit hook set for `cargo fmt`, `cargo clippy`, or `cargo test`, and local hooks keep the pre-commit behavior aligned with the exact commands enforced in CI.

## Governance

- Keep this page aligned with CI workflows and pre-commit hooks.
- Remove legacy Python runtime commands once Rust cutover completes.
