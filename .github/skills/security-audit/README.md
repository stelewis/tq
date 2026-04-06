# Security Audit Skill

This skill provides an end-to-end security audit workflow for repositories and change sets.

It is designed to review the full trust surface, not just application code:

- source code and runtime boundaries
- configuration and manifests
- dependencies and lockfiles
- GitHub Actions and workflows
- hooks, build tools, and release tooling
- prompts, skills, agents, and MCP servers

The Agent Skills specification requires `SKILL.md` and allows additional files and directories, so this README exists as human-facing guidance while [SKILL.md](./SKILL.md) remains the machine-loaded skill entrypoint.

## What It Produces

- a scoped security audit
- findings ordered by severity with evidence
- concrete remediation guidance for each finding
- a phased remediation plan for issues that are not fixed immediately

## Example Prompts

- `/security-audit audit this repository for supply-chain and code security risk, then produce a ranked remediation plan.`
- `/security-audit review this PR for dependency, workflow, secret-handling, and prompt-injection risk.`
- `/security-audit harden our GitHub Actions, hooks, release tooling, and agent surfaces; fix straightforward issues and plan the rest.`

## References

- workflow and activation guidance: [SKILL.md](./SKILL.md)
- audit checklist by surface: [references/audit-surfaces.md](./references/audit-surfaces.md)
- output structure: [references/report-template.md](./references/report-template.md)
- live standards to fetch when needed: [references/live-standards.md](./references/live-standards.md)
