# Copilot Instructions

Description: A test quality toolkit for Python codebases.

Repository: stelewis/tq

## Development Commands

Use the Rust workspace for product code and `uv` for packaging and repository automation.

### Core checks

- Format: `cargo fmt --all --check`
- Lint: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- Tests: `cargo test --workspace --locked`
- Docs sync: `cargo run -p tq-docsgen --locked -- generate all`
- Release policy: `cargo run -p tq-release --locked -- verify-release-policy --repo-root .`
- Packaging check: `cargo package --workspace --locked && uv build`

### Common commands

- Rust CLI: `cargo run -p tq-cli --locked -- <args>`
- Docs generator: `cargo run -p tq-docsgen --locked -- <args>`
- Release tooling: `cargo run -p tq-release --locked -- <args>`
- Python: `uv run python <args>`
- File system operations: `git mv`, `git rm`, `mv`, `rm`

### Terminal note

- Terminal wrapper may not manage multi-line strings correctly.
- You can create a temporary script file in `tmp/` to run complex commands.
- You do not need to quality gate these scripts.

### Full validation

Run the relevant subset for the task. When full validation is required:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked && cargo run -p tq-docsgen --locked -- generate all && cargo run -p tq-release --locked -- verify-release-policy --repo-root . && cargo package --workspace --locked && uv build
```

## Guidelines

- MUST NOT assume existing design, architecture or code is correct.
- MUST NOT implement legacy or backward compatibility code:
  - MUST remove outdated modules, APIs and functions.
  - MUST refactor old code to align with current architecture and design.
  - MUST NOT preserve database or API schemas or implement migrations for legacy support.
- MUST NOT constrain new designs by trying to maintain compatibility or avoid breaking changes.
  - MUST strive for architectural excellence even if it requires significant changes; prefer architectural clarity over convenience patches.
  - MUST NOT take a convenience driven approach that compromises design quality.
- MUST treat internal crate APIs as current-only interfaces, not compatibility surfaces.
  - When an internal crate API changes, MUST update all workspace callers in the same change.
  - MUST remove the old API immediately instead of adding shims, aliases, adapter helpers, or dual-path call sites.
  - MUST bump the shared workspace/internal crate minor version before packaging or release validation when an internal public API changes.
  - If `cargo package --workspace --locked` fails because a published crate version no longer matches the current internal API, MUST fix that by bumping the workspace/internal crate version, not by restoring compatibility code.
- MUST ensure that test modules are properly refactored when source code changes (split, merge, replace, delete).
- MUST develop clean, maintainable, well factored, and elegant code.
- MUST NOT blindly comply with lint rules or contort otherwise clear code to satisfy linting heuristics.

## Security

This project takes a strong stance on supply-chain and codebase security.

- MUST treat dependency additions and upgrades as supply-chain security decisions, not convenience choices.
- MUST prefer mainstream, widely adopted, well maintained ecosystem staples with clear ownership and strong engineering discipline.
- MUST avoid introducing newly created, obscure, weakly maintained, or low-trust packages by default.
- MUST avoid packages that appear speculative, hastily assembled, generated without strong review, or otherwise low-rigor.
- MUST justify any non-obvious dependency choice against established alternatives and explain why owning the code locally is worse.
- MUST inspect transitive dependency impact before adding a package.
- MUST use the repository's dependency and security tooling when dependency changes are involved, including `cargo audit`, `cargo deny check`, and relevant lockfile review.
- MUST treat passing scanners as necessary but not sufficient; reputation, maintenance history, adoption, and release hygiene are equally important.

**Correctness first, design forward.**
