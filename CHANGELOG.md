# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [0.13.0] - 2026-10-25

### Added

- Added target-local accepted findings for `mapping-missing-test` and `test-file-too-large`, with exact paths and optional rationales.
- Added finite size ceilings that expose growth beyond an accepted size without raising the global limit.
- Added `unused-accepted-finding` warnings with independent rule selection and severity controls to identify exceptions whose underlying finding no longer exists.

### Changed

- Size findings for accepted files include both the accepted ceiling and the effective rule limit.

## [0.12.2] - 2026-10-25

### Changed

- Refreshed shipped Rust dependencies and repository development dependencies.
- Updated the pinned Rust toolchain to 1.99.1, Python to 3.15.1, uv to 0.13.5, and Node to 26.12.1, with synchronized CI and publishing defaults.
- Refreshed frozen GitHub Actions and pre-commit hooks.

### Fixed

- Made dependency updates synchronize the local Python version selector.
- Updated release artifact inspection to handle zip 9's fallible member-name decoding.

### Security

- Prevented workflow-run metadata from being interpolated into release workflow shell scripts.
- Upgraded zip to 9.0.5 and Vite to 6.4.5 to resolve the Rust and documentation dependency advisories.

## [0.12.1] - 2026-10-12

### Changed

- Refreshed shipped runtime dependencies and repository development dependencies.
- Updated the pinned Rust toolchain to 1.99.0 and refreshed the Python, uv, maturin, Node, npm, and pre-commit toolchain pins.
- Added Python 3.15 package compatibility checks while retaining Python 3.11 as the minimum supported version.

### Fixed

- Aligned Clippy with the workspace Rust version and updated the developer harness for Rust 1.99 lint checks.
- Made dependency update plans synchronize toolchain pins and CI defaults, select Node's bundled npm version, and refresh frozen GitHub Actions with policy validation.
- Removed toolchain release-series caps so dependency updates select the latest stable releases.
- Fixed dependency update application for compact package metadata, repeated Action references, and shared workflow files.

## [0.12.0] - 2026-08-14

### Added

- Added the `cargo dev` developer harness, which replaces the `tq-release` crate with typed, tested commands for quality gates, dependency freshness and security audits, repository and release policy verification, environment doctoring, release builds, and artifact verification. Change scoping and supply-chain policy enforcement now run from the same local tooling as CI.

### Changed

- Updated shipped runtime dependencies.
- Bumped the pinned Rust toolchain to 1.97.1 and refreshed the pinned Python, uv, Node, and npm toolchains alongside GitHub Actions, pre-commit hooks, and Python development dependencies.

### Security

- Isolated privileged release workflow jobs: release candidates are validated in read-only jobs before checkout-free attestation, provenance and immutable tag binding are reverified at every irreversible boundary, and post-publish smoke checks run without write or OIDC permissions.
- Hardened maintenance tool provisioning by verifying scanner metadata and published lockfiles, requiring exact executable cache keys with no fallback, and removing `cargo-outdated` in favor of Cargo-native drift checks while Dependabot owns scheduled updates.

## [0.11.5] - 2026-07-01

### Changed

- Replaced the pull-request label release-intent system with a Conventional Commits flow driven by Commitizen, so release versioning and changelog entries now derive from commit history. Removed the `verify-pr-release-intent` command and its CI label enforcement, and added an advisory `check-runtime-deps` command that flags shipped CLI dependency changes without blocking merges.

### Fixed

- Set the docs site Vite build target to ES2022 so documentation builds stay compatible with esbuild 0.28, which now rejects destructuring transforms for older browser targets.

## [0.11.4] - 2026-07-01

### Changed

- Refreshed shipped runtime dependencies through a `Cargo.lock` update (including `memchr` to 2.8.2) and dropped unused build-time transitive crates.
- Updated repository tooling to current versions: GitHub Actions, pre-commit hooks, Python dev dependencies, and the docs Node toolchain.
- Bumped the pinned Rust toolchain to 1.96.1.

### Security

- Hardened supply-chain posture by clearing all `cargo audit` and `npm audit` advisories: removed `anyhow` (RUSTSEC-2026-0190) from the dependency graph and bumped the documentation toolchain to patched `vite` 6.4.3 and `esbuild` 0.28.1.

## [0.11.3] - 2026-06-07

### Fixed

- Fixed a CI issue where setup-rust did not activate the pinned Rust toolchain globally.

## [0.11.2] - 2026-06-07

### Changed

