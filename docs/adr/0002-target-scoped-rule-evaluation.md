---
id: 0002
title: "Scope rule evaluation to explicit target package paths"
status: accepted
date: 2026-03-04
tags:
  - rules
  - configuration
  - targets
supersedes: null
superseded_by: null
---

## Context

`tq` moved from single-scope configuration to `[[tool.tq.targets]]`. In a multi-target project, all targets can share one `test_root` (for example, `tests/`), while each target owns a distinct package path under that root.

Without explicit target-scoped path semantics, rules can produce cross-target noise. In particular, structure checks can report files from sibling targets as misplaced for the active target.

## Decision

Rule evaluation is target-scoped and uses explicit package-path context.

- The CLI composition root passes target context into `AnalysisContext` settings, including:
  - `package_path` for the active target
  - `known_target_package_paths` for all configured targets
  - `test_root_display` for user-facing suggestion paths
- Rules that reason about source-to-test layout use the explicit target package path from context, not inferred single-segment package names.
- `structure-mismatch` excludes files that belong to sibling configured target roots.
- Structure suggestions include the configured test root prefix so the destination path is unambiguous.

## Consequences

- Multi-target runs stay deterministic and low-noise.
- Rule behavior is aligned with target boundaries defined in config.
- Suggestions are clearer for users by including the test-root prefix.
- New path-aware rules must follow target-scoped context semantics.

## Alternatives considered

### Infer package scope from source root name

Rejected. This only works for single-segment package roots and fails for multi-target or nested package paths.

### Split tests physically by target-specific roots only

Rejected as a requirement. Projects may intentionally share one top-level `test_root` for all unit tests.

### Keep global scan behavior and filter in reporting

Rejected. Filtering after evaluation still allows cross-target rule logic and increases coupling between rule internals and reporters.

## Related

- [Define tq CLI and configuration contract](./0001-tq-cli-config-contract.md)
- [Configuration reference](../reference/configuration.md)
- [Structure mismatch rule](../reference/rules/structure-mismatch.md)
