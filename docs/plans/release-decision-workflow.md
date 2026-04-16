---
title: Release decision workflow
date_created: 2026-04-17
---

# Implementation Plan: release decision workflow

Implement the release-decision workflow tracked by GitHub issues #41, #42, #43, and #44 so release classification becomes explicit, reviewable, and enforceable. The plan is to keep `docs/developer/versioning.md` as the policy source of truth, add a short maintainer checklist for release vs no-release decisions, use PR labels as the canonical release-intent signal, and extend `tq-release` with a focused consistency check that CI can invoke. The semantic release decision remains human-owned. CI only checks declared intent against narrow, policy-owned signals and asks maintainers to confirm or correct mismatches.

## Architecture and design

The workflow should separate policy, metadata, and enforcement cleanly.

- Policy lives in `docs/developer/versioning.md` and related maintainer-facing docs. This remains the canonical definition of what is release-relevant.
- PR metadata carries declared intent. Prefer labels as the canonical signal: `release:none`, `release:patch`, and `release:minor`.
- Enforcement lives in `tq-release`, but GitHub-specific context gathering stays in CI. The workflow should gather changed files and PR labels, then pass normalized inputs into `tq-release` rather than teaching `tq-release` to call GitHub APIs.
- CI should enforce consistency, not semantic inference. It should detect suspicious contradictions such as `release:none` on a PR that changes a contract surface or a shipped runtime dependency, and it should require version and changelog updates when a PR declares a release.

Recommended design choices:

- Use labels, not PR body parsing, as the canonical release-intent mechanism. Labels are visible in review, easy for maintainers to update, and compatible with bot-authored PRs.
- Require exactly one `release:*` label on merge-ready PRs.
- Treat missing or multiple release labels as a policy failure.
- Keep the suspicious-surface rules intentionally narrow and file/path based where possible. The check should escalate ambiguous cases to maintainers instead of trying to prove semantic impact automatically.
- Extend the existing `verify-release-policy` boundary by adding a dedicated release-intent verification command in `tq-release`, then invoke it from the main CI workflow.

Non-goals:

- No fully automatic diff-to-release decision engine.
- No duplicate policy encoded independently in workflow YAML.
- No GitHub API client logic inside workspace domain crates.

## Tasks

### 1. Refine policy and maintainer workflow (#42)

- Update `docs/developer/versioning.md` with a short release-decision checklist that answers:
  - whether the change touches a documented contract surface
  - whether it affects the published `tq` artifact or only repository tooling
  - whether it changes a shipped runtime dependency
  - whether it fixes a shipped security issue
  - whether it changes an internal workspace API consumed by another crate
- Cross-link the checklist from release-maintainer docs where it materially helps, likely in `docs/developer/releasing.md`.
- Keep one canonical checklist and link to it instead of restating the same questions in multiple docs.
- Clarify the distinction between shipped-product dependency changes and dev-only dependency changes.

### 2. Add explicit PR release-intent metadata (#43)

- Create repository labels for `release:none`, `release:patch`, and `release:minor`.
- Document that one and only one `release:*` label must be present on PRs before merge.
- Decide whether draft PRs are exempt until ready for review. Recommended: enforce on non-draft PRs only.
- Check Dependabot and other automation flows explicitly:
  - confirm label application does not block bot PR creation
  - confirm maintainers can classify bot PRs during review without extra workflow friction
  - if desired, add a lightweight auto-labeler only for obvious maintenance-only bot PRs, but keep human review authoritative
- Update contributor guidance or PR templates only if they support the label-based workflow cleanly. Do not make PR-body fields the source of truth.

### 3. Design the CI integration boundary (#44)

- Extend `crates/tq-release` with a focused command for release-intent verification.
- Keep the command input generic and testable. It should accept normalized inputs such as:
  - declared release intent
  - changed file paths
  - optional flags indicating whether version and changelog updates are present
