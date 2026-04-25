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

- download of the validated wheels and sdist produced by the successful tag CI run
- promotion of artifacts that were built in an unprivileged CI job and attested in a separate tag-only CI job
- verification of the CI-generated artifact attestations
- artifact content policy validation via `tq-release`
- package metadata validation (`twine check dist/*`)
- smoke checks against the validated Linux wheel and sdist entrypoints
- fixture smoke validation with the validated Linux wheel
- trusted publish with `uv publish` on tag-triggered runs
- post-publish smoke validation via `uvx --from tqlint tq`
- consumer provenance verification against the PyPI Linux wheel
- GitHub release upload for wheels, sdist, and checksums

Dry-run validation happens in local release checks and in the tag-triggered CI build path before the publish workflow is allowed to promote artifacts.

Publishing runs in the `pypi` GitHub Actions environment. This environment must be configured with required reviewers for manual approval before publish runs.

For PR-time release classification, use the checklist in [versioning.md](./versioning.md). Maintainers should ensure each non-draft PR has exactly one `release:none`, `release:patch`, or `release:minor` label before merge. Bot-authored PRs follow the same rule; reviewers apply the label during review.

If the release-intent CI job fails, read the reported signal literally:

- missing or multiple `release:*` labels means fix the PR labels first
- `shipped runtime source changes` or `contract policy or reference doc changes` means re-check the checklist in [versioning.md](./versioning.md) and choose `release:patch` or `release:minor` if the shipped contract or behavior changed
- `shipped runtime dependency change` means a runtime manifest or lockfile change affected the published CLI path and `release:none` is not valid
- missing version or changelog metadata on `release:patch` or `release:minor` means prepare both in the same PR before merge

Resolve mismatches by either correcting the label or removing the release-relevant change from the PR.

## Maintainer checklist

1. Ensure `CHANGELOG.md` and version are ready.
2. Run the local validation commands:
   - `cargo fmt --all --check`
   - `cargo clippy --workspace --all-targets --locked -- -D warnings`
   - `cargo test --workspace --locked`
   - `cargo run -p tq-docsgen --locked -- generate all`
   - `cargo run -p tq-release --locked -- verify-release-policy --repo-root .`
   - `cargo package --workspace --locked`
   - `mise run release-build`
3. Create and push a signed release tag.
4. Confirm the tag-triggered CI run completes, including the tag-only artifact attestation job.
5. Approve the pending `pypi` environment deployment in GitHub Actions.
6. Confirm publish workflow success.
7. Verify install paths in a clean environment:
   - `uvx --from tqlint tq --help`
   - `uvx --from tqlint tq check --help`
   - `uv tool install tqlint && tq --help`

`mise run release-build` validates the source distribution plus a host-platform wheel. The full publishable artifact set is built in CI as Linux x86_64, macOS x86_64, macOS arm64, Windows x86_64 wheels, and the source distribution.

## Rollback guidance

- If publish fails before upload, fix workflow and re-run.
- If a bad version is published, publish a corrected patch release.
- Avoid deleting artifacts once consumed; prefer forward fix releases.

## Versioning when tag-triggered publish fails

- Prefer cutting a new patch version tag after fixing workflow issues (for example, `0.4.0` failed before upload → release `0.4.1`).
- Update `CHANGELOG.md` for the new version before tagging.
- Avoid reusing or force-moving existing release tags unless you are intentionally rewriting release history.
