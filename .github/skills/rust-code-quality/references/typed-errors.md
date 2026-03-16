# Typed Errors

Expose errors that preserve the failure kind, source, and boundary context.

Prefer:

- one error enum per crate or boundary,
- explicit variants for distinct failure modes,
- `#[source]` for preserved causes,
- actionable messages that name the path, rule, or contract that failed.

Avoid:

- catch-all `String` errors,
- collapsing unrelated failures into one vague variant,
- losing the source error when crossing a boundary.

If two failure cases need different remediation, they should usually be different variants.
