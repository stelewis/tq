# Audit Surfaces

Use this checklist to review the highest-cost surfaces first instead of spending time on low-impact cleanup while always-on bloat remains.

## Prioritization Rule

Rank surfaces in this order unless the user narrows scope:

1. Always-loaded metadata and instructions.
2. Frequently attached or discovered agent customizations.
3. Frequently referenced docs or templates.
4. Code comments and docstrings.

## Always-Loaded Surfaces

- `copilot-instructions.md`, `AGENTS.md`, and equivalent root instructions:
  - remove generic explanations the model already knows
  - collapse overlapping rules
  - keep only durable repository policy, architecture, and constraints
  - optimize for model consumption not human comprehension
- Skill metadata:
  - tighten `description` because every skill description is discovery tax
  - check whether multiple skills overlap enough to merge or rename
- File instructions:
  - tighten `applyTo` globs
  - remove rules that belong in always-on policy or a task skill instead
  - avoid language that repeats formatter, linter, or compiler output

## High-Frequency Agent Surfaces

- `SKILL.md` bodies:
  - keep the entrypoint short
  - move large checklists and examples into referenced files
  - avoid deep reference chains
- `.github/prompts` and similar prompt directories:
  - merge near-duplicate prompts
  - delete prompts that only restate an obvious task name
  - keep templates short and remove narrative padding
- `.github/agents`:
  - check whether an agent really needs separate persona, tool restrictions, or handoffs
  - remove duplicated workflow text already owned by a skill or prompt
- MCP definitions, hooks, and tool configs:
  - justify each integration by value, trust cost, and context cost
  - remove unused or low-value surfaces

## Frequently Referenced Docs and Templates

- `docs/developer/standards`:
  - keep standards modular and single-responsibility
  - remove repeated principles when one standard already owns them
  - link only when the link saves real search effort
- `docs/developer/architecture.md`, `docs/developer/context.md`, and similar overview docs:
  - remove transient state, plans, and implementation trivia
  - keep durable boundaries, contracts, and decisions
- Mirrored docs or instructions:
  - check for duplicated guidance across multiple copies of the same contract
  - prefer a single durable source for shared guidance when tooling allows
- README and setup docs:
  - keep onboarding steps and stable contracts
  - remove essays, repeated rationale, and obvious navigation links

## Code Surface

- Module docstrings:
  - remove docstrings that only restate the module or package name
  - keep only non-obvious contracts or context
- Function and class docstrings:
  - remove parameter-by-parameter restatements of signatures and types
  - keep side effects, invariants, units, or behavior the code does not make clear
- Comments:
  - remove narration of obvious control flow
  - keep comments only when they preserve non-local reasoning

## Reporting Rule

For each surface, either record a finding or state why it passed. Do not silently skip always-on or high-frequency surfaces.
