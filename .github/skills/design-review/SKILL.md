---
name: design-review
description: Audits code, tests, and automation for simplicity, strict boundaries, maintainability, and architectural clarity. Use when reviewing or refactoring for smaller abstractions, cleaner ownership, deletion of dead or legacy paths, tighter tooling behavior, or overall design excellence beyond security-only or lint-only concerns.
argument-hint: describe the repo or change scope, the surfaces to prioritize, and whether you want findings only or direct fixes too
user-invocable: true
disable-model-invocation: false
---

# Design Review

Use this skill to review or refactor repositories for design quality rather than for security-only risk or prose-only minimalism.

## Problem

Clean design is an asset. Smaller, stricter systems are easier to understand, evolve, secure, and maintain.

This skill addresses design surface that makes code and tooling harder to reason about than necessary:

- unnecessary abstraction and indirection hide ownership and behavior
- mixed responsibilities increase coupling and make changes riskier
- hidden defaults, side effects, and fallback paths make failures harder to predict and debug
- simpler boundaries let both humans and models hold more of the system in working context and reason about it more accurately

## When To Use It

- The user asks for maintainability review, design cleanup, simplification, architectural hardening, or a quality pass focused on elegance and strictness.
- The task involves removing dead abstractions, tightening ownership boundaries, reducing hidden coupling, deleting compatibility scaffolding, or making tooling behavior more explicit.
- The change spans code, tests, configuration, workflows, or developer tooling and needs a single design-oriented audit.

## Workflow

1. Define the review target. Decide whether the task is repo-wide, limited to a change, or focused on specific subsystems.
2. Read local policy first. Prefer repository standards over generic heuristics:
   - docs/developer/standards/code.md
   - docs/developer/standards/docs.md if contracts or developer docs are affected
   - docs/developer/standards/security.md if trust boundaries or automation permissions are touched
3. Inventory the design surface with [references/audit-lenses.md](references/audit-lenses.md). Cover code, tests, configuration, workflows, and docs that carry contracts.
4. Rank issues by maintenance cost and blast radius. Favor findings that reduce complexity, collapse duplicate ownership, or eliminate fragile branches at the root.
5. Choose the durable fix: delete, split by responsibility, move parsing and defaults to boundaries, make invalid states unrepresentable, or tighten tooling behavior.
6. Fix direct problems when the change is small, local, and clearly correct. For larger work, produce a ranked remediation plan with [references/report-template.md](references/report-template.md).

## Branching Rules

- If the issue is mainly low-signal prose, prompt sprawl, or docstring bloat, use minimalism heuristics instead of expanding the design review.
- If the issue is primarily a vulnerability, trust-boundary mistake, secret exposure, or permissions problem, apply the security audit bar first.
- If a Python smell matches existing Python code-quality playbooks, use those patterns for the implementation details and keep this skill at the design level.
- If simplifying the design changes a public contract, make the contract explicit and update tests and docs together.

## References

- Design audit lenses by surface: [references/audit-lenses.md](references/audit-lenses.md)
- Root-cause remediation patterns: [references/remediation-patterns.md](references/remediation-patterns.md)
- Output structure for findings and fixes: [references/report-template.md](references/report-template.md)

## Execution Rules

- Prefer deletion over abstraction when the abstraction no longer earns its maintenance cost.
- Keep boundaries narrow: parse, validate, and normalize at edges.
- Keep side effects at the edges and core logic deterministic.
- Remove legacy compatibility paths, dead flags, and best-effort fallbacks instead of preserving them.
- Prefer explicit ownership, exact imports, and small modules over convenience barrels or mixed responsibilities.
- Minimize configuration surface and hidden coupling between code, tooling, and workflows.
- Update tests with source changes; do not leave stale assertions or parallel APIs behind.
- Keep the response focused on findings, applied fixes, follow-up work, and validation status.

## Required Output

- Scope and assumptions.
- Findings ordered by priority with concrete evidence.
- The fix applied for each issue fixed immediately.
- A ranked follow-up sequence for anything not fixed in the current change.
- Validation status and residual risks.

## Completion Bar

- Findings reduce complexity, ownership confusion, or hidden behavior rather than just restyling code.
- Recommendations favor root-cause design changes over wrappers, compatibility shims, or local patches.
- Code, tests, tooling, and docs remain aligned after the review.
- The result states what was changed, what was left alone, and why.
