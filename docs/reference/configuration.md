# Configuration

Canonical configuration namespace:

- `[tool.tq]`

## Precedence

Configuration is applied in this order (highest precedence first):

1. Dedicated CLI flags
2. Explicit CLI config overrides
3. Discovered project configuration
4. Discovered user configuration

Isolated mode ignores discovered configuration files.

## Rule severity vocabulary

- `error`
- `warning`
- `info`

Rule IDs remain stable even when severities are remapped.
