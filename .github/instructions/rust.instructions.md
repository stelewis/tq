---
applyTo: "**/*.rs"
---

# Code Instructions for Rust

- Prefer small modules, explicit types, and narrow `pub` surfaces.
- Keep construction in binaries and composition roots; do not read files, env vars, or process state from domain logic.
- Use enums, newtypes, and dedicated structs for IDs, closed vocabularies, and validated state.
- Return typed errors with actionable context and preserved causes.
- Prefer borrowing or moving to reflect ownership clearly; clone intentionally and sparingly.
- Use `BTreeMap` and `BTreeSet` when ordering is user-visible, serialized, or asserted in tests.
- Avoid `unwrap` and `expect` in non-test code; in tests, prefer `expect` with a message that explains the setup assumption.
- Derive only the traits a type actually needs and keep invariants explicit.
- Prioritize root-cause refactors and structural improvements over convenience patches.
