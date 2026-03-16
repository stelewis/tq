---
agent: agent
---

Your task is to audit the implementation of the task/issue specified by the user. Fetch any relevant details, codebase context, and any references you require.

Do not assume that the implementation is correct. Your goal is to carefully review the changes made to ensure they fully satisfy the requirements of the issue. Think through design intent and context. You are empowered to make source code changes to achieve the goals of the issue and the project, including removing modules and tests entirely and starting files afresh as needed. Do not be constrained by what already exists or by a desire to avoid breaking changes. Think as if you were building anew. If you detect areas of improvement, logical flaws, or other issues, please address them.

## Audit Guidelines

- **MUST** cross-check implementation against issue requirements
- **MUST NOT** assume existing architecture or code is correct
- **MUST FLAG** backward compatibility or legacy code
- **MUST FLAG** code or tests that re-introduces old architecture
- **MUST FLAG** missing or partially implemented requirements
- **MUST FLAG** logical inconsistencies or design flaws
- **MUST FLAG** violations of software engineering best practices and design principles (SOLID, etc.)
- **MUST FLAG** violations of the [Testing Quality Standards](../../docs/developer/standards/tests.md)

## Remediation Guidelines

If fixing issues is straightforward, make the changes directly. For larger problems, either produce an audit report detailing the gaps and recommended improvements or create new issues to track the necessary work.

Refactors should follow the [Refactor Prompt](./refactor.prompt.md) guidelines.

I recognize this is a big task, but it is absolutely critical that we do it right to ensure we have clean, maintainable, and elegant code that aligns with our vision of the future.
