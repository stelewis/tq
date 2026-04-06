# Minimalism Audit Skill

This skill audits a repository for unnecessary context tax and maintenance surface, then fixes straightforward problems or produces a ranked cleanup plan.

It is designed for codebases that use agent tooling, prompts, instructions, docs, and templates where verbosity or duplication can quietly accumulate.

This README exists as human-facing guidance while [SKILL.md](./SKILL.md) exists as the machine-loaded entrypoint.

## What It Produces

- a prioritized minimalism audit
- findings ranked by context cost and maintenance impact
- direct fixes for small, clear cleanup work
- a phased remediation plan for larger consolidation work

## Example Prompts

- `/minimalism-audit audit this repository for context bloat across instructions, skills, prompts, and docs, then rank the cleanup work.`
- `/minimalism-audit prune our agent customizations and fix the obvious duplication, stale examples, and broad scopes.`
- `/minimalism-audit review this PR for unnecessary docs, prompt sprawl, and always-on instruction growth; fix small issues and plan the rest.`

## References

- [SKILL.md](./SKILL.md)
- [Audit surfaces](./references/audit-surfaces.md)
- [Remediation patterns](./references/remediation-patterns.md)
- [Condensing content](./references/condensing-content.md)
- [Report template](./references/report-template.md)
