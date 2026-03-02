---
name: fetch-docs
description: Fetch up-to-date documentation using the Context7 MCP tool
agent: agent
---

# Instruction

When I ask about a specific library, framework, or API, use the **Context7 MCP** tool to retrieve the latest documentation before providing code or explanations.

## Steps

1. **Identify the Library**: Determine which library or framework the user is asking about.
2. **Resolve ID**: Use the `#tool:context7/resolve-library-id` tool to find the official Context7-compatible ID.
3. **Fetch Docs**: Use the `#tool:context7/query-docs` tool with the resolved ID to pull in relevant documentation snippets.
4. **Answer**: Provide the answer or code generation using these up-to-date snippets to avoid hallucinations of outdated APIs.

## Best Practices

- If I specify a version (e.g., "Next.js 13"), include the version in the resolution step.
- For specific topics, use the `topic` parameter in the fetch tool to narrow down results (e.g., "hooks", "routing").

## References

- <https://github.com/upstash/context7>
- <https://context7.com/>
