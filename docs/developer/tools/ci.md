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

Separate policy workflows enforce frozen automation refs:

- external GitHub Action refs must be pinned to full commit SHAs
- external pre-commit hook revs must be pinned to full commit SHAs

Separate scheduled workflows handle dependency drift outside the main PR and push pipeline:

- direct workspace dependency drift via `cargo outdated --workspace --root-deps-only`
- centralized Rust maintenance tool pin drift in `.github/actions/setup-rust-maintenance-tools/action.yml` for `cargo-outdated`, `cargo-audit`, and `cargo-deny`
- frozen GitHub Action and pre-commit pin drift via `.github/workflows/pinned-external-dependency-drift.yml`

For manual rotation and drift response steps, see [Pin maintenance](./pin-maintenance.md).

## Security toolchain policy

Security scanners are treated as CI tooling, not as part of the `tq` runtime contract.

The workspace uses the pinned MSRV from `rust-toolchain.toml`. CI installs `cargo-audit` and `cargo-deny` on stable through `.github/actions/setup-rust-security-tools`, which delegates version pinning to `.github/actions/setup-rust-maintenance-tools/action.yml`, so scanner installation can move independently of the product toolchain.

The stale dependency workflow installs `cargo-outdated` separately from the product toolchain and checks only root workspace dependencies. This complements Dependabot and other policy checks: `cargo audit` catches published advisories, `cargo deny` enforces explicit bans plus license and source policy, and `cargo outdated` surfaces ordinary version drift.

The maintenance-tool pin workflow covers the embedded versions in `.github/actions/setup-rust-maintenance-tools/action.yml` because those values are not lockfile entries or Dependabot-managed manifests. When drift is detected, the workflow writes a summary, opens or refreshes a tracking issue on scheduled runs, and fails so the review stays visible.

## Publish workflow

The publish workflow validates `cargo package --workspace --locked`, builds wheel and sdist artifacts with `uv build`, verifies artifact content policy with `tq-release`, smoke-tests the published entrypoints, generates provenance attestations, publishes to PyPI with `uv publish`, verifies the consumer-facing wheel, and uploads the release assets and checksums to the GitHub release for the pushed SemVer tag.