- Updated shipped runtime dependencies, including `serde_json` to 1.0.150 and `tar` to 0.4.46.
- Bumped the pinned Rust toolchain to 1.96.0 for local and CI builds.

## [0.11.1] - 2026-04-27

### Changed

- Updated the shipped `clap` dependency to 4.6.1.

## [0.11.0] - 2026-04-08

### Added

- Added terminal styling in the text reporter.

## [0.10.4] - 2026-04-08

### Fixed

- Corrected a missing checkout step in the CI workflow.

## [0.10.3] - 2026-04-08

### Fixed

- Updated CI to use the newer macOS 15 runner image.

## [0.10.2] - 2026-04-08

### Fixed

- Published platform-specific wheels so installs no longer fall back to source distribution builds on unsupported platforms.

## [0.10.1] - 2026-04-08

### Changed

- Bumped the shared workspace version to publish the `tq-rules` rule documentation API consumed by `tq-docsgen`.

## [0.10.0] - 2026-04-07

### Added

- Added `--fail-on <SEVERITY>` CLI flag and `fail_on` config key to configure the minimum severity that triggers a nonzero exit code.
- Added `--severity <RULE_ID=SEVERITY>` CLI flag and `severity_overrides` config key to override the severity of specific built-in rules. Applies at both the global `[tool.tq]` level and per `[[tool.tq.targets]]` entry.

## [0.9.1] - 2026-04-07

### Removed

- Removed the `tq` pre-commit hook because it required an installed Rust toolchain. Opened an issue to create a dedicated pre-commit wrapper repository for `tq` so downstream users can install the shared `tq` hook from published Python artifacts rather than from the Rust source repository.

## [0.9.0] - 2026-04-07

### Added

- Added a pre-commit hook for `tq`.

### Fixed

- Fixed packaging so publishable Linux wheels can be built correctly.

## [0.8.1] - 2026-04-07

### Fixed

- Included `LICENSE` in the source distribution.

## [0.8.0] - 2026-04-07

### Changed

- Hardened read-only workflows by disabling checkout credential persistence.
- Added CodeQL JavaScript/TypeScript analysis for `docs/.vitepress`.

### Security

- Moved the docs site toolchain off vulnerable Vite releases by pinning VitePress's transitive `vite` dependency to 6.4.2.

## [0.7.1] - 2026-04-06

### Fixed

- Fixed a stale config key in the release smoke fixture.

## [0.7.0] - 2026-04-06

### Added

- Added a Rust workspace that implements the `tq` runtime.
- Added a Rust-backed PyPI distribution for `tqlint`, preserving the `tq` CLI entrypoint.
- Added stricter Rust-side contract and conformance coverage.

### Changed

- Rewrote the shipped implementation from the original Python release to a Rust-first architecture.
- Moved documentation, packaging, and publish automation to Rust-first workflows.
- Expanded CI and maintenance automation for the Rust toolchain, pinned maintenance, dependency drift, and security.

### Fixed

- Fixed relative-path handling and path display errors, including platform-specific path-prefix edge cases.
- Fixed release and dependency-policy validation to fail closed when Dependabot manifests or built-in identifiers are invalid.

### Security

- Removed the generic YAML parser surface and tightened dependency and release-policy validation.

## [0.6.3] - 2026-03-04

### Fixed

- `tq` now resolves target roots from the config file directory instead of the current working directory.

## [0.6.2] - 2026-03-03

### Added

- Introduced strict multi-target configuration model; `[[tool.tq.targets]]` is now required with explicit source/test roots.

### Changed

- Refactored analysis engine to execute and aggregate across multiple targets by default.
- Updated CLI to support `--target` filtering and validate unknown target names.
- Migrated documentation and examples to use the new target-based schema.

## [0.5.0] - 2026-03-03

### Added

- Added a VitePress documentation site.

### Changed

- Publish workflow now creates GitHub Releases for pushed SemVer tags with autogenerated notes and attached artifacts.

## [0.4.2] - 2026-03-03

### Fixed

- Added GH_TOKEN environment variable for artifact verification.

## [0.4.1] - 2026-03-03

### Changed

- Hardened publish smoke checks to resolve explicit built artifact paths.
- Added maintainer guidance for `workflow_dispatch` dry-run validation.

## [0.4.0] - 2026-03-02

### Added

- Added CI workflows and policies for PyPI trusted publishing.

### Changed

- Prepared for public release by updating documentation and configuration.

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

- Added `tq check` as the CLI entrypoint with deterministic output, strict exit semantics, and rule selection controls.
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
