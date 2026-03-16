# Documentation Standards

Treat documentation as a first-class part of the codebase.

Keep docs:

- **Useful**: answer a concrete question someone will have.
- **Non-redundant**: one place per concept.
- **Stable**: describe contracts and workflows, not incidental details.
- **Small**: minimal surface area to reduce drift and improve maintainability.
- **Concise**: get to the point quickly without unnecessary exposition.
- **Purposeful**: avoid “docs for docs’ sake,” especially where code is the source of truth.
- **Correct**: avoid drift and outdated information; revisit and update regularly.

Aim for clean, maintainable docs with the same design quality as the code.

## Where Things Live

- **Developer docs** (`docs/developer`):
  - How to use, extend, and contribute to the codebase.
- **ADRs** (`docs/adr`):
  - Architectural Decision Records capturing key design decisions.
- **Design docs** (`docs/design`) - *ephemeral*:
  - Design rationale and vision/context.
- **Plan docs** (`docs/plans`) - *ephemeral*:
  - Implementation plans and specifications for features.

*Note:* Design and plan docs are temporary artifacts used during development. As features ship, refactor these docs into enduring developer docs or remove them.

## Rules For Enduring Developer Docs

### Prefer one reference doc per topic

- Choose one document to be the reference for a topic.
- Other docs should link to it rather than re-explaining it.
- Refactors and consolidations are encouraged.

### Document contracts, not implementation trivia

Avoid:

- Specific issue/PR numbers.
- Temporary sprint plans or “current state” snapshots (keep in docs/design).
- Enumerations of internal event names unless they are a public contract.

### Keep examples minimal and durable

- Use the smallest example that demonstrates the feature.
- Prefer examples that avoid internal-only types.
- When possible, prefer examples that are unlikely to change.

## Doc Quality Standards

### Avoid these anti-patterns

- **Duplicate explanations**: same topic described in multiple places.
- **Leaky layering**: docs exposing internal implementation details.
- **Brittle examples**: examples tied to unstable internal structures.
- **Over-documentation**: documenting obvious code-level details.
