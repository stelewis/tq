---
title: Repository hardening and architecture cleanup
date_created: 2026-07-12
---

# Implementation Plan: Repository hardening and architecture cleanup

Address the eight material findings from the repository-wide audit. The goal is not only to close individual defects, but to remove the design shapes that made them possible: mutable release inputs, untyped policy logic in YAML, hidden IO in domain planning, duplicated vocabularies, and infallible contracts that force silent error handling.

## Architecture and design

Use breaking internal API changes freely. Do not add compatibility wrappers, fallback call paths, or transitional aliases. Each tranche should delete the old shape in the same change that introduces the replacement.

## Tasks

### 1. Harden the release boundary

- [x] Pin publish checkout to the validated commit: update [.github/workflows/publish.yml](../../.github/workflows/publish.yml) so the publish job checks out the `workflow_run.head_sha` or an equivalent immutable SHA captured by `prepare`, not `refs/tags/<release_tag>`. Keep the SemVer tag validation, but treat the tag as release metadata rather than the checkout authority.
- [x] Add a publish concurrency group keyed by release tag in [.github/workflows/publish.yml](../../.github/workflows/publish.yml) so reruns or duplicate `workflow_run` completions cannot race through release creation.
- [x] Exact-pin `maturin` in [.github/dev-tools.toml](../../.github/dev-tools.toml), including any required `zig`/extra policy needed for cross-platform wheels. The pin manifest should be the source of truth for CI wheel builds and local release builds.
- [x] Update `tq-dev` release planning in [crates/tq-dev/src/release.rs](../../crates/tq-dev/src/release.rs) to read the manifest and construct `uv run --isolated --with ...` from the pinned tool version instead of embedding `maturin>=1.11,<2.0`.
- [x] Update wheel build steps in [.github/workflows/ci.yml](../../.github/workflows/ci.yml) to use the pinned maturin version consistently. Prefer routing through `cargo dev release build` when feasible; otherwise generate the exact `--with` value from one manifest-owned constant surfaced by `tq-dev`.
- [x] Extend pin verification in [crates/tq-dev/src/policy.rs](../../crates/tq-dev/src/policy.rs) so the maturin pin is enforced across the manifest, local release plan, CI wheel-build commands, and any docs or action surfaces that mention the release builder.
- [x] Widen the artifact attestation job dependencies in [.github/workflows/ci.yml](../../.github/workflows/ci.yml) so attestations are created only after the same quality, policy, and security jobs required for publish have passed.
- [x] Add contract tests in `crates/tq-dev/tests/` for release plan rendering, maturin pin verification, and publish checkout validation.

### 2. Make rule evaluation fallible and eliminate impossible constructors

- Change the `Rule` trait in [crates/tq-rules/src/lib.rs](../../crates/tq-rules/src/lib.rs) or its owning module so `evaluate` returns `Result<Vec<Finding>, RulesError>` instead of `Vec<Finding>`.
- Update every rule module in [crates/tq-rules/src/](../../crates/tq-rules/src) to use `?` when constructing findings. Delete every `if let Ok(finding)` branch that can silently drop a diagnostic.
- Update `tq-engine` runner code to preserve and report rule-evaluation failures as typed execution errors rather than partial success. A failed rule evaluation should fail the check run unless a future explicit policy says otherwise.
- Replace runtime parsing of builtin rule IDs with const or static construction. Use the existing `RuleId` owned/static representation, or add a deliberately named `RuleId::from_static_unchecked` only if the invariant is confined to literal builtins and tested. Remove `RulesError::InvalidBuiltinRuleId` if it becomes impossible.
- Simplify builtin rule constructors so static builtins are infallible. Propagate that cleanup through registry construction, CLI wiring, tests, and documentation.
- Add regression tests proving invalid `Finding` construction fails a rule run instead of disappearing from output, and proving builtin registry construction cannot fail for static IDs.

### 3. Make engine planning pure and centralize shared domain vocabulary

