# Risky Signature Defaults

Eliminate default values in function and method signatures that hide dependencies, mask missing inputs, or change semantics.

## Objective

- Make core contracts explicit so critical inputs are required.
- Inject dependencies from composition roots instead of creating them in default arguments.
- Remove mutable defaults.
- Update tests to match the new contracts.

## Non-Negotiables

- Do not instantiate objects in default arguments.
- Do not use mutable defaults such as [] or {}.
- Core or service code must not silently pick defaults for critical knobs, identifiers, or timeouts.
- Boundary layers such as CLI, config, or wiring code may provide defaults explicitly.

## What To Search For

- Dependency defaults such as client=...(), session=...(), or now=datetime.now().
- Mutable defaults such as items=[] or cache={}.
- Optional-by-default fallbacks where x: T | None = None becomes an implicit
  hidden default inside the function body.
- Critical parameter defaults such as empty strings, zeroes, or None for IDs, limits, risk knobs, or timeouts.

## Refactor Pattern

1. Inventory each risky signature and classify it as a dependency, mutable default, critical knob, or optional-by-default fallback.
2. Apply one fix:
   - Make the parameter required.
   - Inject the dependency from the composition root.
   - Move defaults into a config object created at the boundary.
   - Split into a strict core function and a boundary wrapper that supplies defaults.
3. Update all call sites. Do not keep legacy overloads alive.
4. Add or update tests so missing required inputs fail fast and hidden dependency instantiation cannot recur.
