# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-03-02

### Added

- Added machine-readable JSON output.

### Changed

- Standardized docs, CI, and contributor workflows to use `uv run tq check`.

### Removed

- Removed `check_test_quality` command and legacy implementation.
- Removed legacy config namespace `[tool.test_quality]` from project configuration.

## [0.2.0] - 2026-03-02

### Added

- Added `tq check` as the canonical CLI entrypoint with deterministic output, strict exit semantics, and rule selection controls.
- Added a strict analysis architecture with immutable indexing, explicit rule contracts, deterministic finding aggregation, and terminal reporting.
- Added built-in rule coverage for mapping, structure alignment, test file size limits, and orphaned tests.

### Deprecated

- `check_test_quality` now runs as a compatibility shim that forwards to `tq check` and emits a deprecation warning.
- Legacy configuration namespace `[tool.test_quality]` is deprecated in favor of `[tool.tq]`.

### Changed

- Runtime configuration now uses strict `[tool.tq]` validation and explicit CLI-over-config precedence, including isolated mode behavior.

## [0.1.0] - 2026-03-02

### Added

- Initial release of the `tq` tool quality checker.
- Project scaffolding and core functionality.
