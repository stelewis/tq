# CI and Automation

CI and publish workflow contracts that contributors should keep in sync with local commands.

## CI jobs

The main CI workflow enforces:

- commit message policy via `commitizen`
- hygiene hooks via `pre-commit`
- formatting via `cargo fmt --all --check`
- lint via `cargo clippy --workspace --all-targets --locked -- -D warnings`
- docs sync via `cargo run -p tq-docsgen --locked -- generate all`
- docs site build via `mise run docs-build`
- tests via `cargo test --workspace --locked`
- repository policy validation via `cargo run -p tq-release --locked -- verify-dependabot --repo-root .`
- build validation via `cargo build`, `cargo package --workspace --locked`, `uv build`, artifact policy verification, and built wheel/sdist entrypoint smoke checks
- security checks via `cargo audit`, `cargo deny`, `gitleaks`, and `detect-secrets`

A separate scheduled workflow checks for stale direct workspace dependencies with `cargo-outdated` so maintainers can see version drift even when no security advisory exists.

## Security toolchain policy

Security scanners are treated as CI tooling, not as part of the `tq` runtime contract.

The workspace uses the pinned MSRV from `rust-toolchain.toml`. CI installs `cargo-audit` and `cargo-deny` on stable through `.github/actions/setup-rust-security-tools` so scanner installation can move independently of the product MSRV.

The stale dependency workflow installs `cargo-outdated` separately from the product toolchain and checks only root workspace dependencies. This complements Dependabot and policy checks: `cargo audit` catches published advisories, `cargo deny` enforces explicit bans and license/source policy, and `cargo outdated` surfaces plain version drift.

## Publish workflow

The publish workflow validates `cargo package --workspace --locked`, builds wheel and sdist artifacts with `uv build`, verifies artifact content policy with `tq-release`, smoke-tests the published entrypoints, generates provenance attestations, publishes to PyPI with `uv publish`, verifies the consumer-facing PyPI wheel, and uploads the release assets plus checksums to the GitHub release for the pushed SemVer tag.
