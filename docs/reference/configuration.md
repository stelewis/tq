# Configuration

Configure `tq` under:

- `[tool.tq]`

`tq` loads configuration strictly and fails fast on unknown keys and invalid types.

## Examples

Start from one of these generated examples, then use the sections below to understand the exact rules and validation model.

### Minimal required configuration

<!-- BEGIN GENERATED:configuration-minimal-config -->
```toml
[tool.tq]

[[tool.tq.targets]]
name = "app"
package = "your_package"
source_root = "src"
test_root = "tests"
```
<!-- END GENERATED:configuration-minimal-config -->

### Typical project configuration

<!-- BEGIN GENERATED:configuration-typical-config -->
```toml
[tool.tq]
init_modules = "ignore"
max_test_file_non_blank_lines = 600
qualifier_strategy = "allowlist"
allowed_qualifiers = ["regression", "config", "fixtures_golden"]

[[tool.tq.targets]]
name = "app"
package = "your_package"
source_root = "src"
test_root = "tests"

[[tool.tq.targets]]
name = "scripts"
package = "scripts"
source_root = "."
test_root = "tests"
```
<!-- END GENERATED:configuration-typical-config -->

## Config file locations

`tq` reads `pyproject.toml` using this model:

- explicit config: file passed via `--config`
- project config: nearest `pyproject.toml` from current working directory upward
- user config: `~/.config/tq/pyproject.toml`

Only the `[tool.tq]` table is read.

## Precedence

Configuration is applied in this order (highest first):

1. Dedicated CLI flags
2. Explicit CLI config overrides (`--config`)
3. Discovered project configuration (`pyproject.toml` nearest cwd)
4. Discovered user configuration (`~/.config/tq/pyproject.toml`)

Use `--isolated` to ignore discovered configuration files.

## Targets model

`[tool.tq]` defines shared defaults and one or more required targets.

- Shared defaults apply to all targets unless overridden per target.
- `[[tool.tq.targets]]` defines strict source/test boundaries for one analysis unit.
- `tq check` runs all configured targets by default.
- `tq check --target <name>` runs only selected targets.

### How targets are represented in TOML

In TOML, `targets` is an array-of-tables under `[tool.tq]`. Each `[[tool.tq.targets]]` block appends one entry to the same `targets` list.

- This is equivalent to a `targets = [...]` key at the data-model level.
- You must declare at least one `[[tool.tq.targets]]` block.
- Each target entry is validated strictly.

## Shared top-level keys

Top-level keys in `[tool.tq]` (other than `targets`) act as shared defaults.

### `init_modules` (optional)

- Type: `string`
- Default: `"include"`
- Allowed values:
  - `include`: treat `__init__.py` modules like any other source module
  - `ignore`: skip `__init__.py` modules in mapping checks

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

### `severity_overrides` (optional)

- Type: `inline-table` with rule ID keys and severity string values
- Default: `{}`
- Meaning: override the severity applied to findings for specific rule IDs
- Validation: rule IDs must be valid kebab-case rule IDs; severity values must be `error`, `warning`, or `info`
- Note: unknown built-in rule IDs are rejected when `tq` resolves the active rule set
- Example: `severity_overrides = { structure-mismatch = "error", test-file-too-large = "info" }`
- Effect: the effective severity replaces the default for those rules in reporting and exit-code evaluation

### `fail_on` (optional)

- Type: `string`
- Default: `"error"`
- Allowed values:
  - `error`: exit nonzero only when there are findings at `error` severity (default behavior)
  - `warning`: exit nonzero when there are findings at `warning` severity or higher
  - `info`: exit nonzero when there are any findings
- Meaning: minimum severity level that triggers a nonzero exit code
- Note: `fail_on` is a global-only key and cannot be set per target

## Target entries (`[[tool.tq.targets]]`)

`targets` is required and must contain one or more entries.

The full `targets` set is validated as follows:

- at least one target is required
- each target `name` must be unique and kebab-case
- target required fields must be non-empty
- duplicate effective source package roots across targets are rejected

Each `[[tool.tq.targets]]` entry supports:

### `name` (required)

