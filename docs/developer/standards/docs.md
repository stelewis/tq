# Documentation Standards

This project treats documentation as a first-class part of the codebase.

The goal is to keep docs:

- **Useful**: answer a concrete question someone will have.
- **Non-redundant**: one canonical place per concept.
- **Stable**: describe contracts and workflows, not incidental details.
- **Small**: minimal surface area to reduce drift and improve maintainability.
- **Concise**: get to the point quickly without unnecessary exposition.
- **Purposeful**: avoid “docs for docs sake” especially where code is the source of truth.
- **Correct**: avoid drift and outdated information; revisit and update regularly.

We want clean, maintainable docs that achieve the same design excellence as the code itself.

## Where Things Live

- **Developer docs** ([docs/developer](../README.md)):
  - How to use, extend, and contribute to the codebase.
- **ADRs** ([docs/adr](../../adr/README.md)):
  - Architectural Decision Records capturing key design decisions.
- **Design docs** ([docs/design](https://github.com/stelewis/tq/tree/main/docs/design)) - *ephemeral*:
  - Design rationale, higher order vision and context.
- **Plan docs** ([docs/plans](https://github.com/stelewis/tq/tree/main/docs/plans)) - *ephemeral*:
  - Implementation plans and specifications for features.

*Note:* Design and plan docs are temporary artifacts used during the development phase. As features are implemented, the documents in these folders should be refactored into enduring developer docs or removed.

## Rules For Enduring Developer Docs

### Prefer one canonical reference

- Choose one document to be the canonical reference for a topic.
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
