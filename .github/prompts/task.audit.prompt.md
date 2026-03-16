---
agent: agent
---

Audit the implementation of the task or issue specified by the user.

Fetch the relevant details and codebase context. Do not assume the implementation or the task description is correct. Review the code against the real requirements and fix clear problems directly when that is the fastest path.

## Audit Guidelines

- Cross-check implementation against the requirements.
- Flag legacy compatibility code or architectural regressions.
- Flag missing requirements, design flaws, and testing-standard violations.
- Prefer direct fixes for straightforward gaps.

## Remediation Guidelines

If the gaps are larger, produce a short audit report or create follow-up issues. Use the [Refactor Prompt](./refactor.prompt.md) for larger structural changes.
