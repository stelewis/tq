# Versioning

Policy for versioning user-visible `tq` contracts.

## Contract surfaces

Contract surfaces covered by this policy:

- CLI flags and behavior (`tq check`)
- Configuration keys and validation semantics (`[tool.tq]`)
- Rule IDs and default severities
- Exit code semantics
- JSON diagnostic schema fields

## Versioning model

`tq` is currently pre-`1.0`. Until `1.0`, the project uses this compatibility model:

- `patch` (`0.x.Y`): non-breaking fixes and additive changes that do not change the meaning of existing contracts
- `minor` (`0.X.0`): contract-impacting changes, including intentional breaking changes

After `1.0`, major-version SemVer semantics apply for breaking changes.

## Change classification

### Patch changes

Use a patch release for:

- bug fixes that preserve documented contract intent
- additive optional configuration keys that are inert by default
- documentation clarifications with no runtime contract change
- rule metadata updates that do not alter rule IDs, default severities, or trigger intent

### Minor changes

Use a minor release for:

- adding a new stable rule enabled by default behavior
- changing a stable rule's trigger intent or widening behavior beyond bug-fix scope
- changing a stable rule default severity
- changing exit code semantics
- removing or renaming CLI flags, configuration keys, or rule IDs
- changing JSON output fields in a breaking way

## Governance linkage

Rule/severity procedural requirements and reference ownership/review live in [governance.md](./governance.md).

## Contract stability commitments

- Rule IDs are stable once published.
- Severity vocabulary is stable: `error`, `warning`, `info`.
- Exit code `2` remains reserved for invalid CLI/configuration and runtime IO setup errors.
- If a contract changes, docs and runtime must land together in the same PR.
