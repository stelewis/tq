# Exit Codes

`tq check` uses:

- `0`: no diagnostics at `error` severity or higher
- `1`: diagnostics at `error` severity or higher
- `2`: invalid CLI options, invalid configuration, or IO/runtime setup errors

## Exit behavior toggles

- `--exit-zero`: return `0` even when diagnostics are found
