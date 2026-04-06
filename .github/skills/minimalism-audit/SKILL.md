---
name: minimalism-audit
description: Audits repositories for context bloat and unnecessary explanatory surface across copilot instructions, file instructions, skills, prompts, agents, MCP definitions, docs, templates, comments, and docstrings. Use when reducing context tax, pruning duplicate guidance, tightening broad scopes, or fixing stale verbose agent tooling.
argument-hint: describe the repo or change scope, the surfaces to prioritize, and whether you want findings only or direct fixes too
user-invocable: true
disable-model-invocation: false
---

# Minimalism Audit

Use this skill to reduce context tax and maintenance surface across documentation, agent tooling, and explanatory code surface.

## Problem

Models are strong at inferring from code, names, schemas, and engineering patterns. The problem this skill addresses is excess and redundant surface:

- always-on and high-frequency text consumes shared context that should stay available for the actual task
- verbosity and duplication increase drift, refactor surface, and loss of single responsibility
- large, repetitive projects are harder to comprehend, evolve, secure, and keep correct
- the paradigm has shifted and users can ask targeted questions themselves; durable docs should preserve contracts and non-obvious context rather than explain everything

## When To Use It

- The user asks for a minimalism pass, docs pruning, prompt cleanup, or context-window reduction.
- The task involves instructions, skills, prompts, agents, MCP definitions, templates, or documentation that may have grown noisy or duplicative.
- The codebase shows broad globs, repeated guidance, stale examples, over-explained comments or docstrings, or multiple files that answer the same question.

## Workflow

1. Define the audit target. Decide whether the task is repo-wide, focused on a change, or limited to specific surfaces.
2. Read local policy first. Prefer repository standards over generic heuristics:
   - docs/developer/standards/docs.md
   - docs/developer/standards/code.md
3. Inventory the surface with [references/audit-surfaces.md](references/audit-surfaces.md). Prioritize always-loaded or high-frequency context before lower-cost surfaces.
4. Rank each issue by cost and blast radius. Favor findings that affect every request, many requests, or many mirrored files.
5. For each issue, choose the optimal durable fix: delete, consolidate, tighten scope, move detail into a reference, or keep as-is with a clear reason.
6. Fix direct problems when the change is small, local, and clearly correct. For larger work, produce a ranked remediation plan with [references/report-template.md](references/report-template.md).

## Branching Rules

- If a surface is always loaded or discovered on most requests, apply a stricter budget. Every sentence must earn its place.
- If multiple files describe the same contract, keep one canonical owner and remove or collapse the rest.
- If docs or docstrings only restate code, types, filenames, or obvious navigation, shorten or delete them.
- If a workflow needs detail, keep the entrypoint small and move specifics into a referenced file instead of expanding the top-level instruction.

## Remediation Guides

Choose the playbook that matches the dominant smell. Read multiple files only when the task spans multiple problem types.

- Prioritized audit checklist: [references/audit-surfaces.md](references/audit-surfaces.md)
- Consolidation and pruning patterns: [references/remediation-patterns.md](references/remediation-patterns.md)
- Condensing content and writing for fast comprehension: [references/condensing-content.md](references/condensing-content.md)
- Output structure for findings and fixes: [references/report-template.md](references/report-template.md)

## Execution Rules

- Treat context as a scarce shared runtime budget, not free documentation space.
- Models are already very smart; add context only when it is non-obvious, or hard to infer from code and structure.
- Prefer deletion over relocation when content has no durable owner.
- Prefer modularity across docs, prompts, and instructions that maintains single responsibility.
- Prefer self-documenting names, schemas, types, and structure over explanatory documentation that merely restates them.
- Keep `copilot-instructions.md` and skill descriptions especially tight because they are high-signal discovery surfaces.
- Use narrow `applyTo` globs. Avoid `**` unless the rule truly applies across the repository.
- Keep prompts task-specific. Merge or delete near-duplicate prompt families.
- Keep examples minimal and durable. Remove issue numbers, transient state, and trivia that code or tools already reveal.
- Treat MCP definitions, hooks, agents, and external integrations as both context-budget and supply-chain decisions.
- Fix root causes and update affected docs, tests, or templates together when the audit results in code changes.

## Required Output

- Scope and assumptions.
- Findings ordered by priority with concrete evidence.
- The fix applied for each issue fixed immediately.
- A ranked follow-up sequence for anything not fixed in the current change.
- Validation status and residual risks.

## Completion Bar

- High-cost surfaces were reviewed first and either fixed or explicitly justified.
- Findings are non-duplicative and tied to a clear removal, consolidation, or scoping action.
- The result reduces maintenance or context surface instead of moving verbosity sideways.
- The response states what was changed, what was left alone, and why.
