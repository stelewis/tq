# Versioning

Policy for versioning user-visible `tq` contracts.

## Contract surfaces

This policy covers:

- CLI flags and behavior (`tq check`)
- Configuration keys and validation semantics (`[tool.tq]`)
- Rule IDs and default severities
- Exit code semantics
- JSON diagnostic schema fields

## Versioning model

`tq` is pre-`1.0`. Until `1.0`, the project uses this compatibility model:

- `patch` (`0.x.Y`): non-breaking fixes and additive changes that do not change the meaning of existing contracts
- `minor` (`0.X.0`): contract-impacting changes, including intentional breaking changes

After `1.0`, major-version SemVer semantics apply for breaking changes.

## Internal workspace crate policy

The Rust workspace uses one shared version for all internal crates.

- Internal crate APIs are current-only. They are not compatibility surfaces.
- When a workspace crate changes a public API consumed by another workspace crate, update all internal callers in the same change.
- Do not preserve old internal APIs with shims, aliases, deprecated wrappers, or dual-path call sites.
- If an internal public API changes, bump the shared workspace version as a minor pre-`1.0` release before packaging or release validation.
- Version bumps must keep `workspace.package.version` and internal `workspace.dependencies.tq-*` version fields aligned. Commitizen updates the inline `workspace.dependencies.tq-*` entries in `Cargo.toml` during `cz bump`.
- `cargo run -p tq-release --locked -- verify-release-policy --repo-root .` is the release-policy gate for this policy. It must pass before packaging.
- `cargo package --workspace --locked` is the packaging gate for this policy. If it fails because a published crate with the same version no longer matches the current internal API, the correct fix is a version bump, not compatibility code.

## Change classification

### Patch changes

Choose a patch release for:

- bug fixes that preserve documented contract intent
- additive optional configuration keys that are inert by default
- documentation clarifications with no runtime contract change
- rule metadata updates that do not alter rule IDs, default severities, or trigger intent

### Minor changes

Choose a minor release for:

- adding a new stable rule enabled by default behavior
- changing a stable rule's trigger intent or widening behavior beyond bug-fix scope
- changing a stable rule default severity
- changing exit code semantics
- removing or renaming CLI flags, configuration keys, or rule IDs
- changing JSON output fields in a breaking way
- changing a public API in any internal workspace crate used by another workspace crate

## Governance linkage

Rule/severity procedural requirements and reference ownership/review live in [governance.md](./governance.md).

## Contract stability commitments

- Rule IDs are stable once published.
- Severity vocabulary is stable: `error`, `warning`, `info`.
- Exit code `2` remains reserved for invalid CLI/configuration and runtime IO setup errors.
- If a contract changes, docs and runtime must land together in the same PR.
