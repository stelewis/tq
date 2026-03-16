# Code Standards

Use these standards to keep the Rust workspace correct, deterministic, and easy to evolve.

Goals:

- **Correctness first**: prevent silent failures and ambiguous behavior.
- **Design excellence**: optimize for long-term maintainability, extensibility, and clarity.
- **Determinism by default**: keep diagnostics, docs, and release outputs stable across runs.
- **Small surface area**: minimize refactor blast-radius; avoid speculative abstraction.

## Principles

- **Separation of concerns**: each crate and module should have one clear reason to change.
- **Explicitness / strictness**: make dependencies and contracts visible; fail fast at boundaries.
- **Strong internal types**: represent IDs, vocabularies, and validated state with dedicated Rust types.
- **Deterministic behavior**: never let filesystem order, hash iteration, or incidental formatting leak into contracts.
- **Minimalism (YAGNI)**: implement today’s requirement cleanly; do not pre-build optional futures.

## Architecture Rules

- **Composition root**: construct runtime graphs in binaries such as `tq-cli`, `tq-docsgen`, and `tq-release`. No hidden construction inside domain crates.
- **Crate ownership is explicit**: each crate owns one boundary. Do not create convenience layers that blur config, discovery, engine, rules, reporting, and tooling responsibilities.
- **Boundaries are strict**: adapters convert formats; they do not guess intent or silently coerce.
- **Domain stays pure**: core logic should not know about CLI parsing, filesystem walking, environment variables, or release automation details.
- **Workspace consistency**: internal crate dependencies belong in the root workspace dependency table and should be consumed with `.workspace = true`.
- **Public API is deliberate**: keep `pub` surfaces narrow and avoid re-export hubs that hide ownership.

## Rust Practices

- **Closed vocabularies use enums or newtypes**: avoid raw strings and boolean parameter pairs in core logic.
- **Errors stay typed**: library crates should expose precise error types and preserve source causes with actionable context.
- **Ownership is intentional**: borrow or move to match the real data flow; clone only when it simplifies a boundary and the cost is understood.
- **Ordering is explicit**: use `BTreeMap`/`BTreeSet` or explicit sorting when order is user-visible, serialized, or asserted in tests.
- **Immutability after validation**: prefer validated structs and pure transformations over mutation-heavy state machines.
- **Unsafe is not a convenience tool**: the workspace forbids `unsafe`; do not introduce it without an explicit architectural reason and review.

## Security By Construction

- **Validate untrusted input at boundaries**: parse, normalize, and reject invalid CLI, config, filesystem, archive, and environment inputs before they reach core logic.
- **Fail closed**: on ambiguous, missing, or invalid security-relevant state, return an actionable error instead of guessing or silently defaulting.
- **Constrain filesystem effects**: canonicalize and validate paths when crossing trust boundaries; do not allow archive extraction, temp handling, or path joins to escape intended roots.
- **Do not leak secrets**: never hardcode secrets, commit live credentials, or emit sensitive values in logs, errors, fixtures, docs, or test snapshots.
- **Prefer structured process execution**: pass explicit argument arrays and validated inputs to subprocesses; do not build shell commands from untrusted strings.
- **Keep diagnostics safe**: preserve enough context to debug failures without exposing tokens, secrets, or other sensitive material.

## Antipatterns to Avoid

- **Catch-all error handling that loses signal**: collapsing distinct failures into opaque messages or sentinel values.
- **Stringly-typed identifiers / closed vocabularies**: raw strings drifting through core logic for IDs, rule names, and states.
- **Silent defaults in runtime models/config**: defaulting missing or invalid fields instead of failing fast.
- **Compatibility coercion**: do not carry legacy adapters, schema upgrades, or dual-path behavior in runtime crates.
- **Hidden IO in domain code**: reading files, env vars, or process state from core planning and evaluation logic.
- **Global mutable state or convenience interior mutability**: avoid shared hidden state when explicit ownership would be clearer.
- **Broad `lib.rs` barrels**: do not flatten module ownership behind large re-export surfaces.
- **Blind lint-rule compliance**: do not contort otherwise clear code to satisfy heuristics; align with the rule intent and use focused exceptions when needed.

## Preferred Patterns

- **Fail-fast contracts**: validate inputs at boundaries and return actionable errors.
- **Explicit imports**: import exact module paths so dependency graphs stay readable and cycle-resistant.
- **Narrow interfaces**: depend on the smallest trait or type surface that models the need.
- **Local reasoning**: keep functions small and side-effect-light; push side effects to the edges.
- **Clear naming**: choose names that express intent and domain meaning.
  - Nouns for types, verbs for actions: classes/types are nouns; functions/methods are verbs.
  - Booleans as predicates: use `is_*`, `has_*`, `can_*`, `should_*`.
  - Collections plural: name collection variables in plural (e.g., `orders`).
  - Prefer specific names over generic ones (`order_id` > `id`; `runner_config` > `config`).
  - Use one canonical name per concept; avoid synonym drift (e.g., `slug` vs `id`).

## Review Checklist

- Are dependencies constructed in a composition root, not inside domain crates?
- Is crate ownership clear, with boundaries that match the architecture docs?
- Are boundary adapters strict, typed, and actionable on failure?
- Does the change validate untrusted inputs and fail closed on invalid or ambiguous state?
- Could any path, archive, subprocess, log, or error surface expose sensitive data or escape its intended boundary?
- Is ordering explicit anywhere output or tests depend on it?
- Is the public API smaller than the implementation, not the other way around?
- If contracts changed, did we update callers, fixtures, and docs instead of adding compatibility code?
