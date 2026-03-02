---
agent: agent
---

# Issue Creation Prompt

Create high-quality GitHub issues for this repository using the GitHub MCP server tools.

## Primary Goal

Produce a single issue (or multiple if requested) that is correct, specific,
actionable, easy to reproduce/verify, appropriately labeled and assigned, and
safe (no secrets, credentials, or PII).

## Guidelines

- Do not invent facts. If information is missing, ask for it.
- Titles should describe the work/problem, not categorize it.
  - Prefer imperative, verb-first phrasing ("Add …", "Fix …", "Implement …", etc.).
  - Avoid category prefixes in titles (e.g., "Epic:", "Feature:", "Bug:", etc.).
- Prefer existing labels.
  - Only apply labels you can confirm exist (e.g., with `mcp_github_get_label`). If unsure, omit labels rather than guessing.
- Prefer crisp verification criteria.
  - Every issue should have acceptance criteria that can be checked as done.
  - Bugs must include clear reproduction steps and expected vs actual behavior.
- Keep it safe.
  - Redact tokens, cookies, API keys, private URLs, wallet addresses, and any identifying data unless explicitly required.

## Workflow (MCP)

1. Determine repo coordinates (`owner`, `repo`).
2. De-duplicate.
   - Use `mcp_github_search_issues` to find likely duplicates before creating a new issue.
3. Choose labels and type conservatively.
   - Confirm labels with `mcp_github_get_label` when applying them.
   - If issue types are enabled, discover valid types with `mcp_github_list_issue_types` and set `type` on create.
4. Draft the issue body in a file.
   - Create `tmp/issues/issue-<timestamp>.md` first.
   - Put `<!-- markdownlint-disable MD013 -->` at the top.
   - Do not hard wrap lines in the issue body.
5. Create the issue.
   - Call `mcp_github_issue_write` with `method: create`.
   - Provide `title`, `body`, optional `labels`, optional `assignees`, optional `type`.
6. If this is an epic, attach sub-issues.
   - Create sub-issues first, then connect them with `mcp_github_sub_issue_write`.

## Tool Call

Use `mcp_github_issue_write` with `method: create`.

- Create issue files at `tmp/issues/issue-<timestamp>.md` first.
  - Do not hard wrap lines in the issue body or text file. This avoids broken rendering on GitHub.
  - Create the issue draft file using the file tool.
  - Use the file content as the `body` field for the tool call.

## References

- GitHub MCP tools: `mcp_github_issue_write`, `mcp_github_issue_read`, `mcp_github_sub_issue_write`
