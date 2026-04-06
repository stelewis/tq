# Remediation Patterns

Use these patterns to fix design problems at the root instead of wrapping them in more structure.

## Delete Hollow Abstractions

Smell:

- wrapper modules, helper classes, or indirection layers that only rename or forward behavior

Preferred fix:

- inline the layer
- move the remaining contract to the true owner
- update call sites and tests in one pass

## Split Mixed Responsibilities

Smell:

- one unit both parses input, performs domain logic, and executes side effects

Preferred fix:

- separate boundary parsing, core logic, and side effects
- keep each piece small enough to own one responsibility

## Push Defaults To Boundaries

Smell:

- core functions rely on hidden defaults, environment reads, or optional parameters that are really required

Preferred fix:

- make core inputs explicit
- set defaults only at CLI, config, HTTP, or composition boundaries

## Replace Mode Switches With Real Structure

Smell:

- strings, booleans, or flag combinations encode multiple behaviors in one flow

Preferred fix:

- use explicit types, separate functions, or distinct modules per behavior
- delete combinations that are not part of the supported contract

## Remove Compatibility Scaffolding

Smell:

- adapters, deprecated branches, or fallback paths preserved only because they already exist

Preferred fix:

- delete the legacy path
- update callers and tests to the current design
- document the new contract if extension points changed

## Tighten Tooling Ownership

Smell:

- multiple scripts or workflows implement the same task differently

Preferred fix:

- keep one canonical path
- delete duplicates and unused options
- make validation steps explicit in the surviving workflow

## Validation Loop

After each cleanup pass:

1. Re-read the changed surface and ask whether ownership and behavior are now easier to explain.
2. Verify tests cover the intended contract rather than the removed implementation detail.
3. Confirm docs and automation still describe the surviving path.
4. Stop when the design is simpler and stricter, not merely rearranged.
