# Developer Tools

Developer tooling and commands used to work on `tq`.

## Status

The Rust rewrite plan introduces and standardizes this tooling surface in phases.

Until cutover is complete, this page documents the target Rust toolchain and is updated incrementally as each phase lands.

## Rust toolchain targets

Post-cutover, this page will document at minimum:

- workspace bootstrap and build commands,
- formatting and linting commands,
- test and conformance commands,
- security and dependency audit commands,
- release and docs-generation commands.

## Governance

- Keep this page aligned with CI workflows and pre-commit hooks.
- Remove legacy Python runtime commands once Rust cutover completes.