- Move filesystem discovery out of `tq-engine` planning. The CLI or another composition-root boundary should call [crates/tq-discovery](../../crates/tq-discovery) and pass typed analysis context into `tq-engine`.
- Redesign the target flow to remove the near-isomorphic `TqTargetConfig` → `TargetPlanInput` → `TargetContext` shuttle. Keep separate types only where they prove a boundary invariant; otherwise collapse or introduce one validated target planning input owned by the right crate.
- Add or move shared vocabulary into `tq-core` for path display normalization, Python test-file naming, test/source path mapping, and default rule options. Downstream crates should import the shared owner rather than reimplementing local helpers.
- Delete duplicate implementations of `path_to_forward_slashes`, `is_test_module`/`is_unit_test_filename`, `strip_prefix("test_")` mapping logic, and the hardcoded `600` max-test-file default from downstream crates.
- Decide and document symlink behavior in `tq-discovery`: either report skipped symlinks as diagnostics or explicitly model them as ignored entries. Keep traversal defenses strict for `..`, absolute paths, symlinks, and root escape.
- Tighten ordering semantics in `tq-engine` reporting if needed. If the current message-length sort is intentional, name the ordering contract; otherwise replace it with a domain-stable sort key.
- Add cross-crate tests for shared test-file vocabulary, default propagation, pure engine planning with prebuilt analysis context, and discovery symlink behavior.

### 4. Move remaining workflow policy logic into `tq-dev`

- Replace the inline `change-scope` classifier in [.github/workflows/ci.yml](../../.github/workflows/ci.yml) with a tested `cargo dev` command that returns typed scope results for docs-only, runtime dependency, security, and release-relevant changes. Treat unknown paths as requiring the broader gate, not as safe to skip.
- Replace the frozen pre-commit policy Ruby in [.github/workflows/frozen-pre-commit-policy.yml](../../.github/workflows/frozen-pre-commit-policy.yml) with a `cargo dev policy` subcommand that reuses the YAML parsing boundary documented in [ADR 0003](../adr/0003-hand-rolled-yaml-parsing-in-tq-dev.md).
- Replace the pinned-actions sed/bash policy in [.github/workflows/pinned-actions-policy.yml](../../.github/workflows/pinned-actions-policy.yml) with a `cargo dev policy` subcommand that shares the external action reference scanner with the scheduled drift audit.
- Move release artifact set validation from [.github/scripts/verify-release-artifact-set.sh](../../.github/scripts/verify-release-artifact-set.sh) into `cargo dev release verify-artifacts`, alongside artifact content policy. The Rust verifier should validate both the expected artifact set and forbidden archive members in one typed report.
- Remove obsolete scripts and workflow heredocs after each replacement lands. Do not keep the shell/Ruby versions as fallback paths.
- Scope docs Pages permissions to the deploy job in [.github/workflows/docs-pages.yml](../../.github/workflows/docs-pages.yml), and consider `npm ci --ignore-scripts` for docs dependency installation unless the docs build proves lifecycle scripts are required.
- Add missing workflow `timeout-minutes` values for drift issue sync/fail jobs and Copilot setup where they are currently absent.
- Add integration tests for each new `tq-dev` workflow command, including fail-closed behavior for unknown change paths, malformed action refs, non-SHA pre-commit revisions, and malformed artifact sets.

## Verification

- Run `cargo fmt --all --check` after each tranche.
- Run `cargo clippy --workspace --all-targets --locked -- -D warnings` after each Rust tranche.
- Run `cargo test --workspace --locked` after each Rust tranche and after workflow-command migrations.
- Run `cargo dev policy verify-pins --repo-root .` after changing tool pins, release build commands, or workflow surfaces that consume `.github/dev-tools.toml`.
- Run `cargo dev release build --dry-run --repo-root .` and `cargo dev release verify-artifacts --dist-dir dist` for release-plan and artifact-verifier changes. For verifier changes, include synthetic fixture coverage rather than relying only on live `dist/` contents.
- Run `cargo dev check --profile full all --repo-root .` before considering the plan complete.
- Review workflow diffs for permissions, checkout credentials, SHA pinning, timeout coverage, and absence of inline parsing logic.
- Confirm that deleted fallback paths are actually gone with targeted searches for `maturin>=`, `refs/tags/${{`, `if let Ok(finding)`, workflow Ruby heredocs, pinned-action sed parsing, and `verify-release-artifact-set.sh` references.
