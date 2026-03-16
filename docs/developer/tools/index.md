# Developer Tools

Contributor tooling and automation.

## Scope

The project toolchain is Rust-first for build, lint, test, docs generation, and release verification.

Distribution is through PyPI: the package name is `tqlint` and the installed command is `tq`.

Artifacts are built from the workspace CLI crate through `maturin`.

The product MSRV is Rust 1.94.0. Local and CI commands should use the pinned workspace toolchain unless a workflow explicitly documents a different bootstrap boundary.

## Guides

- [Local workflows](./local-workflows.md)
- [Docs and release tooling](./docs-and-release.md)
- [CI and automation](./ci.md)

## Core commands

- `cargo check --workspace --all-targets --locked`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `cargo audit`
- `cargo deny check`
- `cargo outdated --workspace --root-deps-only`
- `cargo run -p tq-docsgen --locked -- generate all`
- `cargo run -p tq-release --locked -- verify-artifact-contents --dist-dir dist`

## Governance

- Keep this entrypoint aligned with CI workflows, pre-commit hooks, and publish automation.
- Keep detailed workflows in the linked pages above instead of re-expanding them here.
