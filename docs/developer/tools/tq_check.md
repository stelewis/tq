# tq check contract (v1)

This document freezes the external user contract for `tq check`.

The contract is intentionally explicit so implementation work can proceed without CLI, configuration, or rule-policy drift.

## Scope

This page defines:

- command and exit behavior,
- configuration namespace and precedence,
- stable v1 rule IDs and default severities,
- migration behavior from the legacy checker surface.

## Command model

Primary entrypoint:

- `tq check`

Design alignment targets:

- subcommand-first CLI (Ruff/Ty style),
- deterministic diagnostic output,
- machine-readable reporting support.

## Exit code policy

`tq check` exit codes:

- `0`: no diagnostics at severity `error` or higher.
- `1`: one or more diagnostics at severity `error` or higher.
- `2`: invalid CLI options, invalid configuration, or IO/runtime setup errors.

Behavioral toggles such as `--exit-zero` and warning escalation are part of the planned CLI alignment with Ty and are treated as contract-level features.

## Configuration contract

Canonical configuration namespace:

- `[tool.tq]`

Legacy namespace:

- `[tool.test_quality]` is deprecated and migration-only.

Precedence policy:

- dedicated CLI flags override config file values,
- explicit CLI config overrides override discovered config,
- project config overrides user config,
- isolated mode ignores discovered configuration files.

## v1 rule IDs and default severities

Rule IDs are stable kebab-case identifiers.

Stable v1 rules:

- `mapping-missing-test` (default severity: `error`)
- `structure-mismatch` (default severity: `warning`)
- `test-file-too-large` (default severity: `warning`)
- `orphaned-test` (default severity: `warning`)

Severity vocabulary:

- `error`, `warning`, `info`

Severity remapping is supported at CLI/config boundaries without changing rule IDs.

## Migration policy

Migration from legacy behavior proceeds in three steps:

1. `tq check` is the documented default entrypoint.
2. `check_test_quality` remains as a compatibility shim for at least one minor release.
3. The shim is removed in a later minor release with explicit release-note communication.

## References

- [ADR 0001](../../adr/0001-tq-cli-contract-and-v1-policy.md)
- [Ruff docs](https://docs.astral.sh/ruff/)
- [Ruff linter](https://docs.astral.sh/ruff/linter/)
- [Ruff configuration](https://docs.astral.sh/ruff/configuration/)
- [Ty docs](https://docs.astral.sh/ty/)
- [Ty CLI reference](https://docs.astral.sh/ty/reference/cli/)
- [Ty exit codes](https://docs.astral.sh/ty/reference/exit-codes/)
- [Ty rules reference](https://docs.astral.sh/ty/reference/rules/)
