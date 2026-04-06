# Release Workflow

Release workflow for publishing `tq` to PyPI.

## Package identity

- Repository and import package name is `tq`.
- Published distribution name is `tqlint`.
- CLI command exposed by the package is `tq`.

## User install and run commands

- Project dependency: `uv add --dev tqlint` then `uv run tq check`
- Ephemeral execution: `uvx --from tqlint tq check`
- Global tool: `uv tool install tqlint` then `tq check`

## Publish automation

Publishing is handled by the [publish workflow](https://github.com/stelewis/tq/blob/main/.github/workflows/publish.yml) on SemVer tags matching `<major>.<minor>.<patch>`.

The workflow performs:

- download of the validated wheel and sdist produced by the successful tag CI run
- promotion of artifacts that were built in an unprivileged CI job and attested in a separate tag-only CI job
- verification of the CI-generated artifact attestations
- artifact content policy validation via `tq-release`
- package metadata validation (`twine check dist/*`)
- smoke checks against the validated wheel and sdist entrypoints
- fixture smoke validation with the validated wheel
- trusted publish with `uv publish` on tag-triggered runs
- post-publish smoke validation via `uvx --from tqlint tq`
- consumer provenance verification against the PyPI wheel
- GitHub release upload for wheel, sdist, and checksums

Dry-run validation happens in local release checks and in the tag-triggered CI build path before the publish workflow is allowed to promote artifacts.

Publishing runs in the `pypi` GitHub Actions environment. This environment must be configured with required reviewers for manual approval before publish runs.

## Maintainer checklist

1. Ensure `CHANGELOG.md` and version are ready.
2. Run the local validation commands:
   - `cargo fmt --all --check`
   - `cargo clippy --workspace --all-targets --locked -- -D warnings`
   - `cargo test --workspace --locked`
   - `cargo run -p tq-docsgen --locked -- generate all`
   - `cargo run -p tq-release --locked -- verify-release-policy --repo-root .`
   - `cargo package --workspace --locked`
   - `uv build`
3. Create and push a signed release tag.
4. Confirm the tag-triggered CI run completes, including the tag-only artifact attestation job.
5. Approve the pending `pypi` environment deployment in GitHub Actions.
6. Confirm publish workflow success.
7. Verify install paths in a clean environment:
   - `uvx --from tqlint tq --help`
   - `uvx --from tqlint tq check --help`
   - `uv tool install tqlint && tq --help`

## Rollback guidance

- If publish fails before upload, fix workflow and re-run.
- If a bad version is published, publish a corrected patch release.
- Avoid deleting artifacts once consumed; prefer forward fix releases.

## Versioning when tag-triggered publish fails

- Prefer cutting a new patch version tag after fixing workflow issues (for example, `0.4.0` failed before upload → release `0.4.1`).
- Update `CHANGELOG.md` for the new version before tagging.
- Avoid reusing or force-moving existing release tags unless you are intentionally rewriting release history.
