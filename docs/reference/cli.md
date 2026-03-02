# CLI Reference

`tq` uses a subcommand-first CLI.

## Usage

- `tq check`

## Output formats

- `text` (default): concise terminal diagnostics.
- `json`: machine-readable diagnostics for CI and tooling.

Use:

```sh
tq check --output-format json
```

## Language support

`tq` currently targets Python codebases.
