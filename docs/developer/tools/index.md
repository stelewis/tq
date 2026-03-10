# Developer Tools

Developer tooling and commands used to work on `tq`.

## Status

The Rust rewrite plan introduces and standardizes this tooling surface in phases.

<!-- TODO – AFTER RUST REWRITE COMPLETION: REMOVE ALL EPHEMERAL WORDING E.G. PHASE 1, ETC. -->

Phase 1 (workspace bootstrap) is complete. The commands below are now canonical for Rust workspace bootstrap and baseline quality checks.

## Phase 1 commands

### Workspace and toolchain

- `cargo --version`
- `cargo build --workspace`
- `cargo run -p tq-cli -- --help`

### Quality gates

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

### Combined local check

- `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

## Future phases

As additional phases land, this page will be extended with:

- docs generation commands,
- release verification commands,
- security and dependency audit commands.

### Conformance harness commands

- `cargo test -p tq-cli --test conformance_harness -- --ignored --nocapture`

The conformance harness runs fixture projects through both runtimes, enforces deterministic repeated output, and prints a parity report that distinguishes exact matches from documented intentional deltas.

If the baseline Python executable is not available at `.venv/bin/python`, set `TQ_CONFORMANCE_PYTHON` to the interpreter that should run `python -m tq.cli.main`.

## Governance

- Keep this page aligned with CI workflows and pre-commit hooks.
- Remove legacy Python runtime commands once Rust cutover completes.
