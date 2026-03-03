# Reference

tq is designed with operator ergonomics that are similar to `ruff` and `ty`.

## Overview

- [CLI](./cli.md): command model, options, and output formats
- [Configuration](./configuration.md): `[tool.tq]` keys, validation, and precedence
- [Exit Codes](./exit-codes.md): process exit semantics
- [Rules](./rules/index.md): rule IDs, severities, and behavior

## Contract principles

- rule IDs are stable
- diagnostics are deterministic
- configuration is strict and fail-fast
