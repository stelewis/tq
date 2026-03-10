# Architecture

Use this page to understand `tq`'s architecture boundaries.

## ADR usage in this project

`tq` uses Architectural Decision Records (ADRs) to capture significant architecture decisions.

ADRs are stored in `docs/adr/`.

## Architecture note

`tq` is a layered, deterministic static analysis tool for test quality:

- **Composition root:** `crates/tq-cli/src/main.rs` resolves config, validates targets, plans target runs, constructs rules, executes the engine, and selects the output reporter.
- **Config boundary:** `crates/tq-config` owns strict config loading, precedence, validation, and materialization into runtime target config.
- **Discovery boundary:** `crates/tq-discovery` scans configured source and test roots into immutable analysis indexes.
- **Domain execution:** `crates/tq-engine` plans target-scoped runs, enforces engine invariants, and aggregates deterministic findings.
- **Rule implementations:** `crates/tq-rules` contains the built-in rule registry, rule selection, and focused rule implementations with stable rule IDs.
- **Output adapters:** `crates/tq-reporting` renders text and JSON output without pulling reporting concerns into the engine.
- **Tooling adapters:** `crates/tq-docsgen` and `crates/tq-release` own docs generation and release-policy enforcement so repository tooling follows the same Rust-first architecture as the runtime.

The core design emphasizes strict boundaries, immutable analysis inputs, stable rule contracts, and deterministic output ordering.
