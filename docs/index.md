---
layout: home

hero:
  name: tq
  text: Test quality checks for Python codebases
  tagline: Keep tests discoverable, focused, actionable, and maintainable.
  image:
    src: /tq-logo-large.svg
    alt: tq
  actions:
    - theme: brand
      text: What is tq?
      link: /guide/what-is-tq
    - theme: alt
      text: Quickstart
      link: /guide/quickstart
    - theme: alt
      text: Rules Index
      link: /reference/rules/
    - theme: alt
      text: GitHub
      link: https://github.com/stelewis/tq

features:
  - icon: 📐
    title: Enforce test structure contracts
    details: Catch missing, orphaned, oversized, and structure-mismatched tests with stable rule IDs.
  - icon: 🧭
    title: Keep diagnostics deterministic
    details: Get reproducible findings and machine-readable JSON output that stays reliable in CI automation.
  - icon: ⚡️
    title: Run with ruff-style ergonomics
    details: Use a subcommand-first operator surface with explicit rule selection, suppression, and precedence.
  - icon: 🎯
    title: Scope checks by target
    details: Define multiple targets in pyproject and run focused checks with --target when repos have mixed layouts.
---
