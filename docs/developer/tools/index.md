# Developer Tools

Contributor tooling and automation.

## Scope

The project toolchain is Rust-first for build, lint, test, docs generation, and release verification.

Distribution is through PyPI: the package name is `tqlint` and the installed command is `tq`.

Artifacts are built from the workspace CLI crate through `maturin`.

The product MSRV is Rust 1.96. Local and CI commands should use the pinned workspace toolchain unless a workflow explicitly documents a different bootstrap boundary.

## Guides

- [Developer harness](./dev-harness.md)
- [Local workflows](./local-workflows.md)
- [Docs and release tooling](./docs-and-release.md)
- [CI and automation](./ci.md)
- [Pin maintenance](./pin-maintenance.md)

## Core commands

- `cargo check --workspace --all-targets --locked`
- `cargo dev check routine --repo-root .`
- `cargo dev check all --repo-root .`
- `cargo dev check --profile full all --repo-root .`
- `cargo audit -D warnings`
- `cargo deny check`
- `cargo update --dry-run`
- `cargo dev release verify-artifacts --dist-dir dist --profile <expected-profile>`

## Governance

- Keep this entrypoint aligned with CI workflows, pre-commit hooks, and publish automation.
- Security posture is defined in [Security standards](../standards/security.md).
- Dependency admission and package trust requirements are defined in [Supply-chain security standards](../standards/supply-chain-security.md).
- Keep detailed workflows in the linked pages above instead of re-expanding them here.
