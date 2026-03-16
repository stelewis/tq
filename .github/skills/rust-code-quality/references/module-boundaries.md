# Module Boundaries

Keep crate and module ownership explicit.

Prefer:

- narrow `lib.rs` exports,
- direct imports from owning modules,
- composition roots that construct dependencies explicitly,
- modules that each have one clear reason to change.

Refactor when you see:

- `lib.rs` turning into a broad re-export hub,
- convenience layers that blur crate ownership,
- domain crates constructing adapters or reading the filesystem directly,
- tests that span multiple source responsibilities in one file.

The goal is local reasoning: one owner per concept and one contract per test suite.
