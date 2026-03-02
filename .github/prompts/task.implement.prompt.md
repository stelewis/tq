---
agent: agent
---

Your task is to implement the issue / task specified by the user. Fetch any
relevant details, codebase context, and any references you require.

Do not assume the specification is correct. Your goal is to carefully read the
issue requirements and think through design intent and context. You are
empowered to make source code changes and architectural decisions to achieve
the goals of the issue and the project, including removing modules and tests
entirely and starting files afresh as needed. Do not be constrained by what
already exists or by a desire to avoid breaking changes. Think as if you were
building anew. If you detect areas of improvement, logical flaws, or other
issues, please address them.

## Large Issues

If implementing the entire issue in one go is too big or complex to do in a
single turn, break it down into smaller sub-tasks. Either create new issues
for each sub-task following the [Issue Creation Prompt](./issue.create.prompt.md)
guidelines or implement what you can and report back to the user with a plan
for completing the rest.

## Implementation

- MUST NOT assume existing architecture or code is correct
- MUST NOT maintain backward compatibility or legacy code
- MUST NOT implement code or tests that re-introduces old architecture

- MUST satisfy all requirements specified in the issue
- MUST strive for software engineering best practices and design principles (SOLID, etc.)
- MUST strive for architectural excellence even if it requires significant changes
- MUST adhere to the [Testing Quality Standards](../../docs/developer/standards/tests.md)

- MUST FLAG logical inconsistencies or design flaws

I recognize this is a big task, but it is absolutely critical that we do it
right to ensure we have clean, maintainable, and elegant code that aligns with
our vision of the future.
