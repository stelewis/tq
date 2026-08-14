# CI and Automation

CI and publish workflow contracts that contributors should keep in sync with local commands.

## CI jobs

The main CI workflow enforces:

- commit message policy via `commitizen`
- hygiene hooks via `pre-commit`
- formatting via `cargo fmt --all --check`
- Rust lint via `cargo clippy --workspace --all-targets --locked -- -D warnings`
- GitHub Actions syntax, expression, security, and shell validation via pinned `actionlint` with pinned ShellCheck integration
- repository automation semantics via `cargo dev policy verify-automation --repo-root .`
- advisory runtime dependency check via `cargo dev runtime-deps ...` only on pull requests when `Cargo.lock` or `Cargo.toml` change; reports whether the shipped CLI dependency graph changed without blocking the PR
- docs sync via `cargo run -p tq-docsgen --locked -- generate all` only when docs contract inputs, generated reference outputs, `crates/tq-docsgen/**`, `crates/tq-cli/**`, or `crates/tq-rules/**` change
- docs site build via `npm run docs:build` only when docs content, docs toolchain files, or docs generator inputs change
- tests via `cargo test --workspace --locked`
- release-policy validation via `cargo dev policy verify-release --repo-root .`
- build validation via `cargo build`, `uv build --sdist`, a release-wheel matrix for Linux x86_64, macOS x86_64, macOS arm64, and Windows x86_64, artifact policy verification, built artifact entrypoint smoke checks, and Linux wheel plus sdist compatibility smoke checks across Python 3.11 to 3.14
- secret scanning via `gitleaks` on every push and pull request, with GitHub secret scanning enabled on the repository
- Rust dependency security checks via `cargo audit` and `cargo deny` only when Rust dependency or Rust security-policy files change
- docs dependency security checks via `npm audit --package-lock-only` only when docs dependency or docs-toolchain files change

Separate policy workflows enforce frozen automation refs:

- external GitHub Action refs must be pinned to full commit SHAs
- external pre-commit hook revs must be pinned to full commit SHAs

Separate scheduled workflows handle dependency drift and security review outside the main PR and push pipeline:

- weekly Rust advisory and policy scanning via `.github/workflows/rust-security-advisories.yml`, `cargo audit -D warnings`, and `cargo deny check`
- weekly docs dependency auditing via `.github/workflows/docs-security.yml` and `npm audit --package-lock-only`
- centralized Rust maintenance tool pin, source, and packaged-lock validation for `cargo-audit` and `cargo-deny`
- frozen GitHub Action and pre-commit pin drift via `.github/workflows/pinned-external-dependency-drift.yml`

For manual rotation and drift response steps, see [Pin maintenance](./pin-maintenance.md).

## Security toolchain policy

Security scanners are treated as CI tooling, not as part of the `tq` runtime contract.

The workspace uses the pinned MSRV from `rust-toolchain.toml`. CI installs `cargo-audit` and `cargo-deny` on stable through `.github/actions/setup-rust-security-tools`, which delegates version pinning to `.github/actions/setup-rust-maintenance-tools/action.yml`, so scanner installation can move independently of the product toolchain. Cargo-audit scans every Cargo.lock entry with warnings denied; cargo-deny evaluates the resolved graph for advisories, bans, licenses, and source policy. Main CI reruns both scanners only when Rust dependency or Rust security-policy surfaces change; the scheduled Rust workflow covers advisory churn between repository changes.

The docs dependency audit uses `npm audit --package-lock-only` and only reruns in main CI when the Node or docs-toolchain surface changes. `package.json` is the sole owner of the Node toolchain: setup-node reads `engines.node` directly, validates that `packageManager` matches that release's bundled npm, and `health doctor` consumes the same fields. The scheduled docs security workflow covers advisory churn for the VitePress toolchain between repository changes. The docs sync and docs build jobs follow the same model: they are skipped unless docs content, docs generator inputs, generated reference outputs, or docs-toolchain files changed.

The `cargo dev change-scope` command owns CI path classification. Unknown changed paths emit a warning and enable every scoped gate, so classification drift cannot skip security or documentation checks. The independent `cargo dev policy verify-automation` gate fails when any tracked path remains unclassified, forcing the change-scope contract to be updated before merge.

The same automation policy discovers every workflow and job. It requires positive job timeouts, rejects workflow-global write permissions and `write-all`, centralizes npm dependency installation behind `npm ci --ignore-scripts`, verifies immutable external references, and validates active Dependabot coverage. New workflow files and jobs therefore enter the policy automatically instead of relying on a manually maintained test list.

Pinned `actionlint` complements these repository-specific rules with full GitHub workflow parsing, expression and context typing, action input/output checks, reusable-workflow validation, injection checks, and embedded shell analysis through ShellCheck. Neither layer substitutes for the other. Local orchestration belongs to `cargo dev check`, which verifies and invokes the pinned `actionlint` and ShellCheck executables from `PATH`. CI provisioning remains at the workflow boundary and uses actionlint's official image, which includes actionlint, ShellCheck, and pyflakes; the manifest-owned version and immutable image digest are stricter than the upstream download script, which does not verify the downloaded archive. Pin policy requires the workflow to consume the exact versioned digest. CI registers actionlint's version-pinned problem matcher so findings annotate affected workflow lines.

Dependabot covers manifest and lockfile updates. Local maintenance uses `cargo update --dry-run` when an immediate compatible-drift review is needed. This complements other policy checks: `cargo audit` scans the raw lock, `cargo deny` enforces graph-aware advisories, explicit bans, licenses, and source policy, and `npm audit --package-lock-only` covers the docs lockfile.

The maintenance-tool pin workflow covers the embedded versions in `.github/actions/setup-rust-maintenance-tools/action.yml` because those values are not lockfile entries or Dependabot-managed manifests. When drift is detected, the workflow writes a summary, opens or refreshes a tracking issue on scheduled runs, and fails so the review stays visible.

## Publish workflow

On SemVer tag pushes, the unprivileged CI build jobs validate and upload release-candidate wheel and sdist artifacts. A read-only tag job downloads and validates the full set, then a separate checkout-free job with OIDC permission attests those unchanged bytes and uploads the final `validated-dist` artifact for promotion.

The publish workflow runs after successful push-triggered CI runs and never checks out the triggering revision. It recognizes a release only when that exact source run emitted one `validated-dist` artifact, takes the release tag from the source-run event, and then requires both the repository's active, no-bypass immutable-tag ruleset and an exact tag-to-source-commit binding. A read-only job downloads the validated artifact set, verifies CI-generated attestations, and smoke-tests the Linux wheel and sdist. A separate environment-protected job downloads the same immutable run artifact, reverifies attestations, checks the tag-to-commit binding immediately before each irreversible publication operation, publishes with `uv publish`, and uploads release assets and checksums. A final read-only job verifies the published entrypoint and consumer-facing Linux wheel provenance after publish credentials are gone.

Workflows that already perform a trusted checkout use the repository-local `setup-python-uv` composite action. The checkout-free publish workflow cannot execute local actions, so it defines the same pinned external setup actions once with YAML anchors and reuses those definitions across jobs.
