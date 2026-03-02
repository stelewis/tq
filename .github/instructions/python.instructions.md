---
applyTo: "**/*.py"
---

# Code Instructions for Python

This document describes the project's preferred Python style, typing, and tooling conventions.

- Use Google docstring format
    - 72 characters per line
    - Summary line one line long, imperative mood
    - Add module-level docstrings to all Python files
    - Only specify type in docstrings if additional explanation is needed beyond what type hints provide (e.g., for complex types, units, or special argument behavior)
- Use Python's typing module for type annotations
    - Use PEP 585-style type annotations, leveraging built-in generic types
    - Use PEP 695-style type parameter syntax
- Leverage functional programming paradigms where appropriate
- Include `from __future__ import annotations` at the top files, including test files, if type hints are used
- Utilize function signatures that enforce keyword-only arguments where it improves clarity
    - Use `*` or `*,` in function definitions to require keyword arguments
- Prioritize root-cause refactors and best-practice structural changes
    - Breaking changes are encouraged if it moves us towards best practice
