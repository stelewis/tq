---
id: 0001
title: "Freeze tq CLI contract, v1 rule policy, and migration path"
status: accepted
date: 2026-03-02
tags: ["cli", "configuration", "rules", "migration"]
supersedes: null
superseded_by: null
---

## Context

`tq` is moving from a project-specific script (`check_test_quality`) to a reusable open source tool. To avoid contract drift during implementation, we need an explicit phase-one decision that freezes the user-facing behavior before core rebuild work continues.

The design target is alignment with the operator ergonomics used by Ruff and Ty:

- subcommand-first interface (`<tool> check`),
- strict and predictable exit code semantics,
- explicit rule identifiers with stable suppression and selection behavior,
- CLI-over-config precedence,
- configuration in `pyproject.toml` under a dedicated tool namespace.

## Decision

### 1. External command contract

`tq` will use a subcommand interface with `tq check` as the primary command.

The phase-one contract freezes these outcomes:

- `tq check` is the canonical lint entrypoint.
- `check_test_quality` is legacy and enters migration mode, to be removed in a future release.
- Output modes will be designed for both human and machine use.

### 2. Exit code policy

`tq check` exit codes are:

- `0`: no diagnostics at `error` severity or higher.
- `1`: one or more diagnostics at `error` severity or higher.
- `2`: invalid CLI options, invalid configuration, or IO/runtime setup errors.

This mirrors Ruff and Ty conventions for normal and abnormal termination.

### 3. Configuration namespace and precedence

Configuration namespace is frozen as `[tool.tq]` in `pyproject.toml`.

Precedence policy is frozen as:

- dedicated CLI flags override all config files,
- explicit CLI overrides override file-based settings,
- project configuration overrides user-level defaults,
- `--isolated` mode ignores discovered configuration.

### 4. v1 rule identifier and severity policy

Rule identifiers are frozen as stable kebab-case names, inspired by Ty’s human-readable slugs and Ruff’s stable rule identity model.

Initial v1 rules:

- `mapping-missing-test` (default: `error`)
- `structure-mismatch` (default: `warning`)
- `test-file-too-large` (default: `warning`)
- `orphaned-test` (default: `warning`)

Severity levels are `error`, `warning`, and `info`. CLI-level severity remapping is part of the public contract and will support promoting/demoting specific rules without changing rule IDs.

### 5. Migration policy

Migration from legacy checker behavior is frozen as:

- phase-in: `tq check` becomes documented primary command,
- deprecation: `check_test_quality` remains as a compatibility shim for at least one minor release,
- removal: shim is removed in a subsequent minor release after explicit release note warnings.

Legacy config namespace `[tool.test_quality]` is treated as deprecated. During migration, projects should move to `[tool.tq]`.

## Consequences

- Implementation work can proceed without CLI or policy ambiguity.
- Downstream users get a stable contract for CI integration and automation.
- Migration has explicit guardrails and avoids sudden breakage.
- We accept temporary dual-surface complexity while the compatibility shim exists.

## Alternatives considered

### Keep `check_test_quality` as the long-term entrypoint

Rejected. It does not match the desired multi-command operator model and does not scale for future capabilities.

### Use opaque numeric rule IDs

Rejected. Numeric IDs are less self-describing and weaker for documentation and developer ergonomics.

### Preserve `[tool.test_quality]` as the canonical config namespace

Rejected. It binds the new OSS identity to a legacy script name and complicates future feature taxonomy.

## Related

- [Ruff overview](https://docs.astral.sh/ruff/)
- [Ruff linter](https://docs.astral.sh/ruff/linter/)
- [Ruff configuration](https://docs.astral.sh/ruff/configuration/)
- [Ruff versioning](https://docs.astral.sh/ruff/versioning/)
- [Ty overview](https://docs.astral.sh/ty/)
- [Ty CLI reference](https://docs.astral.sh/ty/reference/cli/)
- [Ty exit codes](https://docs.astral.sh/ty/reference/exit-codes/)
- [Ty rules reference](https://docs.astral.sh/ty/reference/rules/)
- [astral-sh/ruff repository](https://github.com/astral-sh/ruff)
- [astral-sh/ty repository](https://github.com/astral-sh/ty)
