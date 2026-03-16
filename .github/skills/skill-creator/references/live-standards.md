# Live Standards

Use this file when creating or updating a skill that needs to stay aligned with current Agent Skills or VS Code behavior.

## Why This File Exists

Agent Skills guidance evolves. Frontmatter fields, discovery behavior, activation patterns, and VS Code-specific support can drift over time. Keep the canonical URLs here so agents can refresh against the live docs instead of relying only on bundled summaries.

## Canonical URLs

- Agent Skills specification: <https://agentskills.io/specification>
- Agent Skills client implementation guide: <https://agentskills.io/client-implementation/adding-skills-support>
- VS Code Agent Skills documentation: <https://code.visualstudio.com/docs/copilot/customization/agent-skills>

## When To Fetch

Fetch the relevant URLs when:

- The task involves SKILL.md frontmatter, directory layout, packaging, or validation rules.
- The task depends on discovery or activation behavior.
- The task needs VS Code-specific fields such as `argument-hint`, `user-invocable`, or `disable-model-invocation`.
- The user asks for latest guidance, best practices, or cross-client compatibility.

## Suggested User Nudge

When useful, suggest taking action like:

- "Before drafting the skill, would you like me to check the latest Agent Skills spec and related documentation?"
