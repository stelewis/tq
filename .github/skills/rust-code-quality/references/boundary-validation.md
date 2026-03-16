# Boundary Validation

Validation belongs at boundaries, not deep inside domain code.

Prefer this split:

- adapters read files, CLI args, env vars, or manifests,
- boundary code validates and materializes typed state,
- domain code assumes validated inputs and stays simple.

Refactor when you see:

- repeated `Option` or string checks across domain functions,
- path normalization spread through multiple layers,
- late validation that turns input errors into opaque runtime failures.

A good result is one strict materialization path and a smaller pure core.
