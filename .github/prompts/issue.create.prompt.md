---
name: issue-create
description: Create a high-quality GitHub issue for this repository
argument-hint: Describe the problem, goal, and any known acceptance criteria
agent: agent
---

Create high-quality GitHub issues for this repository using the GitHub MCP server tools.

## Guidelines

- Do not invent facts. If information is missing, ask for it.
- Titles should describe the work/problem, not categorize it.
  - Prefer imperative, verb-first phrasing ("Add …", "Fix …", "Implement …", etc.).
  - Avoid category prefixes in titles (e.g., "Epic:", "Feature:", "Bug:", etc.).
- Prefer existing labels.
  - Only apply labels you can confirm exist (e.g., with `#tool:github/get_label`). If unsure, omit labels rather than guessing.
- Prefer crisp verification criteria.
  - Every issue should have acceptance criteria that can be checked as done.
  - Bugs must include clear reproduction steps and expected vs actual behavior.
- Keep it safe.
  - Redact tokens, cookies, API keys, private URLs, wallet addresses, and any identifying data unless explicitly required.

## Workflow

1. Determine repo coordinates (`owner`, `repo`).
2. Check for duplicates before creating a new issue.
3. Choose labels and type conservatively.
4. Draft the issue body in `tmp/issues/issue-<timestamp>.md`.
5. Create the issue.
6. If this is an epic, attach sub-issues.
   - Create the sub-issues first, then connect them.

## Tool Call

Use `#tool:github/issue_write`.

- Create issue files at `tmp/issues/issue-<timestamp>.md` first.
  - Do not hard wrap the issue body.
  - Use the file content as the `body` field.
