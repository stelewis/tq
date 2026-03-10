# Copilot Instructions

Description: tq inspects a codebase's tests and enforces quality rules so tests remain discoverable, focused, actionable, and maintainable.

Repository: stelewis/tq

## Development Commands

Project uses Rust workspace tooling plus `uv` for packaging and repository automation.

### Code Quality

- Format: `cargo fmt --all --check`
- Lint: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- Tests: `cargo test --workspace --locked`
- Docs sync: `cargo run -p tq-docsgen --locked -- generate all`
- Release policy: `cargo run -p tq-release --locked -- verify-dependabot --repo-root .`
- Packaging check: `cargo package --workspace --locked && uv build`

### Common Tool Calls

- Rust CLI: `cargo run -p tq-cli --locked -- <args>`
- Docs generator: `cargo run -p tq-docsgen --locked -- <args>`
- Release tooling: `cargo run -p tq-release --locked -- <args>`
- Python: `uv run python <args>`
- Packaged project command: `uv run tq <args>`
- File system operations: `git mv`, `git rm`, `mv`, `rm`

### Multiline Command Line Issues

- Terminal wrapper may not manage multi-line strings correctly.
- You can create a temporary script file in `tmp/` to run complex commands.
- You do not need to quality gate these scripts.

### Full Validation

Before committing, run relevant checks. Full validation suite if required:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked && cargo run -p tq-docsgen --locked -- generate all && cargo run -p tq-release --locked -- verify-dependabot --repo-root . && cargo package --workspace --locked && uv build
```

## Guidelines

- MUST NOT assume existing design, architecture or code is correct.
- MUST NOT implement legacy or backward compatibility code:
  - MUST remove outdated modules, APIs and functions.
  - MUST refactor old code to align with current architecture and design.
  - MUST NOT preserve database or API schemas or implement migrations for legacy support.
- MUST NOT constrain new designs by trying to maintain compatibility or avoid breaking changes.
  - MUST strive for architectural excellence even if it requires significant changes.
  - MUST NOT take a convenience driven approach that compromises design quality.
- MUST ensure that test modules are properly refactored when source code changes (split, merge, replace, delete).
- MUST develop clean, maintainable, well factored, and elegant code.
- MUST NOT blindly comply with lint rules or contort otherwise clear code to satisfy linting heuristics.

**Correctness first, design forward.**
