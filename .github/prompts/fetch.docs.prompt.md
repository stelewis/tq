---
name: fetch-docs
description: Fetch up-to-date documentation using the Context7 MCP tool
agent: agent
---

Use Context7 whenever the user asks about an external library, framework, or API.

1. Identify the library and version, if one was specified.
2. Resolve the Context7 library ID. Use the `#tool:context7/resolve-library-id` tool.
3. Query the docs for the specific topic the user asked about. Use the `#tool:context7/query-docs` tool.
4. Answer from the fetched material rather than memory.

If the user names a version or narrow topic, include it in the lookup.

## References

- <https://github.com/upstash/context7>
- <https://context7.com/>
