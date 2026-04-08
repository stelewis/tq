---
id: 0001
title: "Define tq CLI and configuration contract"
status: accepted
date: 2026-03-02
tags:
- cli
- configuration
- rules
supersedes: null
superseded_by: null
---

## Context

`tq` needs a stable, long-lived operator contract so CI integrations, automation, and contributor workflows do not drift over time.

The design target aligns with operator ergonomics used by Ruff and Ty:

- subcommand-first interface (`<tool> check`),
- strict and predictable exit code semantics,
- explicit rule identifiers with stable suppression and selection behavior,
- CLI-over-config precedence,
- configuration in `pyproject.toml` under a dedicated tool namespace.

## Decision

### 1. External command contract

`tq` uses a subcommand interface with `tq check` as the canonical command.

Output serves both human and machine consumers.

### 2. Exit code policy

`tq check` exit codes are:

- `0`: no findings at or above the configured fail threshold.
- `1`: one or more findings at or above the configured fail threshold.
- `2`: invalid CLI options, invalid configuration, or IO/runtime setup errors.

The default fail threshold is `error`, which preserves the original behavior. Users may raise the threshold strictness with `--fail-on` or `[tool.tq].fail_on`.

Severity remapping is applied before exit-code evaluation, so `severity_overrides` and `--severity` affect both reporting and exit status.

This mirrors Ruff and Ty conventions for normal and abnormal termination while keeping fail policy explicit in `tq`.

### 3. Configuration namespace and precedence

The configuration namespace is `[tool.tq]` in `pyproject.toml`.

Precedence policy:

- dedicated CLI flags override all config files,
- explicit CLI overrides override file-based settings,
- project configuration overrides user-level defaults,
- `--isolated` mode ignores discovered configuration.

### 4. Rule identifier and severity policy

Rule identifiers are stable kebab-case names, inspired by Ty’s human-readable slugs and Ruff’s stable rule identity model.

Example built-in rules:

- `mapping-missing-test` (`error`)
- `structure-mismatch` (`warning`)
- `test-file-too-large` (`warning`)
- `orphaned-test` (`warning`)

Severity levels are `error`, `warning`, and `info`. CLI-level severity remapping is part of the public contract and will support promoting/demoting specific rules without changing rule IDs.

## Consequences

- Implementation and docs can evolve without contract ambiguity.
- Downstream users have one stable command and one config namespace.
- CI and automation scripts stay deterministic and simpler to maintain.

## Alternatives considered

### Use opaque numeric rule IDs

Rejected. Numeric IDs are less self-describing and weaker for documentation and developer ergonomics.

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
