# Release Workflow

Release workflow for publishing `tq` to PyPI.

## Package identity

- Repository and import package name is `tq`.
- Published distribution name is `tqlint`.
- CLI commands exposed by the package are `tq` and `tqlint`.

## User install and run commands

- Project dependency: `uv add --dev tqlint` then `uv run tq check`
- Ephemeral execution: `uvx tqlint check`
- Global tool: `uv tool install tqlint` then `tq check`

## Publish automation

Publishing is handled by [publish workflow](../../.github/workflows/publish.yml) on SemVer tags matching `<major>.<minor>.<patch>`.

The workflow performs:

- `uv build`
- artifact content policy validation via `tq-release`
- package metadata validation (`twine check dist/*`)
- smoke checks against built wheel and sdist entrypoints
- fixture smoke validation with the built wheel
- GitHub artifact attestation generation for wheel and sdist
- attestation verification before publish
- trusted publish with `uv publish` on tag-triggered runs
- post-publish smoke validation via `uvx tqlint`
- consumer provenance verification against the PyPI wheel
- GitHub release upload for wheel, sdist, and checksums

Manual `workflow_dispatch` runs support dry-run build and smoke validation without publishing to PyPI.

## Run a dry-run release validation

1. Open **GitHub → Actions → Publish**.
2. Click **Run workflow**.
3. Select branch `main` (or your release branch).
4. Run it.

Expected behavior for `workflow_dispatch` runs:

- build and metadata validation run
- wheel and sdist smoke checks run
- fixture smoke checks run
- `Publish` step is skipped
- post-publish steps are skipped

Publishing runs in the `pypi` GitHub Actions environment. This environment must be configured with required reviewers for manual approval before publish runs.

## Maintainer checklist

1. Ensure `CHANGELOG.md` and version are ready.
2. Run quality gates locally:
   - `cargo fmt --all --check`
   - `cargo clippy --workspace --all-targets --locked -- -D warnings`
   - `cargo test --workspace --locked`
   - `cargo run -p tq-docsgen --locked -- generate all`
   - `uv build`
3. Create and push a signed release tag.
4. Approve the pending `pypi` environment deployment in GitHub Actions.
5. Confirm publish workflow success.
6. Verify install paths in a clean environment:
   - `uvx tqlint --help`
   - `uvx tqlint check --help`

## Rollback guidance

- If publish fails before upload, fix workflow and re-run.
- If a bad version is published, publish a corrected patch release.
- Avoid deleting artifacts once consumed; prefer forward fix releases.

## Versioning when tag-triggered publish fails

- Prefer cutting a new patch version tag after fixing workflow issues (for example, `0.4.0` failed before upload → release `0.4.1`).
- Update `CHANGELOG.md` for the new version before tagging.
- Avoid reusing or force-moving existing release tags unless you are intentionally rewriting release history.
