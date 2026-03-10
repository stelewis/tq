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

Enforcement:

- CI and publish checks run `cargo run -p tq-release --locked -- verify-artifact-contents --dist-dir dist`

Why this matters:

- keeps installed artifacts minimal and predictable,
- reduces accidental leakage of internal automation or repository-only files,
- tightens release hygiene and consumer trust.

### References

- Release workflow: [publish workflow](https://github.com/stelewis/tq/tree/main/.github/workflows/publish.yml)
- Verification guide: [attestation verification](../attestation.md)

## Security disclosure policy

Policy file: [SECURITY.md](https://github.com/stelewis/tq/tree/main/SECURITY.md)

Potential vulnerabilities must be reported privately using GitHub private vulnerability reporting.
