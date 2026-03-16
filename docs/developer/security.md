# Security

Security in `tq` is not limited to dependency scanning. It covers source code, dependency admission, CI and release automation, artifact contents, and disclosure handling.

## Security model

- Treat security as a design property, not a cleanup task.
- Keep trust boundaries narrow and explicit.
- Validate untrusted inputs at the boundary and fail closed on ambiguity.
- Keep automation deterministic, reviewable, and pinned.
- Keep the dependency surface small and high-trust.
- Keep shipped artifacts minimal, attestable, and predictable.

## Source code expectations

Code must:

- validate untrusted CLI, config, filesystem, archive, environment, and process inputs before they reach core logic,
- reject ambiguous or invalid security-relevant state instead of guessing,
- defend against path traversal, symlink escape, absolute-path escape, and unsafe extraction flows,
- avoid secret exposure in code, logs, errors, fixtures, snapshots, and docs,
- redact sensitive diagnostics by default,
- prefer structured subprocess execution with validated arguments over shell construction.

Source-level rules live in [Code standards](./standards/code.md).

## Supply chain and automation

Dependency changes are security decisions. New crates, Python packages, Node packages, GitHub Actions, pre-commit hooks, and build tools must clear a trust and maintenance bar, not just a feature bar.

Repository automation must also preserve the security posture:

- external GitHub Actions are SHA pinned,
- pre-commit hooks are high-quality and SHA pinned,
- Dependabot must cover the full GitHub Actions surface,
- Rust dependency changes must remain compatible with `cargo audit` and `cargo deny`,
- lockfiles are part of code review,
- release artifacts must pass provenance and content policy checks.

Dependency-admission rules live in [Supply-chain security standards](./standards/supply-chain-security.md).

Repository-enforced automation policies live in [Project policies](./standards/policies.md).

## Disclosure

Suspected vulnerabilities must be reported privately through [SECURITY.md](../../SECURITY.md).

## Related docs

- [Code standards](./standards/code.md)
- [Supply-chain security standards](./standards/supply-chain-security.md)
- [Project policies](./standards/policies.md)
- [Developer tools](./tools/index.md)
- [Attestation verification](./attestation.md)
