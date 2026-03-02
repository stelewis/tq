---
agent: agent
---

Fetch the websites, APIs, or other online resources specified by the user. Gather the necessary data and context to assist with their request. Use the Jina AI Reader tool to fetch its content in an LLM-friendly format.

## Steps

1. Prepend `https://r.jina.ai/` to the target URL.
2. Use the available HTTP tool to perform a GET request to this constructed URL.
3. If provided with a search query instead of a URL, use `https://s.jina.ai/?q=` followed by the query to get search results.
