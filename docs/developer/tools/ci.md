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
- secret scanning via `gitleaks` and `detect-secrets` on every push and pull request
- Rust dependency security checks via `cargo audit` and `cargo deny` only when Rust dependency or Rust security-policy files change
- docs dependency security checks via `npm audit --package-lock-only` only when docs dependency or docs-toolchain files change

Separate policy workflows enforce frozen automation refs:

- external GitHub Action refs must be pinned to full commit SHAs
- external pre-commit hook revs must be pinned to full commit SHAs

Separate scheduled workflows handle dependency drift and security review outside the main PR and push pipeline:

- weekly Rust advisory and policy scanning via `.github/workflows/rust-security-advisories.yml`, `cargo audit`, and `cargo deny check`
- weekly docs dependency auditing via `.github/workflows/docs-security.yml` and `npm audit --package-lock-only`
- direct workspace dependency drift via `cargo outdated --workspace --root-deps-only`
- centralized Rust maintenance tool pin drift in `.github/actions/setup-rust-maintenance-tools/action.yml` for `cargo-outdated`, `cargo-audit`, and `cargo-deny`
- frozen GitHub Action and pre-commit pin drift via `.github/workflows/pinned-external-dependency-drift.yml`

For manual rotation and drift response steps, see [Pin maintenance](./pin-maintenance.md).

## Security toolchain policy

Security scanners are treated as CI tooling, not as part of the `tq` runtime contract.

The workspace uses the pinned MSRV from `rust-toolchain.toml`. CI installs `cargo-audit` and `cargo-deny` on stable through `.github/actions/setup-rust-security-tools`, which delegates version pinning to `.github/actions/setup-rust-maintenance-tools/action.yml`, so scanner installation can move independently of the product toolchain. Main CI reruns those scanners only when Rust dependency or Rust security-policy surfaces change; the scheduled Rust workflow covers advisory churn between repository changes.

The docs dependency audit uses `npm audit --package-lock-only` and only reruns in main CI when the Node or docs-toolchain surface changes. The scheduled docs security workflow covers advisory churn for the VitePress toolchain between repository changes.

The stale dependency workflow installs `cargo-outdated` separately from the product toolchain and checks only root workspace dependencies. This complements Dependabot and other policy checks: `cargo audit` catches published advisories, `cargo deny` enforces explicit bans plus license and source policy, `npm audit --package-lock-only` covers the docs lockfile, and `cargo outdated` surfaces ordinary version drift.

The maintenance-tool pin workflow covers the embedded versions in `.github/actions/setup-rust-maintenance-tools/action.yml` because those values are not lockfile entries or Dependabot-managed manifests. When drift is detected, the workflow writes a summary, opens or refreshes a tracking issue on scheduled runs, and fails so the review stays visible.

## Publish workflow

On SemVer tag pushes, the unprivileged CI build job validates and uploads release-candidate wheel and sdist artifacts, then a separate tag-only CI job downloads those artifacts, generates provenance attestations, and uploads the final `validated-dist` artifact for promotion.

The publish workflow runs after that successful tag-triggered CI run, downloads the validated wheel and sdist artifacts from CI, verifies the CI-generated provenance attestations, re-runs artifact content policy validation with `tq-release`, smoke-tests the release artifacts, publishes to PyPI with `uv publish`, verifies the consumer-facing wheel, and uploads the release assets and checksums to the GitHub release for the SemVer tag.
