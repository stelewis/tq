# Remediation Patterns

Use these patterns to fix minimalism issues at the root instead of trimming words without changing the underlying surface area.

## Duplicate Guidance

Smell:

- two or more files describe the same contract, workflow, or policy

Preferred fix:

- keep one canonical owner
- delete duplicates when possible
- otherwise replace duplicates with a short pointer only if the boundary is non-obvious

## Broad Always-On Instructions

Smell:

- root instructions expand to include task-specific workflows, examples, or repeated philosophy
- file instructions use `**` or very broad globs without a strong reason

Preferred fix:

- keep always-on files limited to repository-wide rules
- move task workflows into a skill or prompt
- narrow `applyTo` to the smallest durable file set

## Bloated Skill Entry Points

Smell:

- `SKILL.md` contains long examples, deep explanations, or many subdomains at once

Preferred fix:

- keep `SKILL.md` as the dispatcher
- move checklists, examples, and specialty guidance into one-level-deep references
- shorten metadata to what discovery actually needs

## Prompt Proliferation

Smell:

- multiple prompts differ only by a noun swap or a small wording change

Preferred fix:

- merge prompts around a stable workflow
- use parameters or attached context instead of cloning prompt files
- delete prompts that no longer represent an active workflow

## Over-Linked Docs

Smell:

- pages link to every related file even when the navigation tree is obvious

Preferred fix:

- keep only links that save real search effort or identify the canonical owner
- remove decorative or reflexive cross-links

## Docstrings That Restate Code

Smell:

- docstrings repeat names, types, or obvious control flow already visible in the code

Preferred fix:

- delete the docstring if no non-obvious context remains
- otherwise keep only contract details, invariants, error behavior, units, or side effects

## Template Drift

Smell:

- mirrored docs and instructions repeat the same guidance and drift apart over time

Preferred fix:

- reduce mirrored wording where tooling or generation allows
- when duplication is unavoidable, keep the duplicated text short and durable
- update both sides in one change and verify they still agree

## Surface Without Ownership

Smell:

- a file exists because it was once useful, but no workflow now depends on it and no owner keeps it current

Preferred fix:

- delete the file
- if the information still matters, fold the durable part into the canonical doc that already owns that topic

## Validation Loop

After each cleanup pass:

1. Re-read the edited surface and ask whether the main task still works with less text.
2. Check that no canonical guidance was lost.
3. Verify related tests, docs, or generation outputs if the cleanup changed behavior.
4. Stop when the surface is simpler, not merely rearranged.
