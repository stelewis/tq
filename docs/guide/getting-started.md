# Getting Started

Run `tq check` at the root of your Python repository.

## First run

```sh
uv run tq check
```

`tq` discovers source files under `src/` and matching tests under `tests/`.

## Machine-readable output

Use JSON output for automation:

```sh
uv run tq check --output-format json
```

JSON output includes deterministic finding fields:

- `rule_id`
- `severity`
- `message`
- `path`
- `line`
- `suggestion`
