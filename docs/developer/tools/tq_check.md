# tq check contract

This document defines the external user contract for `tq check`.

The contract is intentionally explicit so users can proceed without CLI, configuration, or rule-policy drift.

## Scope

This page defines:

- command and exit behavior,
- configuration namespace and precedence,
- stable rule IDs and default severities,
- migration mapping from removed legacy surfaces.

## Command model

Primary entrypoint:

- `tq check`

Design alignment targets:

- subcommand-first CLI (Ruff/Ty style),
- deterministic diagnostic output,
- machine-readable reporting support.

## Output formats

`tq check` supports:

- text (default): concise human-readable terminal diagnostics.
- json (`--output-format json`): machine-readable diagnostics payload with stable fields:
  - finding fields: `rule_id`, `severity`, `message`, `path`, `line`, `suggestion`
  - summary fields: `errors`, `warnings`, `infos`, `total`

## Exit code policy

`tq check` exit codes:

- `0`: no diagnostics at severity `error` or higher.
- `1`: one or more diagnostics at severity `error` or higher.
- `2`: invalid CLI options, invalid configuration, or IO/runtime setup errors.

Behavioral toggles such as `--exit-zero` and warning escalation are part of the planned CLI alignment with Ty and are treated as contract-level features.

## Configuration contract

Canonical configuration namespace:

- `[tool.tq]`

Precedence policy:

- dedicated CLI flags override config file values,
- explicit CLI config overrides override discovered config,
- project config overrides user config,
- isolated mode ignores discovered configuration files.

## Rule IDs and default severities

Rule IDs are stable kebab-case identifiers.

Stable rules:

- `mapping-missing-test` (default severity: `error`)
- `structure-mismatch` (default severity: `warning`)
- `test-file-too-large` (default severity: `warning`)
- `orphaned-test` (default severity: `warning`)

Severity vocabulary:

- `error`, `warning`, `info`

Severity remapping is supported at CLI/config boundaries without changing rule IDs.
