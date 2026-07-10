# Developer Harness

The developer harness is the repository-owned entrypoint for deterministic maintenance work. Use it when a task manages local setup, pinned tools, dependency freshness, security audits, cleanup, or release packaging.

Run commands through the Cargo alias:

```bash
cargo dev <group> <command>
```

## Command groups

- `cargo dev policy verify-pins --repo-root .` checks repository-owned tool pins and configuration surfaces. This is CI-safe and is included in release-policy validation.
- `cargo dev policy audit-external-pins --repo-root .` checks frozen GitHub Action and pre-commit pins against the latest upstream SemVer release tags.
- `cargo dev health doctor --repo-root .` checks the local developer environment against pinned tools and native build prerequisites.
- `cargo dev health cleanup --repo-root .` removes obsolete project Rust toolchains and harness build caches.
- `cargo dev deps audit-latest --repo-root .` reports dependency and toolchain drift without changing files.
- `cargo dev deps update --repo-root .` applies deterministic dependency and toolchain updates.
- `cargo dev deps audit-security --repo-root .` runs the pinned Rust and npm security audit tools.
- `cargo dev release build --repo-root .` builds the local release sdist and host wheel.

## Output modes

Reporting commands support `--output human`, `--output json`, `--output agent`, and `--quiet`.

- `human` is the default terminal UI.
- `json` is for automation that needs a stable machine-readable report.
- `agent` is compact structured text for coding agents and issue summaries.
- `quiet` suppresses success output while preserving findings, stderr errors, and non-zero exit codes.

## Design rule

When repository maintenance becomes repetitive, surprising, or easy to get wrong, prefer adding a deterministic harness command or check over adding more prose. Keep CI on repo-policy and check-only commands. Keep local machine mutation in explicit `setup`, `cleanup`, and `update` commands.
