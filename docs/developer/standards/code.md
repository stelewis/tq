# Code Standards

Use these standards to keep code correct, maintainable, and easy to evolve.

Goals:

- **Correctness first**: prevent silent failures and ambiguous behavior.
- **Design excellence**: optimize for long-term maintainability, extensibility, and clarity.
- **Small surface area**: minimize refactor blast-radius; avoid speculative abstraction.

## Principles

- **SOLID / separation of concerns**: each unit has one reason to change; policies don’t depend on details.
- **Explicitness / strictness**: make dependencies and contracts visible; fail fast at boundaries.
- **Cleanliness**: prefer simple shapes, clear naming, and small functions.
- **Minimalism (YAGNI)**: implement today’s requirement cleanly; do not pre-build optional futures.
- **Extensibility without fragility**: enable new features by adding new code paths, not by editing many unrelated ones.

## Architecture Rules

- **Composition root**: construct the object graph in one place (CLI entrypoint / app bootstrap). No “hidden construction” inside domain logic.
- **Dependency injection**: pass dependencies explicitly (constructors / factory functions), not via global state or implicit defaults.
- **Boundaries are strict**: adapters convert formats; they do not guess intent or silently coerce.
- **Domain stays pure**: core logic should not know about IO, filesystem, environment variables, or external SDKs.

## Antipatterns to Avoid

- **Re-export hubs in package `__init__.py` files**: creates unstable import graphs and hides ownership.
- **Catch-all error handling that loses signal**: swallowing exceptions or returning sentinel values in critical loops.
- **Stringly-typed identifiers / closed vocabularies**: raw strings drifting through core logic for IDs, enums, and state.
- **Defaults in function signatures that hide behavior**: implicit deps or “magic” config weaken contracts and tests.
- **Silent defaults in runtime models/config**: defaulting missing/invalid fields instead of failing fast (defaults belong in the composition root or boundary config).
- **Compatibility coercion**: do not auto-upgrade legacy shapes at runtime; fix the boundary inputs.
- **“Forever fixtures” mindset**: fixtures are not a compatibility promise; when schemas change, regenerate/update fixtures.
- **Blind lint-rule compliance**: do not contort otherwise clear code to satisfy linting heuristics; align with the rule intent and use scoped exceptions when needed.

## Preferred Patterns

- **Fail-fast contracts**: validate inputs at boundaries; raise actionable errors with file/line context.
- **Strong internal types**: use dedicated types for identifiers and vocabularies; keep conversion at edges.
- **Explicit imports**: prefer importing exact module paths; keep dependency graphs readable and cycle-resistant.
- **Schema evolution by version bump**: change runtime schemas intentionally and update fixtures/tests accordingly.
- **No runtime migrations / backward compatibility**: if a schema or contract changes, break intentionally and update the callers/fixtures rather than carrying adapters in core code.
- **Narrow interfaces**: depend on small protocols/ABCs that model *what you need*, not the full dependency.
- **Local reasoning**: keep functions small and side-effect-free where possible; push side effects to the edges.
- **Clear naming**: choose names that express intent and domain meaning.
  - Nouns for types, verbs for actions: classes/types are nouns; functions/methods are verbs.
  - Booleans as predicates: use `is_*`, `has_*`, `can_*`, `should_*`.
  - Collections plural: name collection variables in plural (e.g., `orders`).
  - Prefer specific names over generic ones (`order_id` > `id`; `runner_config` > `config`).
  - Use one canonical name per concept; avoid synonym drift (e.g., `slug` vs `id`).

## Review Checklist

- Are dependencies constructed in a composition root, not inside core logic?
- Are boundary adapters strict (no silent coercion), with good error messages?
- Are defaults explicit and located at the edges (not silently applied inside models/core logic)?
- If schemas/contracts changed, did we bump/update callers and fixtures instead of adding runtime compatibility?
- Is the code minimal (no unused abstractions), and the refactor surface area contained?
