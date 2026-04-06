# Design Review Skill

This skill reviews repositories and changes for simplicity, strict boundaries, maintainability, and architectural clarity.

It is designed for tasks where the problem is implementation or system design quality, not just verbose docs or explicit security risk.

Clean design is an asset: smaller, stricter systems are easier to understand, evolve, secure, and maintain.

The Agent Skills specification requires `SKILL.md` and allows additional files and directories, so this README exists as human-facing guidance while [SKILL.md](./SKILL.md) remains the machine-loaded entrypoint.

## What It Produces

- a scoped design and maintainability review
- findings ordered by architectural and maintenance impact
- direct fixes for small, clearly correct cleanup work
- a phased remediation plan for larger refactors

## Example Prompts

- `/design-review review this repository for unnecessary abstraction, hidden coupling, and weak boundaries, then rank the cleanup work.`
- `/design-review simplify this PR's implementation and tooling changes without changing the intended contract.`
- `/design-review harden this subsystem for clarity and maintainability; fix straightforward issues and plan the rest.`

## References

- workflow and activation guidance: [SKILL.md](./SKILL.md)
- audit checklist by surface: [references/audit-lenses.md](./references/audit-lenses.md)
- remediation playbooks: [references/remediation-patterns.md](./references/remediation-patterns.md)
- output structure: [references/report-template.md](./references/report-template.md)
