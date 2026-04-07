# Exit Codes

`tq check` uses:

- `0`: no findings at or above the configured fail threshold
- `1`: one or more findings at or above the configured fail threshold
- `2`: invalid CLI options, invalid configuration, or IO/runtime setup errors

The default fail threshold is `error` severity.

## Exit behavior toggles

- `--exit-zero`: return `0` even when diagnostics are found
- `--fail-on <SEVERITY>`: set the minimum severity that triggers exit code `1`
  - `error` (default): exit `1` only when there are `error` findings
  - `warning`: exit `1` when there are `warning` or `error` findings
  - `info`: exit `1` when there are any findings

Configure the fail threshold in `pyproject.toml`:

```toml
[tool.tq]
fail_on = "warning"
```

Note: severity overrides applied via `severity_overrides` or `--severity` affect findings before the fail threshold is evaluated.
