---
name: docs-sweep
description: Audit a docs subtree and remove or refactor stale documents
argument-hint: Folder path to sweep and any scope constraints
agent: agent
---

Sweep a docs folder and remove or refactor stale documents.

## Context

- Documentation guidelines: [Documentation](../../docs/developer/standards/docs.md)
- Refactor standards: [Refactor Prompt](./refactor.prompt.md)

## Requirements

- Treat design and plan docs as temporary unless they still serve active work.
- Audit each doc against the current codebase.
- Keep one enduring reference doc per topic.
- Prefer consolidating durable content over duplicating it.

## Procedure

1. Enumerate all docs under `[folder_path]` (recursive).
2. For each file:
   - Extract claims and implied contracts.
   - Classify the file as **keep**, **refactor**, or **remove**.
3. Consolidate overlapping topics into the best long-term destination.
4. Delete documents that should not remain.
5. Return a short sweep report listing files kept, refactored, and removed.
