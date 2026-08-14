# Pin Maintenance

Maintain frozen third-party refs as an explicit supply-chain workflow, not as incidental cleanup.

This guide covers:

- external GitHub Action refs in `.github/workflows/**` and `.github/actions/**`
- Docker-backed action image digests
- frozen pre-commit hook revs in `.pre-commit-config.yaml`
- Rust maintenance tool versions in `.github/actions/setup-rust-maintenance-tools/action.yml`
- actionlint and ShellCheck versions in `.github/dev-tools.toml`
- scheduled drift reporting for those pinned refs

## Enforcement and visibility

The repository uses four separate controls so frozen refs stay both strict and maintainable:

- [Pinned Actions Policy](https://github.com/stelewis/tq/blob/main/.github/workflows/pinned-actions-policy.yml) requires external actions to use full commit SHAs and Docker-backed actions to use SHA-256 image digests.
- [Frozen Pre-commit Policy](https://github.com/stelewis/tq/blob/main/.github/workflows/frozen-pre-commit-policy.yml) fails if an external pre-commit hook rev is not a full commit SHA.
- [Pinned External Dependency Drift](https://github.com/stelewis/tq/blob/main/.github/workflows/pinned-external-dependency-drift.yml) makes stale frozen refs visible when they lag the latest upstream SemVer release tag.
- [Rust Maintenance Tool Pins](https://github.com/stelewis/tq/blob/main/.github/workflows/rust-maintenance-tool-pins.yml) verifies pinned `cargo-audit` and `cargo-deny` release metadata and audits their published Cargo locks. Cargo validates each downloaded crates.io archive against the registry checksum; repository metadata is an admission signal, not proof that an archive matches upstream source.

Dependabot remains the default update path for supported GitHub Action and pre-commit dependencies. Use manual rotation for Docker image digests, urgent updates, drift responses, and any dependency that Dependabot cannot update completely.

Run `cargo update` for the Rust lockfile, then use `cargo dev deps update --dry-run --repo-root .` before applying the remaining repository-owned dependency and toolchain updates. Cargo launches the harness, so the harness does not nest a Cargo lockfile mutation that its parent process would overwrite.

Use `cargo dev deps audit-maintenance-tools --repo-root .` to run the scheduled Rust maintenance-tool subset locally. `cargo dev deps audit-latest --repo-root .` is the comprehensive freshness command for every pinned tool and project dependency ecosystem. Python is compared within its pinned minor series, Node within its pinned major series, and other tool pins against the latest stable SemVer release. The exact npm version in `packageManager` follows the npm release bundled with the pinned Node version.

Treat the currently pinned scanner releases as bootstrap trust. Before changing either scanner pin, use the existing trusted cargo-audit installation to audit the candidate release's published Cargo.lock, review its current crates.io owners and repository metadata, and only then update `.github/dev-tools.toml` and the setup-action default. CI repeats the metadata and lock checks after installation; it is verification of reviewed admission evidence, not a substitute for pre-rotation review.

Node and npm versions are owned by `package.json`. Actionlint and ShellCheck versions are owned by `.github/dev-tools.toml`; developers install those versions with their environment manager. When actionlint changes, resolve the official image tag to its registry digest and update `automation.actionlint-image.digest` in the same change. Pin policy derives the required versioned image reference and rejects a stale workflow consumer.

## GitHub Actions rotation

Review the upstream release first. Frozen SHAs are only useful when the release behind the SHA is acceptable.

1. Read the release notes and confirm the source repository still meets the repository trust bar.
2. Resolve the exact release tag to a commit SHA.
3. Update the `uses:` ref and the trailing version comment together.
4. If the change touches `.github/dependabot.yml`, keep the GitHub Actions coverage contract intact.
5. Let the pinned-actions and CI policy workflows validate the result.

Useful command pattern:

```bash
git ls-remote --tags "https://github.com/<owner>/<repo>.git"
```

Preferred edit shape:

```yaml
uses: owner/repo@0123456789abcdef0123456789abcdef01234567 # v1.2.3
```

Do not pin to a moving major tag such as `@v4` or a branch name. Keep the human-readable version comment so later reviews do not have to reverse-resolve the SHA by hand.

## Docker action rotation

Docker-backed actions must use an immutable SHA-256 image digest. For actionlint, update its version and image digest in `.github/dev-tools.toml`; pin policy derives the required `docker://<repository>:<version>@sha256:<digest>` workflow reference from those fields.

Resolve the digest from the trusted registry after reviewing the release and image provenance. Do not use mutable tags such as `latest`, a version tag without a digest, or a digest copied from an untrusted mirror.

## Pre-commit rotation

Use a frozen autoupdate flow so the file stays commit-pinned.

1. Update the hook revs with a frozen pre-commit autoupdate command.
2. Review the hook changes and upstream release notes.
3. Run the relevant hooks locally.
4. Let the frozen-pre-commit policy workflow validate that every external hook remains commit-pinned.

Useful command pattern:

```bash
uv run prek pre-commit autoupdate --freeze
uv run prek pre-commit run --all-files
```

Preferred edit shape:

```yaml
rev: 0123456789abcdef0123456789abcdef01234567  # frozen: v1.2.3
```

Do not replace the frozen SHA with a tag. The version comment is documentation only; the SHA is the actual control.

## Responding to drift issues

The scheduled external-pin drift workflow opens or refreshes a single tracking issue titled `chore: review frozen external pins` when it detects lagging action or pre-commit refs. The Rust maintenance-tool pin workflow uses the same issue lifecycle with the title `chore: review Rust maintenance tool pins`.

When that issue appears:

1. Prefer the existing Dependabot PR if it already covers the reported dependency.
2. Rotate remaining stale refs manually using the steps above.
3. Re-run or wait for the drift workflow after merge so it can close the issue automatically.

If the workflow cannot resolve an upstream SemVer release tag, or cannot derive a supported GitHub remote for a pre-commit repo entry, treat that as a manual review task. Either update the dependency source, or document why the upstream release surface does not fit the repository's frozen-pin maintenance model.

## Review checklist

- Was the source repository reviewed as a dependency admission decision, not just as a version bump?
- Do the current crates.io owners still match the reviewed project or organization?
- Is the new ref pinned to a full 40-character commit SHA?
- Does the human-readable version comment match the intended upstream release?
- If `.github/dependabot.yml` changed, does `cargo dev policy verify-dependabot --repo-root .` still pass?
- If `.pre-commit-config.yaml` changed, were the inline allowlist comments preserved on the frozen `rev:` lines?
