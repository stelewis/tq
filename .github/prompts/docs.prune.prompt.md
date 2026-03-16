---
agent: agent
---

Audit a documentation file and decide whether to keep it, refactor its durable content, or remove it.

## Context

- Documentation guidelines: [Documentation](../../docs/developer/standards/docs.md)
- Refactor standards: [Refactor Prompt](./refactor.prompt.md)

## Requirements

- Audit the doc against the current codebase.
- Separate durable guidance from plans, TODOs, and implementation trivia.
- Prefer one enduring reference doc per topic.
- Do not preserve legacy wording or compatibility baggage just because it already exists.

## Actions

1. Read the file and extract its concrete claims.
2. Check those claims against the codebase.
3. Decide:
   - **Keep** if the doc is durable and non-redundant.
   - **Refactor** if the durable content belongs in a better long-term doc.
   - **Remove** if the file is obsolete, redundant, or purely historical.
4. Delete files that should not remain.
