---
name: task-audit
description: Audit an implementation against canonical requirements and fix straightforward gaps
argument-hint: Describe the implementation to audit or provide the issue to check against
agent: agent
---

Your task is to audit the implementation described by the user.

Use the user's description as the starting point. If the user identifies a GitHub issue, fetch it first with `#tool:github/issue_read` and treat the issue as the canonical requirements source. Otherwise, treat the user's description as canonical. Gather any supporting codebase context or resources you need to complete the work; you may use `#tool:web/fetch` where required.

Fetch the relevant details and codebase context. Do not assume the implementation or the task description is correct. Review the code against the real requirements and fix clear problems directly when that is the fastest path.

## Audit Guidelines

- MUST cross-check implementation against the canonical requirements source
- MUST NOT assume existing architecture or code is correct
- MUST FLAG backward compatibility or legacy code
- MUST FLAG code or tests that re-introduces old architecture
- MUST FLAG missing or partially implemented requirements
- MUST FLAG logical inconsistencies or design flaws
- MUST FLAG violations of software engineering best practices and design principles (SOLID, etc.)
- MUST FLAG violations of the [Testing Quality Standards](../../docs/developer/standards/tests.md)

## Remediation Guidelines

If fixing issues is straightforward, make the changes directly. For larger problems, produce an audit report with the gaps and recommended improvements or create follow-up issues.

Refactors should follow the [Refactor Prompt](./refactor.prompt.md) guidelines.

Do what is required to achieve clean, maintainable, and elegant code that aligns with the project direction.
