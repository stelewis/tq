---
applyTo: "**/*.py"
---

# Code Instructions for Python

Preferred Python style, typing, and tooling conventions.

- Prefer clear names and structure over explanatory docstrings
- Treat docstrings as optional; add them only when they capture context the code and type hints do not
- Keep docstrings concise, Google style, wrapped to 72 characters, with a one-line imperative summary when possible
- Use docstrings for contracts, side effects, units, invariants, error cases, or other non-obvious behavior
- Do not restate types, parameter lists, or control flow that are already clear from the code
- Include `from __future__ import annotations` at the top of files
- Use keyword-only arguments where they improve call-site clarity
- Leverage functional programming paradigms where appropriate
- Prioritize root-cause refactors and best-practice structural changes
