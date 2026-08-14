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
- promotion of artifacts built and fully validated in unprivileged CI, then attested in a checkout-free tag-only job
- verification of the CI-generated artifact attestations
- checkout-free, read-only package metadata and artifact smoke validation
- a minimal privileged job that reverifies attestations and tag binding before trusted publishing
- trusted publish with `uv publish` on tag-triggered runs
- checkout-free post-publish smoke and consumer provenance verification without publish credentials
- GitHub release upload for wheels, sdist, and checksums

Dry-run validation happens in local release checks and in the tag-triggered CI build path before the publish workflow is allowed to promote artifacts.

The publish workflow never checks out or executes the triggering revision. Source-owned release policy, archive-content validation, and build smoke checks run in unprivileged CI before `validated-dist` is attested. The follow-on workflow separates read-only candidate execution, credentialed publication, and read-only consumer verification into distinct jobs.

Publishing runs in the `pypi` GitHub Actions environment. This environment must be configured with required reviewers for manual approval before publish runs.

Repository rules must also keep the active `Immutable tags` ruleset enabled for all tags, with no bypass actors and both update and deletion restrictions. The publish workflow verifies that remote policy before resolving a release candidate and rechecks the release tag binding immediately before PyPI publication and GitHub release creation.

## Maintainer checklist

1. Ensure `CHANGELOG.md` and version are ready.
2. Run the local validation commands:
   - `cargo dev check --profile full all --repo-root .`
3. Create and push a signed release tag.
4. Confirm the tag-triggered CI run completes, including the read-only full-set validation and checkout-free attestation jobs.
5. Approve the pending `pypi` environment deployment in GitHub Actions.
6. Confirm publish workflow success.
7. Verify install paths in a clean environment:
   - `uvx --from tqlint tq --help`
   - `uvx --from tqlint tq check --help`
   - `uv tool install tqlint && tq --help`

`cargo dev check --profile full all` validates the source distribution plus a host-platform wheel. The full publishable artifact set is built in CI as Linux x86_64, macOS x86_64, macOS arm64, Windows x86_64 wheels, and the source distribution.

## Rollback guidance

- If publish fails before upload, fix workflow and re-run.
- If a bad version is published, publish a corrected patch release.
- Release tags and published artifacts are immutable; recover with a forward fix release.

## Versioning when tag-triggered publish fails

- Prefer cutting a new patch version tag after fixing workflow issues (for example, `0.4.0` failed before upload → release `0.4.1`).
- Update `CHANGELOG.md` for the new version before tagging.
- Avoid reusing or force-moving existing release tags unless you are intentionally rewriting release history.
