# Strong Internal Types

Use dedicated Rust types for validated concepts that should not drift as raw strings, paths, booleans, or tuples.

Prefer:

- enums for closed vocabularies,
- newtypes for stable identifiers,
- validated structs for boundary-parsed state.

Refactor pattern:

1. Parse external strings or loose shapes at the boundary.
2. Convert once into enums, newtypes, or validated structs.
3. Keep core logic on typed values only.
4. Update tests to assert boundary parsing and typed behavior separately.

Avoid carrying raw `String` or `&str` values through the engine just because they arrived that way.
