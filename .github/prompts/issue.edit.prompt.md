---
agent: agent
---

# Issue Edit Prompt (Agent)

Edit GitHub issues for this repository using the GitHub MCP server tools.

## Workflow (MCP)

1. Determine repo coordinates (`owner`, `repo`).
2. Fetch the current issue state.
  - Use `mcp_github_issue_read` with `method: get` (and optionally `get_comments` if you need more context).
3. Decide the minimum necessary change.
  - Preserve any correct context; do not rewrite history or remove important details.
  - If the ask is ambiguous, ask focused questions rather than making assumptions.
4. Draft the updated body in a file.
  - Create `tmp/issues/issue-<timestamp>.md` first.
  - Put `<!-- markdownlint-disable MD013 -->` at the top.
  - Do not hard wrap lines in the issue body.
5. Apply the edit.
  - Use `mcp_github_issue_write` with `method: update` and `issue_number`.
  - Update only what’s needed: `title`, `body`, `labels`, `assignees`, `state`.
  - If you’re closing an issue, include `state_reason` (e.g., `completed`, `not_planned`, `duplicate`).
6. (Optional) Maintain epic relationships.
  - If reorganizing sub-issues, use `mcp_github_sub_issue_write` (`add`, `remove`, `reprioritize`).

## Editing Guidelines

- Do not invent facts. If key details are missing, ask for them.
- Keep it safe: redact tokens, cookies, API keys, private URLs, wallet addresses, and any identifying data unless explicitly required.
- Prefer verifiable acceptance criteria.
  - If the issue lacks acceptance criteria, add a short bullet list.
- Prefer conservative labeling.
  - Only apply labels you can confirm exist (e.g., with `mcp_github_get_label`). If unsure, omit labels rather than guessing.

- Create issue files at `tmp/issues/issue-<timestamp>.md` first.
  - Do not hard wrap lines in the issue body or text file. This avoids broken rendering on GitHub.
  - Create the issue draft file using the file tool.
  - Use the file content as the `body` field for the tool call.

## References

- GitHub MCP tools: `mcp_github_issue_read`, `mcp_github_issue_write`, `mcp_github_sub_issue_write`
- See also: `./issue.create.prompt.md`
