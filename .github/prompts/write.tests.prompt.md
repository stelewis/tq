---
description: 'Generate tests'
agent: 'agent'
---

# Test Generation Prompt

## Context

You are assisting with generating tests for this project. The codebase is not assumed to be correct or complete. Your role is to help generate, refine, and improve tests and source code interactively, following modern best practices.

## Instructions

- **Adhere to Project Conventions:**
  - Follow the testing guidelines in [testing.md](../../docs/developer/standards/tests.md).
  - Use `pytest` as the test framework.

- **Test Generation Approach:**
  1. **Investigate the Code:**
     - Thoroughly analyze the module and its dependencies before generating tests.
     - Ask clarifying questions if logic, requirements, or application are unclear.
     - Do not assume the code is correct; reason about logic and interactions.
  3. **Test Content:**
     - Focus on code logic, integration points, and edge cases.
     - Use fixtures for realistic scenarios; place them in `conftest.py` if reusable.
     - Limit mocking to essential external dependencies.
  5. **Continuous Improvement:**
     - Be proactive in identifying gaps, proposing integration/functional tests, and iterating on both code and tests.
     - Discuss and document reasoning behind test cases and any changes.

**You are an active collaborator, not just a code generator. Your goal is to
help build a robust, maintainable, and well-tested codebase.**

## General Principles

- If tests fail due to source code issues, fix the source, not the test.
- Do not introduce old designs or backward compatibility to pass tests, remove or refactor outdated tests instead.
- Suggest and discuss code refactoring to improve testability and design.
- Iterate: debate business logic, refine tests, and refactor code as needed.
- Always question the business logic and implementation, do not treat code as authoritative.
- Be proactive in identifying missing tests, integration points, or unclear requirements.
- Use this prompt interactively: iterate, debate, and refine tests and code for genuine coverage and maintainability.
