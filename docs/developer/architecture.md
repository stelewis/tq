# Architecture

Architecture overview for contributors.

## ADR usage in this project

`tq` uses Architectural Decision Records (ADRs) to record significant architecture decisions.

ADRs are stored in `docs/adr/`.

## Architecture note

`tq` is a layered, deterministic static analysis tool for test quality:

- **Composition root:** `src/tq/cli/main.py` wires config resolution, filesystem discovery, rule construction, engine execution, and reporting.
- **Config boundary:** `src/tq/config/loader.py` enforces strict key validation, deterministic precedence, and explicit materialization into `TqConfig`.
- **Discovery boundary:** `src/tq/discovery/filesystem.py` scans source/test trees into immutable `AnalysisIndex` data.
- **Domain execution:** `src/tq/engine/runner.py` runs `Rule` protocol implementations over immutable `AnalysisContext` and aggregates sorted, deterministic findings.
- **Rule implementations:** `src/tq/rules/` contains focused rules with stable `RuleId` contracts.
- **Output adapters:** `src/tq/reporting/terminal.py` and `src/tq/reporting/json.py` format results for human and machine consumers.

The core design emphasis is strict boundaries, immutable analysis inputs, stable rule contracts, and deterministic output ordering.
