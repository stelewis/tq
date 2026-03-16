---
agent: agent
---

# Issue Edit Prompt (Agent)

Edit GitHub issues for this repository using the GitHub MCP server tools.

## Workflow (MCP)

1. Determine repo coordinates (`owner`, `repo`).
2. Fetch the current issue state.
  - Use `#tool:github/issue_read`.
3. Decide the minimum necessary change.
  - Preserve any correct context; do not rewrite history or remove important details.
  - If the ask is ambiguous, ask focused questions rather than making assumptions.
4. Draft the updated body in a file.
  - Create `tmp/issues/issue-<timestamp>.md` first.
  - Do not hard wrap lines in the issue body.
5. Apply the edit.
  - Use `#tool:github/issue_write`.
  - Update only what’s needed: `title`, `body`, `labels`, `assignees`, `state`.
  - If you’re closing an issue, include `state_reason` (e.g., `completed`, `not_planned`, `duplicate`).
6. (Optional) Maintain epic relationships.
  - If reorganizing sub-issues, use `#tool:github/sub_issue_write` (`add`, `remove`, `reprioritize`).

## Editing Guidelines

- Do not invent facts. If key details are missing, ask for them.
- Keep it safe: redact tokens, cookies, API keys, private URLs, wallet addresses, and any identifying data unless explicitly required.
- Prefer verifiable acceptance criteria.
  - If the issue lacks acceptance criteria, add a short bullet list.
- Prefer conservative labeling.
  - Only apply labels you can confirm exist (e.g., with `#tool:github/get_label`). If unsure, omit labels rather than guessing.
