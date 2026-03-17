# Pin Maintenance

Maintain frozen third-party refs as an explicit supply-chain workflow, not as incidental cleanup.

This guide covers:

- external GitHub Action refs in `.github/workflows/**` and `.github/actions/**`
- frozen pre-commit hook revs in `.pre-commit-config.yaml`
- scheduled drift reporting for those pinned refs

## Enforcement and visibility

The repository uses three separate controls so frozen refs stay both strict and maintainable:

- [Pinned Actions Policy](https://github.com/stelewis/tq/blob/main/.github/workflows/pinned-actions-policy.yml) fails if an external `uses:` ref is not pinned to a full commit SHA.
- [Frozen Pre-commit Policy](https://github.com/stelewis/tq/blob/main/.github/workflows/frozen-pre-commit-policy.yml) fails if an external pre-commit hook rev is not a full commit SHA.
- [Pinned External Dependency Drift](https://github.com/stelewis/tq/blob/main/.github/workflows/pinned-external-dependency-drift.yml) makes stale frozen refs visible when they lag the latest upstream SemVer release tag.

Dependabot remains the default update path for both surfaces. Use manual rotation when you need an urgent update, when you are responding to a drift issue, or when a Dependabot PR needs a manual follow-up.

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

## Pre-commit rotation

Use a frozen autoupdate flow so the file stays commit-pinned.

1. Update the hook revs with a frozen pre-commit autoupdate command.
2. Review the hook changes and upstream release notes.
3. Preserve the inline `# pragma: allowlist secret` comments on the exact frozen `rev:` lines.
4. Run the relevant hooks locally.
5. Let the frozen-pre-commit policy workflow validate that every external hook remains commit-pinned.

Useful command pattern:

```bash
uv run prek pre-commit autoupdate --freeze
uv run prek pre-commit run --all-files
```

Preferred edit shape:

```yaml
rev: 0123456789abcdef0123456789abcdef01234567  # frozen: v1.2.3  # pragma: allowlist secret
```

Do not replace the frozen SHA with a tag. The version comment is documentation only; the SHA is the actual control.

## Responding to drift issues

The scheduled drift workflow opens or refreshes a single tracking issue titled `chore: review frozen external pins` when it detects lagging action or pre-commit refs.

When that issue appears:

1. Prefer the existing Dependabot PR if it already covers the reported dependency.
2. Rotate remaining stale refs manually using the steps above.
3. Re-run or wait for the drift workflow after merge so it can close the issue automatically.

If the workflow cannot resolve an upstream SemVer release tag, treat that as a manual review task. Either update the dependency source, or document why the upstream release surface does not fit the repository's frozen-pin maintenance model.

## Review checklist

- Was the source repository reviewed as a dependency admission decision, not just as a version bump?
- Is the new ref pinned to a full 40-character commit SHA?
- Does the human-readable version comment match the intended upstream release?
- If `.github/dependabot.yml` changed, does `cargo run -p tq-release --locked -- verify-dependabot --repo-root .` still pass?
- If `.pre-commit-config.yaml` changed, were the inline allowlist comments preserved on the frozen `rev:` lines?
