# Governance

Governance policy for `tq` contract documentation.

## Reference ownership and review

Reference pages are contract pages and require stricter ownership and review than narrative docs.

| Reference section | Canonical source(s) | Required reviewer focus |
| --- | --- | --- |
| `docs/reference/cli.md` | `crates/tq-cli/src/cli.rs` + `docs/reference/cli/options-manifest.yaml` + `cargo run -p tq-docsgen --locked -- generate cli` | CLI contract shape, flag semantics, and generated table sync |
| `docs/reference/configuration.md` | Runtime config model + `docs/reference/config/examples-manifest.yaml` + `cargo run -p tq-docsgen --locked -- generate config` | Key/schema correctness, precedence behavior, and strict validation semantics |
| `docs/reference/exit-codes.md` | CLI/runtime exit behavior | Exit contract stability and compatibility impact |
| `docs/reference/rules/index.md` and `docs/reference/rules/*.md` | `docs/reference/rules/manifest.yaml` + `cargo run -p tq-docsgen --locked -- generate rules` | Rule ID stability, severity defaults, and per-rule contract clarity |
| `docs/developer/versioning.md` | Project versioning policy for user-visible contracts | Correct classification of breaking vs non-breaking changes |

Use this review baseline for any PR that touches `docs/reference/**`:

- Include at least one reviewer for the touched contract surface.
- Keep one canonical source of truth; link instead of duplicating contract text.
- For generated references, update the source manifest and commit regenerated outputs in the same PR.

## Rule and severity change policy

Rule additions and default severity changes are contract changes.

- MUST update `docs/reference/rules/manifest.yaml` in the same PR as runtime changes.
- MUST include `added_in` and update `behavior_changes` for affected rules.
- MUST regenerate rules docs via `cargo run -p tq-docsgen --locked -- generate rules`.
- MUST classify release impact using [versioning policy](./versioning.md).
- MUST summarize contract impact in `CHANGELOG.md`.
