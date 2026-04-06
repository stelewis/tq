# Project Policies

Repository policies enforced by automation.

## GitHub Actions policy

### External actions must be SHA pinned

Policy file: [pinned actions policy](https://github.com/stelewis/tq/blob/main/.github/workflows/pinned-actions-policy.yml)

All external `uses:` references in workflows and composite actions must pin to a full 40-character commit SHA.

Allowed exceptions:

- local actions (`./...`)
- `docker://` references

Why this matters:

- protects workflows from mutable tag drift,
- supports reproducible and auditable CI behavior,
- reduces supply-chain risk.

## Pre-commit hook policy

### External pre-commit hooks must be SHA pinned

Policy file: [frozen pre-commit policy](https://github.com/stelewis/tq/blob/main/.github/workflows/frozen-pre-commit-policy.yml)

All external pre-commit hooks in `.pre-commit-config.yaml` must pin `rev:` to a full 40-character commit SHA.

Allowed exceptions:

- local hooks (`repo: local`)

Why this matters:

- protects local quality and secret-scanning automation from mutable tag drift,
- keeps hook execution reproducible across contributor machines and CI,
- aligns pre-commit hooks with the same supply-chain bar as GitHub Actions.

## Dependabot coverage policy

GitHub Actions dependency updates must cover both workflow files and local composite actions.

Required coverage:

- `directory: "/"` or `directories: ["/"]` to cover `.github/workflows`
- `directory: "/.github/actions/*"` or equivalent `directories` entry to cover local composite actions
- a single `github-actions` update block that owns the whole GitHub Actions surface

Enforcement:

- CI runs `cargo run -p tq-release --locked -- verify-dependabot --repo-root .`

Why this matters:

- prevents local actions from drifting outside automated dependency updates,
- keeps workflow and composite-action maintenance in one explicit policy surface,

## Frozen pin drift visibility

Policy file: [pinned external dependency drift](https://github.com/stelewis/tq/blob/main/.github/workflows/pinned-external-dependency-drift.yml)

Pinned GitHub Action refs and frozen pre-commit hook revs must be reviewed for upstream drift on a schedule, even when they are already commit-pinned.

Enforcement:

- the scheduled workflow writes a summary, opens or refreshes a tracking issue when drift is detected, and fails so stale pins stay visible.

Why this matters:

- commit pinning prevents mutable ref drift but does not keep versions current,
- scheduled review catches stale frozen refs that live outside lockfiles,
- a single tracking issue keeps maintenance visible without scattering ad hoc reminders.

## Rust advisory visibility

Policy file: [Rust security advisories](https://github.com/stelewis/tq/blob/main/.github/workflows/rust-security-advisories.yml)

Rust advisory and dependency-policy scans must run on a weekly schedule and on demand, with the main CI pipeline re-running them when Rust dependency or Rust security-policy surfaces change.

Enforcement:

- the scheduled workflow runs `cargo audit` and `cargo deny check` weekly and on demand.
- the main CI workflow reruns `cargo audit` and `cargo deny check` when `Cargo.lock`, any `Cargo.toml`, `deny.toml`, or the Rust security workflow and action definitions change.

Why this matters:

- new RustSec advisories can be published without any repository changes,
- path-gated PR and push checks preserve fast feedback on unrelated changes while still blocking risky dependency and policy edits,
- Dependabot and stale-version checks help with version drift but do not replace repo-owned advisory scanning,
- weekly review keeps lockfile risk visible between ordinary development activity.

## Docs dependency advisory visibility

Policy file: [Docs security workflow](https://github.com/stelewis/tq/blob/main/.github/workflows/docs-security.yml)

Docs toolchain advisory scans must run on a weekly schedule and on demand, with the main CI pipeline re-running them only when Node or docs-toolchain dependency surfaces change.

Enforcement:

- the scheduled workflow runs `npm audit --package-lock-only` weekly and on demand.
- the main CI workflow reruns `npm audit --package-lock-only` when `package.json`, `package-lock.json`, `mise.toml`, or the docs-toolchain workflow and action definitions change.

Why this matters:

- new npm advisories can be published without any repository changes,
- path-gated checks keep unrelated Rust and content PRs from being blocked by docs-only ecosystem churn,
- lockfile-only auditing keeps the docs dependency review surface explicit without paying for an unnecessary full install.

## Release provenance policy

Tag-triggered release runs must:

- promote only `dist/*` artifacts validated by the tag-triggered CI run,
- verify the CI-generated build provenance attestations before publish,
- re-run artifact content policy validation before publish,
- publish only after successful provenance verification.

Why this matters:

- establishes cryptographic provenance for released artifacts,
- gives maintainers and consumers a verifiable trust signal,
- aligns release validation with supply-chain security best practices.

## Packaging content policy

Release artifacts must only contain runtime content required for users.

Forbidden repository paths in wheel and sdist artifacts:

- `scripts/`
- `tests/`
- `docs/`
- `tmp/`
- `.github/`
- `.vscode/`

Enforcement:

- CI and publish checks run `cargo run -p tq-release --locked -- verify-artifact-contents --dist-dir dist`

Why this matters:

- keeps installed artifacts minimal and predictable,
- reduces accidental leakage of internal automation, editor metadata, or repository-only files,
- tightens release hygiene and consumer trust.

### References

- Release workflow: [publish workflow](https://github.com/stelewis/tq/blob/main/.github/workflows/publish.yml)
- Verification guide: [attestation verification](../attestation.md)

## Security disclosure policy

Policy file: [SECURITY.md](https://github.com/stelewis/tq/blob/main/SECURITY.md)

Potential vulnerabilities must be reported privately using GitHub private vulnerability reporting.

## Dependency trust posture

Dependency admission is treated as a supply-chain security decision, not a convenience decision.

Contributors must prefer mainstream, widely adopted, actively maintained packages with clear ownership and strong engineering discipline. New, obscure, weakly maintained, or otherwise low-trust packages require a documented exception and should normally be rejected.

Policy files: [Security](../security.md), [Supply-chain security standards](./supply-chain-security.md)