- Keep GitHub-specific data collection in the workflow layer or a thin adapter step.
- Define the first suspicious-signal set conservatively. Candidate signals:
  - changes under `crates/tq-cli/`, `crates/tq-config/`, `crates/tq-core/`, `crates/tq-engine/`, `crates/tq-rules/`, or `crates/tq-reporting/`
  - changes to docs that define contract surfaces such as `docs/developer/versioning.md`, `docs/reference/**`, or exit-code docs when those represent contract updates
  - runtime dependency changes in the shipped artifact path, including `Cargo.toml` or lockfile changes that affect `tq-cli` or crates reachable from the published binary
  - workspace version or changelog changes that imply release intent already exists
- Define clear default non-release signals:
  - `.github/workflows/**`
  - `.pre-commit-config.yaml`
  - dev-tool-only sections of `pyproject.toml`
  - `uv.lock` changes limited to dev tooling
  - release-tooling-only changes under `crates/tq-release/` unless they also change release policy docs or workflow semantics materially
- Fail with actionable diagnostics that say what signal triggered the check and what maintainer action is expected.

### 4. Integrate the new check into CI

- Add a CI step in the existing main workflow that runs on pull requests.
- Gather PR labels and changed files in the workflow, normalize them, and pass them to the new `tq-release` command.
- Keep enforcement ordering simple:
  - first verify exactly one release-intent label
  - then run the release-intent consistency check
  - if the declared intent is `release:patch` or `release:minor`, require the expected version/changelog state
- Decide whether the version/changelog requirement applies on every release-labeled PR or only before merge. Recommended: require it in the PR so reviewers see the final contract impact.
- Ensure bot PRs run through the same check once labeled.

### 5. Add tests and fixtures

- Add focused unit tests in `tq-release` for the release-intent checker.
- Cover at least these cases:
  - `release:none` with docs/CI/dev-tooling-only changes passes
  - `release:none` with contract-surface changes fails
  - `release:none` with a shipped runtime dependency change fails
  - `release:patch` with appropriate version/changelog updates passes
  - `release:patch` or `release:minor` with missing version/changelog updates fails
  - multiple or missing release labels fail before semantic checks
- Add CI-level contract coverage if the workflow logic has a non-trivial adapter layer.

### 6. Document the end-to-end maintainer flow

- Update maintainer-facing docs to explain:
  - how to choose a release label
  - what the CI failure messages mean
  - how to resolve mismatches
  - how bot PRs should be handled
- Keep the guidance short and operational. The policy doc should define the contract; the maintainer docs should describe the workflow.

## Delivery sequence

Implement in this order:

1. Finalize the policy wording and checklist in docs.
2. Add the release labels and document the PR classification rule.
3. Implement the `tq-release` release-intent checker with unit tests.
4. Integrate the checker into CI using workflow-collected inputs.
5. Update maintainer and contributor docs once the final workflow behavior is stable.

This order keeps design decisions explicit before code lands and avoids building CI behavior against an unstable policy.

## Verification

- Docs review confirms there is one canonical release-decision checklist and no policy duplication.
- Repository labels exist for all three release-intent states and the chosen PR workflow is documented.
- `tq-release` has focused automated tests for release-intent validation.
- CI fails when:
  - no `release:*` label is present on a non-draft PR
  - more than one `release:*` label is present
  - `release:none` conflicts with suspicious shipped-surface changes
  - `release:patch` or `release:minor` is declared without required version/changelog updates
- CI passes for known maintenance-only changes such as workflow updates, pre-commit bumps, and dev-tool lock updates.
- Full relevant validation passes after implementation:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets --locked -- -D warnings`
  - `cargo test --workspace --locked`
  - `cargo run -p tq-docsgen --locked -- generate all`
  - `cargo run -p tq-release --locked -- verify-release-policy --repo-root .`

## Risks and watchpoints

- Overly broad suspicious-surface rules will make the check noisy and reduce trust.
- GitHub-specific assumptions inside `tq-release` would blur boundaries and make testing harder.
- Requiring PR-body fields in addition to labels would create duplication and bot friction.
- Treating every dependency change as release-relevant would regress back to diff-based overreach.

The implementation should stay strict at the boundaries while remaining conservative about what it claims to know automatically.
