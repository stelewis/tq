---
agent: agent
---

Your task is to audit a documentation file and decide whether it should be kept, refactored into developer docs, or removed.

## Context

- Documentation guidelines: [Documentation](../../docs/developer/standards/docs.md)
- Refactor standards: [Refactor Prompt](./refactor.prompt.md)

## User Input

User will provide a documentation file to audit: `[file_name]`

## Requirements

- MUST audit the doc against the actual codebase (no stale claims).
- MUST identify whether the doc contains:
  - **Outstanding work** (items not yet implemented)
  - **Enduring value** worth keeping (contracts, workflows, durable guidance)
  - **Implementation trivia** that should be removed
- MUST avoid “docs for docs sake”; prefer code as the source of truth.
- MUST NOT add backward-compat/legacy adapters while refactoring; fix boundaries.

## Actions

1. Read the file and extract its concrete claims, TODOs, and “plan” items.
2. Audit those items against the codebase:
   - Confirm what exists, what’s missing, and what’s obsolete.
3. Decide:
   - **Keep**: if it is already durable and non-redundant.
   - **Refactor**: move enduring content into the best developer doc(s); delete the original if it becomes redundant.
   - **Remove**: if implemented, redundant, or purely historical.
4. If removing, delete it via `rm`.