- Type: `string`
- Meaning: stable target identifier for filtering and reporting
- Validation: kebab-case, unique within `targets`

### `package` (required)

- Type: `string`
- Meaning: import package path to analyze (for example `tq` or `scripts`)
- Validation: dotted Python identifier segments (for example `tq`, `scripts`, `my_pkg.tools`)

### `source_root` (required)

- Type: `string`
- Meaning: root directory that contains source packages
- Resolution: relative paths resolve from the directory containing the `pyproject.toml` file that defines `[[tool.tq.targets]]`
- Validation: non-empty string

### `test_root` (required)

- Type: `string`
- Meaning: root directory that contains tests
- Resolution: relative paths resolve from the directory containing the `pyproject.toml` file that defines `[[tool.tq.targets]]`
- Validation: non-empty string

### Optional target overrides

A target may override any shared top-level optional key:

- `init_modules`
- `max_test_file_non_blank_lines`
- `qualifier_strategy`
- `allowed_qualifiers`
- `select`
- `ignore`
- `severity_overrides`

### `accepted_findings` (optional, target-only)

Accept a finding for one exact file without hiding the rule elsewhere. Each entry requires `rule` and `path`. An optional `rationale` can explain the exception; when provided, it must be a non-empty string. Only `mapping-missing-test` and `test-file-too-large` support acceptance.

```toml
[[tool.tq.targets]]
name = "app"
package = "your_package"
source_root = "src"
test_root = "tests"

[[tool.tq.targets.accepted_findings]]
rule = "mapping-missing-test"
path = "boundary.py"
rationale = "Exercised by the shared contract suite."

[[tool.tq.targets.accepted_findings]]
rule = "test-file-too-large"
path = "your_package/test_boundary.py"
max_non_blank_lines = 740
rationale = "One cohesive boundary contract."
```

Mapping paths are relative to the target's source package directory (`source_root` joined with `package`); size paths are relative to `test_root`. Paths use `/` separators and must name a Python module or, for size findings, a `test_*.py` file. Absolute paths, platform prefixes, glob syntax, empty path components, `.` and `..` are rejected. Duplicate rule/path pairs and unknown entry keys are errors.

Size entries require a positive integer `max_non_blank_lines`: the largest reviewed size, using the rule's count of non-blank, non-comment lines. The underlying rule still uses the effective `max_test_file_non_blank_lines` threshold. A finding at or below its accepted ceiling is omitted; growth above it is reported normally, even when the file was previously accepted. Mapping entries do not permit a size ceiling.

Accepted findings are absent from text, JSON, summary counts, and exit-code evaluation. Other files, rules, and targets are unaffected. Growth findings show both the accepted ceiling and the effective rule limit.

If the underlying finding disappears, including after deletion or a threshold change, the entry produces an `unused-accepted-finding` warning with a suggestion to remove it. Both `unused-accepted-finding` and the underlying rule must be active: include both when using an explicit `select` list. All rules are selected by default. Use `ignore = ["unused-accepted-finding"]` to disable cleanup warnings without disabling acceptance or growth detection. Severity overrides for the underlying rule do not affect cleanup warnings; set `severity_overrides = { unused-accepted-finding = "info" }` to change their severity independently. `fail_on` applies normally.

Acceptances belong only to their target; there is no shared default or CLI suppression flag. Configuration precedence replaces the complete target list, including its acceptances, rather than merging individual entries.

## Built-in rule IDs

- `mapping-missing-test`
- `structure-mismatch`
- `test-file-too-large`
- `orphaned-test`
- `unused-accepted-finding`

## Rule and severity model

Rule IDs are stable kebab-case identifiers. Severity vocabulary is fixed:

- `error`
- `warning`
- `info`

Severity remapping may be applied at CLI/config boundaries without changing rule IDs. Use `severity_overrides` in config or `--severity` on the CLI to promote or demote individual rules.

## CLI override example

Use discovered config but tighten the file-size limit for one run:

```sh
tq check --max-test-file-non-blank-lines 300
```

Run one configured target:

```sh
tq check --target scripts
```

Run with an explicit config file:

```sh
tq check --config /path/to/pyproject.toml
```
