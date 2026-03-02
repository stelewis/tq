# Project policies

This page is the canonical reference for repository policies enforced by automation.

## GitHub Actions policy

### External actions must be SHA pinned

Policy file: [pinned actions policy](https://github.com/stelewis/tq/blob/main/.github/workflows/pinned-actions-policy.yml)

All external `uses:` references in workflows and composite actions must pin to a full 40-character commit SHA.

Allowed exceptions:

- local actions (`./...`)
- `docker://` references

Why:

- protects workflows from mutable tag drift,
- supports reproducible and auditable CI behavior,
- reduces supply-chain risk.

## Release provenance policy

Tag-triggered release runs must:

- build `dist/*` artifacts,
- generate build provenance attestations,
- verify those attestations before publish,
- publish only after successful provenance verification.

Why:

- establishes cryptographic provenance for released artifacts,
- gives maintainers and consumers a verifiable trust signal,
- aligns release validation with supply-chain security best practices.

### References

- Release workflow: [publish workflow](https://github.com/stelewis/tq/blob/main/.github/workflows/publish.yml)
- Verification guide: [attestation verification](../tools/attestation_verification.md)

## Security disclosure policy

Policy file: [security policy](https://github.com/stelewis/tq/blob/main/SECURITY.md)

Potential vulnerabilities must be reported privately using GitHub private vulnerability reporting.
