# Attestation verification

This document explains the steps for verifying release artifact provenance for `tq`.

## Scope

This page defines:

- maintainer verification in release automation,
- consumer verification before install,
- offline/air-gapped verification workflow.

## Maintainer verification in CI

The publish workflow verifies each built wheel and sdist attestation using:

- repository identity (`--repo stelewis/tq` via `${GITHUB_REPOSITORY}`),
- signer workflow identity (`--signer-workflow stelewis/tq/.github/workflows/publish.yml`),
- hosted-runner enforcement (`--deny-self-hosted-runners`).

Equivalent command shape:

```sh
gh attestation verify dist/tqlint-<version>-py3-none-any.whl \
  --repo stelewis/tq \
  --signer-workflow stelewis/tq/.github/workflows/publish.yml \
  --deny-self-hosted-runners
```

## Consumer verification before install

1. Download the exact artifact you plan to install.
2. Verify provenance before install.

Example:

```sh
python -m pip download --no-deps tqlint==<version>
gh attestation verify tqlint-<version>-py3-none-any.whl \
  --repo stelewis/tq \
  --signer-workflow stelewis/tq/.github/workflows/publish.yml \
  --deny-self-hosted-runners
```

Release automation runs an equivalent consumer check after publish by downloading from PyPI and verifying the wheel attestation before the workflow is marked green.

## Offline verification

For offline/air-gapped verification, use the GitHub CLI bundle workflow:

- `gh attestation download ... -R stelewis/tq`
- `gh attestation trusted-root > trusted_root.jsonl`
- `gh attestation verify ... --bundle <file>.jsonl --custom-trusted-root trusted_root.jsonl`

## Integrity model

`tq` release verification is provenance-first:

- Attestations are the primary trust signal because they bind artifact identity to repository/workflow identity and runner policy.
- `SHA256SUMS` is published with each GitHub Release as an integrity convenience for local mirroring and checksum validation.
- Detached signatures are not currently included for this project.
