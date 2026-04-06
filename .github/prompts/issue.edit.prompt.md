---
name: issue-edit
description: Edit an existing GitHub issue with the minimum necessary change
argument-hint: Issue number and the specific update to make
agent: agent
---

Edit GitHub issues for this repository using the GitHub MCP server tools.

## Workflow

1. Determine repo coordinates (`owner`, `repo`).
2. Fetch the current issue state.
3. Decide the minimum necessary change.
  - Preserve correct context and avoid unnecessary rewrites.
  - If the ask is ambiguous, ask focused questions.
4. Draft the updated body in a file.
  - Create `tmp/issues/issue-<timestamp>.md` first.
5. Apply the edit.
  - Update only what is needed.
  - If closing the issue, include `state_reason`.
6. (Optional) Maintain epic relationships.
  - If reorganizing sub-issues, use `#tool:github/sub_issue_write`.

## Editing Guidelines

- Do not invent facts. If key details are missing, ask for them.
- Keep it safe: redact tokens, cookies, API keys, private URLs, wallet addresses, and any identifying data unless explicitly required.
- Prefer verifiable acceptance criteria; add a short list if the issue lacks them.
- Prefer conservative labeling; only apply labels you can confirm exist.
