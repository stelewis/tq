# Project Policies

Repository policies enforced by automation.

## GitHub Actions policy

### External actions must be SHA pinned

Policy file: [pinned actions policy](https://github.com/stelewis/tq/tree/main/.github/workflows/pinned-actions-policy.yml)

All external `uses:` references in workflows and composite actions must pin to a full 40-character commit SHA.

Allowed exceptions:

- local actions (`./...`)
- `docker://` references

Why this matters:

- protects workflows from mutable tag drift,
- supports reproducible and auditable CI behavior,
- reduces supply-chain risk.

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

## Release provenance policy

Tag-triggered release runs must:

- build `dist/*` artifacts,
- generate build provenance attestations,
- verify those attestations before publish,
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

- Release workflow: [publish workflow](https://github.com/stelewis/tq/tree/main/.github/workflows/publish.yml)
- Verification guide: [attestation verification](../attestation.md)

## Security disclosure policy

Policy file: [SECURITY.md](https://github.com/stelewis/tq/tree/main/SECURITY.md)

Potential vulnerabilities must be reported privately using GitHub private vulnerability reporting.
