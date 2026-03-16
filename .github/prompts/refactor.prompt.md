---
agent: agent
---

Support a design-forward refactor of the codebase according to the user’s goals.

Think through design intent and make the changes needed to reach the target architecture. Remove or replace modules and tests freely when the existing structure is the problem.

## Refactor Implementation

- MUST NOT assume existing architecture or code is correct
- MUST NOT constrain new designs by trying to maintain compatibility or avoid breaking changes
- MUST NOT implement backward compatibility or legacy code
- MUST NOT write code or tests in a way that re-introduces the old architecture
- MUST strive for architectural excellence even if it requires significant changes
- MUST adhere to the [Testing Standards](../../docs/developer/standards/tests.md)
- MUST ensure test modules are also properly refactored

## Outdated Tests

Do not leave monolithic test modules that test multiple source modules.

For each test module implicated in a refactor:

- **Rewrite** when a suite still validates a real contract but is tied to old names or signatures.
- **Split** when one file tests multiple source modules or responsibilities.
- **Merge** when multiple suites duplicate the same contract.
- **Fix** when a suite is logically right and only needs mechanical updates.
- **Delete** when it encodes old layering or stale semantics.
- **Replace** when it is large, fragile, or asserts legacy constructs.

## Quality Gates

Ensure all checks pass:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `cargo run -p tq-docsgen --locked -- generate all`
- `cargo run -p tq-release --locked -- verify-dependabot --repo-root .`
- `cargo package --workspace --locked`
- `uv build`
