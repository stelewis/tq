# Repository Security Standards

This repository already has explicit security standards. Use them as the source of truth for audits and recommendations.

## Primary Sources

- [docs/developer/standards/code.md](../../../../docs/developer/standards/code.md)
- [docs/developer/standards/supply-chain-security.md](../../../../docs/developer/standards/supply-chain-security.md)
- [docs/developer/standards/policies.md](../../../../docs/developer/standards/policies.md)
- [docs/developer/tools/ci.md](../../../../docs/developer/tools/ci.md)
- [deny.toml](../../../../deny.toml)
- [.github/actions/setup-rust-security-tools/action.yml](../../../../.github/actions/setup-rust-security-tools/action.yml)

## Runtime Code Review Invariants

- Validate untrusted input at boundaries and fail closed on invalid or ambiguous state.
- Review new or changed filesystem logic for `..`, symlink traversal, absolute paths, and root escape.
- Review archive handling, temp paths, and extraction flows for boundary escape and unintended writes.
- Prefer structured subprocess execution with validated argument arrays over shell command construction.
- Keep secrets and sensitive local data out of logs, errors, fixtures, snapshots, and generated docs.
- Keep domain crates pure. Hidden filesystem, environment, or process reads inside domain logic are review targets because they blur trust boundaries.

## Supply-Chain Review Invariants

- Dependency admission is a security decision, not a convenience decision.
- Prefer mainstream, widely adopted, well maintained packages with clear ownership and a narrow purpose.
- Reject obscure, weakly maintained, speculative, or low-rigor packages unless there is a documented exception with explicit risk acceptance.
- Scanner output is necessary but not sufficient. Review trust, maintenance quality, release hygiene, and transitive impact.
- Review lockfiles and dependency trees, not just manifest diffs.

## Enforced Repository Policy

- `cargo audit` is required for Rust advisory scanning.
- `cargo deny check` is required for advisory, bans, license, and source policy enforcement.
- `cargo run -p tq-release --locked -- verify-dependabot --repo-root .` enforces Dependabot coverage for GitHub Actions surfaces.
- CI also runs `gitleaks` and `detect-secrets`.
- External GitHub Actions must be pinned to full 40-character SHAs.
- Security scanners are installed on the stable toolchain through `.github/actions/setup-rust-security-tools` rather than the product MSRV.

## Current High-Signal Deny Policy

Review `deny.toml` for the authoritative policy. Today it explicitly bans:

- `serde_yaml` because it is deprecated upstream; use `serde_yaml_ng` instead.
- `serde_yml` because it is unsound and unmaintained; use `serde_yaml_ng` instead.

When a review touches dependency selection, check whether the proposed crate belongs in `deny.toml` or conflicts with an existing ban.
