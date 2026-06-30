---
name: release-decision
description: Classify a change's release impact and the Conventional Commit type that carries it
argument-hint: Provide a PR number, commit range, base/head refs, or a summary of the change to classify
agent: agent
---

Classify how a change affects the published `tq` release and which Conventional Commit type should carry it.

Release intent in this repository comes from Conventional Commit types. `cz check` enforces commit format and `cz bump` derives the next version and `CHANGELOG.md` from commit history at release time. Your job is to recommend the correct commit type so the next `cz bump` releases the change correctly.

## Source of truth

- Treat [docs/developer/versioning.md](../../docs/developer/versioning.md) as the policy of record, with [docs/developer/releasing.md](../../docs/developer/releasing.md) for release mechanics.
- Do not invent a parallel policy or infer release impact from a diff alone without grounding it in that policy.

## Decision workflow

1. Identify a concrete anchor, preferring in order: a pull request with base/head refs, a commit range, a local `git diff`, or an explicit changed-file list. Ask for missing context only when it is required to avoid guessing.
2. Gather the minimum evidence: `git diff --name-only`, `git diff --stat`, or a targeted diff, plus the policy docs above.
3. When you have base and head refs, run the advisory runtime dependency check and use its result as a signal:

   ```bash
   cargo run -p tq-release --locked -- check-runtime-deps --repo-root . --base-ref <base> --head-ref <head>
   ```

4. Apply the policy in order:
   - Does the change affect the published `tq` artifact at all? If not, it is non-shipping.
   - Does it alter a documented contract surface (CLI flags/behavior, config keys, rule IDs or default severities, exit codes, JSON schema) or an internal workspace API consumed by another crate? If yes, it is a `minor` change.
   - Does it preserve contract meaning but still change shipped behavior, including shipped runtime dependency updates? If yes, it is a `patch` change.
   - Is it repository-only maintenance (CI, GitHub Actions, pre-commit hooks, docs-site tooling, `tq-release`, `tq-docsgen`, dev-only dependencies)? If yes, it is non-shipping.

## Commit type mapping

- `minor` change: `feat:` (or `feat!:` / a `BREAKING CHANGE:` footer for an intentional contract break; pre-`1.0` both land as a minor bump).
- `patch` change: `fix:`.
- Non-shipping change: `chore:`, `build:`, `ci:`, `docs:`, `refactor:`, `test:`, or `style:` as appropriate. These do not trigger a release.

When the runtime dependency check reports a change, the dependency update is shipped and must be committed as `fix:` (or `feat:` if it widens behavior), even when the original commit was a Dependabot `chore(deps):`.

## Output contract

Return a concise decision:

- `Decision:` `minor`, `patch`, `non-shipping`, or `insufficient-context`
- `Commit type:` the Conventional Commit type to use (for example `fix:`)
- `Why:` a short explanation grounded in the policy and the signals you checked
- `Signals checked:` the concrete files, diffs, or commands used

If context is incomplete, return `Decision: insufficient-context` and ask only the smallest set of concrete follow-up questions needed to classify the change. Do not edit files or prepare a release unless explicitly asked.

{{$input}}
