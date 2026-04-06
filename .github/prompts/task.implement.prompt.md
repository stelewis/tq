---
name: task-implement
description: Implement a task from the user description or a GitHub issue
argument-hint: Describe the task or provide the issue to implement
agent: agent
---

Your task is to implement the work described by the user.

Use the user's description as the starting point. If the user identifies a GitHub issue, fetch it first with `#tool:github/issue_read` and treat the issue as the canonical requirements source. Otherwise, treat the user's description as canonical. Gather any supporting codebase context or resources you need to complete the work; you may use `#tool:web/fetch` where required.

Fetch the relevant details, codebase context, and references you need. Do not assume the spec or existing code is correct. Build the right solution for the project, not the smallest change that preserves outdated structure.

## Large Issues

If the task is too large for one pass, split it into smaller sub-tasks, create follow-up issues when useful, and report the remaining plan clearly.

## Implementation

- MUST NOT assume existing architecture or code is correct
- MUST NOT maintain backward compatibility or legacy code
- MUST NOT implement code or tests that re-introduces old architecture

- MUST satisfy all requirements from the canonical requirements source
- MUST strive for software engineering best practices and design principles (SOLID, etc.)
- MUST strive for architectural excellence even if it requires significant changes
- MUST adhere to the [Testing Quality Standards](../../docs/developer/standards/tests.md)
- MUST FLAG logical inconsistencies or design flaws

Do what is required to achieve clean, maintainable, and elegant code that aligns with the project direction.
