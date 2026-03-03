# Configuration

Configuration namespace:

- `[tool.tq]`

`tq` loads configuration strictly and fails fast on unknown keys or invalid types.

## Config file locations

`tq` reads from `pyproject.toml` files using this model:

- explicit config: file passed via `--config`
- project config: nearest `pyproject.toml` from current working directory upward
- user config: `~/.config/tq/pyproject.toml`

Only the `[tool.tq]` table is read.

## Precedence

Configuration is applied in this order (highest precedence first):

1. Dedicated CLI flags
2. Explicit CLI config overrides (`--config`)
3. Discovered project configuration (`pyproject.toml` nearest cwd)
4. Discovered user configuration (`~/.config/tq/pyproject.toml`)

Isolated mode (`--isolated`) ignores discovered configuration files.

## Targets model

`[tool.tq]` defines shared defaults and one or more required targets.

- Shared defaults apply to all targets unless overridden per target.
- `[[tool.tq.targets]]` defines strict source/test boundaries for one analysis unit.
- `tq check` runs all configured targets by default.
- `tq check --target <name>` runs only selected targets.

## Top-level keys

### `targets` (required)

- Type: `array[table]`
- Meaning: explicit list of analysis targets
- Validation:
  - at least one target is required
  - each target `name` must be unique and kebab-case
  - target required fields must be non-empty
  - duplicate effective source package roots across targets are rejected

### `ignore_init_modules` (optional)

- Type: `boolean`
- Default: `false`
- Meaning: when `true`, mapping checks skip `__init__.py` modules

### `max_test_file_non_blank_lines` (optional)

- Type: `integer`
- Default: `600`
- Validation: must be `>= 1`
- Meaning: threshold used by `test-file-too-large`

### `qualifier_strategy` (optional)

- Type: `string`
- Default: `any-suffix`
- Allowed values:
  - `none`: do not strip suffixes from test module names
  - `any-suffix`: allow any suffix to map to a base module
  - `allowlist`: only strip suffixes listed in `allowed_qualifiers`

### `allowed_qualifiers` (optional)

- Type: `array[string]`
- Default: `[]`
- Meaning: qualifier suffixes allowed when `qualifier_strategy = "allowlist"`
- Constraint: must be non-empty when using `allowlist`

### `select` (optional)

- Type: `array[string]`
- Default: all built-in rules when omitted
- Meaning: explicit allow-list of rule IDs to run
- Validation: values must be valid kebab-case rule IDs and known built-ins

### `ignore` (optional)

- Type: `array[string]`
- Default: `[]`
- Meaning: rule IDs to skip after selection is resolved
- Validation: values must be valid kebab-case rule IDs and known built-ins

## Target keys

Each `[[tool.tq.targets]]` entry supports:

### `name` (required)

- Type: `string`
- Meaning: stable target identifier for filtering and reporting
- Validation: kebab-case, unique within `targets`

### `package` (required)

- Type: `string`
- Meaning: import package path to analyze (for example `tq` or `scripts`)
- Validation: non-empty string

### `source_root` (required)

- Type: `string`
- Meaning: root directory that contains source packages
- Resolution: relative paths resolve from current working directory
- Validation: non-empty string

### `test_root` (required)

- Type: `string`
- Meaning: root directory that contains tests
- Resolution: relative paths resolve from current working directory
- Validation: non-empty string

### Optional target overrides

A target may override any top-level optional key:

- `ignore_init_modules`
- `max_test_file_non_blank_lines`
- `qualifier_strategy`
- `allowed_qualifiers`
- `select`
- `ignore`

## Built-in rule IDs

- `mapping-missing-test`
- `structure-mismatch`
- `test-file-too-large`
- `orphaned-test`

## Rule and severity model

Rule IDs are stable kebab-case identifiers. Severity vocabulary is fixed:

- `error`
- `warning`
- `info`

Severity remapping may be applied at CLI/config boundaries without changing rule IDs.

## Examples

## Minimal required configuration

```toml
[tool.tq]
[[tool.tq.targets]]
name = "tq"
package = "tq"
source_root = "src"
test_root = "tests"
```

## Typical project configuration

```toml
[tool.tq]
ignore_init_modules = true
max_test_file_non_blank_lines = 600
qualifier_strategy = "allowlist"
allowed_qualifiers = ["regression", "config", "fixtures_golden"]

[[tool.tq.targets]]
name = "tq"
package = "tq"
source_root = "src"
test_root = "tests"

[[tool.tq.targets]]
name = "scripts"
package = "scripts"
source_root = "."
test_root = "tests"
```

## CLI override example

Run with discovered config but tighten the file-size limit for one invocation:

```sh
tq check --max-test-file-non-blank-lines 300
```

Run only one configured target:

```sh
tq check --target scripts
```

Run with an explicit config file:

```sh
tq check --config /path/to/pyproject.toml
```
