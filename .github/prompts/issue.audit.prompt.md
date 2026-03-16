---
agent: agent
---

Audit the implementation of the GitHub issue specified by the user.

Fetch the issue and any needed references. Do not assume the implementation or the issue text is correct. Review the code against the real requirements and the project design, and fix clear problems directly when that is the fastest path.

## Audit Guidelines

- Cross-check implementation against the issue requirements.
- Flag legacy compatibility code or architectural regressions.
- Flag missing requirements, design flaws, and testing-standard violations.
- Prefer direct fixes for straightforward gaps.

## Remediation Guidelines

If the gaps are larger, produce a short audit report or create follow-up issues. Use the [Refactor Prompt](./refactor.prompt.md) for larger structural changes.
