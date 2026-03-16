# Catch-All Error Handling

Eliminate error-handling patterns that hide failures by silently dropping or best-effort accepting unexpected inputs.

## Objective

- Prevent silent loss of unexpected inputs in critical loops.
- Surface structured, typed errors with bounded context.
- Update tests so failure paths are observable.

## Non-Negotiables

- Do not use bare except: or except BaseException:.
- In async code, always re-raise asyncio.CancelledError.
- Unknown payload shapes are errors and must be validated at boundaries.
- Do not keep compatibility scaffolding; update all call sites and tests.

## What To Search For

- Broad handlers such as except Exception, bare except, or return_exceptions=True without inspection.
- Silent drops such as else: continue, pass on unknown types, or ignore unknown message branches.
- Masking defaults such as payload.get("k", "") or get(..., 0) where the default is treated as valid data.
- while True or async for loops that catch and continue without surfacing the error.

## Refactor Pattern

1. Classify each location as a critical loop that must surface failures or as lower-stakes tooling code.
2. Replace catch-all logic with explicit handling:
   - Handle known cases with match or explicit if branches.
   - Raise a dedicated error, or emit a typed error object, for unknown cases.
3. If the loop must keep running, emit the structured error to the existing error surface such as metrics, callbacks, health state, or an event bus, then continue.
4. Bound the context on errors: include a subsystem identifier, optional message kind, a short bounded identifier list, and a raw sample capped at roughly 2 KB.
