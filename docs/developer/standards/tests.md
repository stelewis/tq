# Testing Standards

This project treats the test suite as a first-class part of the codebase.

The goal is to keep tests:

- **Discoverable**: easy to find the test for a module.
- **Focused**: small surface area, minimal cross-module coupling.
- **Actionable**: failures point to one contract, not "the system".
- **Maintainable**: tests refactor with the code (SOLID applies here too).

## Structure Rules

### One test module per source module (minimum)

For each source module there must be *at least one* corresponding test module.

Required mapping:

- Source: `src/<project>/<path>/<module>.py`
- Test: `tests/<project>/<path>/test_<module>.py`

This is intentionally strict. It prevents "mystery tests" and keeps the suite aligned as the architecture evolves.

It is totally acceptable for a module’s first test to be a placeholder smoke test (e.g., exercising an import or a minimal happy-path). The point is to make expanding coverage easy and keep the suite navigable.

### Splitting tests (qualifiers)

If a test suite is too large for one file, split into multiple files by concern. Use the naming convention:

- `test_<module>_<qualifier>.py`

Only add a qualifier when it improves clarity and separation of concerns. Qualifiers should be stable and describe a single responsibility.

Avoid qualifiers that encode implementation details or ephemeral refactors.

Note: pytest runs with `--import-mode=importlib` (see `pyproject.toml`) to avoid import name clashes across similarly named tests.

### Single-target rule (unit tests)

As a default, a unit test module targets exactly one source module: the one implied by its path/name.

This means:

- Do not keep a monolithic test file after splitting a large source module into smaller modules.
- Do not "reach across" and assert behavior that belongs to other modules.

If you need behavior across modules, that is usually a sign the contract belongs in a higher-level integration test.

### Integration tests

Integration tests are allowed when they validate real workflows. These may span multiple modules by design.

Place integration tests in `tests/integration/` to separate them from unit tests. This keeps the unit test suite focused and discoverable.

Rules:

- Mark integration tests with `pytest.mark.integration`.
- Keep integration test names workflow-oriented (e.g. `test_ingest_cli.py`).

### End-to-end tests

If needed, place in `tests/e2e/` and mark with `pytest.mark.e2e`.

### Golden / snapshot fixture tests

Golden (aka “snapshot”) tests are allowed when they validate a correctness-critical contract that is hard to express as small unit assertions.

- A **golden fixture** should be derived from a real incident.
- A **synthetic fixture** (small, hand-constructed) can be useful for targeted contracts but it should be treated as a unit/invariant test input — not evidence we match reality.

Rules:

- Must be fully offline and deterministic (no network, no wall-clock time).
- Keep fixtures small and reviewable (prefer JSONL/JSON; stable ordering; version fields).
- Avoid “assert the entire snapshot equals a blob” unless you have a strong reason.

Placement:

- Keep golden tests in `tests/<project>/...` near the subsystem they validate.
- Store fixture files adjacent to the test module (e.g. `fixtures/golden` or `fixtures/synthetic`).

### Excluded Tests

Some marked tests may be excluded from test runs to keep `pytest` fast and deterministic. Run excluded tests explicitly when needed:

```bash
uv run pytest -m e2e
```

## Test Quality Standards

### Avoid these anti-patterns

- **Monolithic unit tests**: one test module covering many unrelated modules.
- **Cross-module unit tests**: tests with no clear single target.
- **Duplicated coverage**: multiple suites asserting the same contract.
- **Redundant tests**: re-testing behavior already validated elsewhere.
- **Structure mismatches**: test location doesn’t mirror the source module.
- **Misnamed tests**: name implies a different target than what it covers.
- **Orphaned tests**: tests primarily covering code that no longer exists.
- **Vacuous tests**: tests that pass without meaningfully exercising behavior.
- **Very large test modules**: tests that try to cover too much in one suite.

Practical refactor rule:

- If you split `foo.py` into `foo/alpha.py` and `foo/beta.py`, the tests should split too. Keeping `test_foo.py` as a grab-bag is the failure mode to avoid.

### Use pytest markers

Markers should communicate intent and allow selective runs.

Use the project markers registered in `pyproject.toml` if the test is one of:

- `pytest.mark.e2e`: end-to-end tests (full user workflows).
- `pytest.mark.golden`: deterministic golden/snapshot fixture tests (offline).
- `pytest.mark.integration`: multi-module workflow tests.
- `pytest.mark.regression`: tests added for a fixed bug/regression.
- `pytest.mark.slow`: tests that may be skipped in fast CI jobs.
- `pytest.mark.smoke`: quick, minimal tests (including placeholders).

### Fixtures

Place reusable fixtures in `conftest.py` files at appropriate levels to share them across test modules to avoid duplication and reduce the future refactor surface.

## Workflow

This project includes an automated test quality checker which can be run via:

```bash
uv run check_test_quality
```
