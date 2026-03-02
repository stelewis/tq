# tq release workflow

This document defines the release workflow for publishing `tq` to PyPI.

## Package identity

- Repository and import package remain `tq`.
- Published distribution name is `tqlint`.
- CLI commands exposed by the package are `tq` and `tqlint`.

## User install and run commands

- Project dependency: `uv add --dev tqlint` then `uv run tq check`
- Ephemeral execution: `uvx tqlint check`
- Global tool: `uv tool install tqlint` then `tq check`

## Publish automation

Publishing is handled by [publish workflow](../../../.github/workflows/publish.yml) on SemVer tags matching `<major>.<minor>.<patch>` (for example `0.4.0`).

The workflow performs:

- `uv build`
- package metadata validation (`twine check dist/*`)
- smoke checks against built wheel and sdist
- GitHub artifact attestation for `dist/*` (supply-chain provenance)
- trusted publish with `uv publish` (tag-triggered runs)
- post-publish smoke checks via `uvx tqlint`

Manual `workflow_dispatch` runs are supported for dry-run build/smoke validation without publishing to PyPI.

Publishing runs in the `pypi` GitHub Actions environment. Configure that environment with required reviewers for manual approval before publish runs.

## Maintainer checklist

1. Ensure `CHANGELOG.md` and version are ready.
2. Run quality gates locally:
   - `uv run ruff format`
   - `uv run ruff check --fix`
   - `uv run ty check`
   - `uv run tq check`
   - `uv run pytest -q`
3. Create and push a signed release tag (for example `0.3.1`).
4. Approve the pending `pypi` environment deployment in GitHub Actions.
5. Confirm publish workflow success.
6. Verify install paths in a clean environment:
   - `uvx tqlint --help`
   - `uvx tqlint check --help`

## Rollback guidance

- If publish fails before upload, fix workflow and re-run.
- If a bad version is published, publish a corrected patch release.
- Avoid deleting artifacts once consumed; prefer forward fix releases.
