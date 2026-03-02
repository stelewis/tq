---
agent: agent
---

Your task is to sweep a docs folder and remove or refactor stale documents so the
codebase stays clean and maintainable.

## Context

- Documentation guidelines: [Documentation](../../docs/developer/standards/docs.md)
- Refactor standards: [Refactor Prompt](./refactor.prompt.md)

## User Input

User will provide a folder to sweep: `[folder_path]` (example: `docs/design` or `docs/implementation`)

## Goals

- Reduce doc surface area and drift.
- Keep only durable, canonical docs (contracts/workflows), not temporary plans.
- Prefer refactoring into developer docs over duplicating explanations.

## Requirements

- MUST treat design/implementation docs as temporary unless still actively needed.
- MUST audit each doc against the actual codebase (no stale claims).
- MUST avoid “docs for docs sake”; keep only enduring value.
- MUST keep one canonical reference per topic; others should link or be removed.
- MUST NOT add backward-compat/legacy adapters while refactoring; fix boundaries.

## Procedure

1. Enumerate all docs under `[folder_path]` (recursive).
2. For each file:
   - Extract concrete claims/TODOs and any implied contracts.
   - Audit against the codebase to classify: **implemented**, **obsolete**, **still pending**, **unclear**.
   - Decide: **keep**, **refactor**, or **remove**.
3. Consolidate:
   - If multiple docs cover one topic, pick a canonical destination in `docs/developer/`.
   - Refactor durable content into that destination and delete redundant sources.
4. Remove:
   - Delete documents that are implemented, redundant, obsolete, or purely historical via `rm`.
5. Output:
   - A short sweep report listing files kept/refactored/removed and the new canonical doc links.
