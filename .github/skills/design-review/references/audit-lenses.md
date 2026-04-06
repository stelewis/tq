# Audit Lenses

Use these lenses to review design quality across code and adjacent automation instead of focusing only on syntax or lint noise.

## Prioritization Rule

Rank surfaces in this order unless the user narrows scope:

1. Core code paths and ownership boundaries.
2. Tests that define or obscure contracts.
3. Configuration, workflows, and automation that shape behavior.
4. Docs that preserve non-obvious contracts.

## Code Structure

- Mixed responsibilities inside one module, class, or function.
- Indirection layers that only rename or forward calls.
- Hidden dependencies passed through globals, environment reads, or import-time side effects.
- Boolean or string switches that encode multiple modes without a real type or boundary.
- Compatibility wrappers or dead branches that no longer protect a real contract.

## Boundaries And Data Flow

- Parsing, validation, normalization, and defaulting happening deep in core logic.
- Best-effort handling that accepts malformed or partial input as valid.
- Error handling that hides failure origin or mixes domain and transport concerns.
- Side effects in constructors, imports, or helpers that should stay at composition edges.

## Tests And Contracts

- Tests that duplicate implementation detail instead of asserting behavior.
- Fixtures or helpers that bypass real boundaries and make invalid states easy to create.
- Stale tests protecting deleted behavior or obsolete compatibility paths.
- Missing coverage for the simplified contract after a refactor.

## Tooling And Automation

- Workflow steps or scripts with unclear ownership, duplicated logic, or silent fallbacks.
- Configuration files that expose options no supported workflow actually uses.
- Build, CI, or release paths that preserve legacy branches instead of a single clear path.

## Documentation Surface

- Docs that hide the real contract by describing the old implementation.
- Architecture text that names files and trivia instead of boundaries and ownership.
- Missing durable docs after a refactor that changes how a subsystem should be extended.

## Reporting Rule

For each in-scope surface, either record a finding or state why it passed. Do not silently skip a high-impact boundary.
