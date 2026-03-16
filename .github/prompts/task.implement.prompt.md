---
agent: agent
---

Implement the issue or task specified by the user.

Fetch the relevant details, codebase context, and references you need. Do not assume the spec or existing code is correct. Build the right solution for the project, not the smallest change that preserves outdated structure.

## Large Issues

If the task is too large for one pass, split it into smaller sub-tasks, create follow-up issues when useful, and report the remaining plan clearly.

## Implementation

- MUST NOT assume existing architecture or code is correct
- MUST NOT maintain backward compatibility or legacy code
- MUST NOT implement code or tests that re-introduces old architecture
- MUST satisfy all requirements specified in the issue
- MUST strive for software engineering best practices and design principles (SOLID, etc.)
- MUST strive for architectural excellence even if it requires significant changes
- MUST adhere to the [Testing Quality Standards](../../docs/developer/standards/tests.md)
- MUST FLAG logical inconsistencies or design flaws
