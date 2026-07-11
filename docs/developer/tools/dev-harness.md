# Developer Harness

The developer harness is the repository-owned entrypoint for deterministic maintenance work. Use it when a task manages local setup, pinned tools, dependency freshness, security audits, cleanup, or release packaging.

Run commands through the Cargo alias:

```bash
cargo dev <group> <command>
```

## Command groups

- `cargo dev check [target] --repo-root .` runs repository validation through a deterministic harness-owned task catalog. Use `routine` for the daily Rust gate, `all` for the broad repo gate, or target `docs`, `release-policy`, and `release-build` directly when those surfaces are implicated.
- `cargo dev policy verify-pins --repo-root .` checks repository-owned tool pins and configuration surfaces. This is CI-safe and is included in release-policy validation.
- `cargo dev policy verify-release --repo-root .` runs the full release-policy gate: workspace version consistency, Dependabot coverage, and pinned tool surfaces.
- `cargo dev policy audit-external-pins --repo-root .` checks frozen GitHub Action and pre-commit pins against the latest upstream SemVer release tags.
- `cargo dev health doctor --repo-root .` checks the local developer environment against pinned tools and native build prerequisites.
- `cargo dev health cleanup --dry-run --repo-root .` prints the cleanup action plan for repository-owned harness caches.
- `cargo dev health cleanup --repo-root .` removes repository-owned harness build caches.
- `cargo dev setup --dry-run --repo-root .` prints the setup commands that would run, including any currently needed pinned Cargo maintenance-tool installs.
- `cargo dev deps audit-latest --repo-root .` reports dependency and toolchain drift without changing files.
- `cargo dev deps audit-maintenance-tools --repo-root .` reports drift in the pinned Rust maintenance tools installed by the shared setup action.
- `cargo dev deps update --dry-run --repo-root .` prints the dependency update action plan, including command actions and any Rust pin file edits.
- `cargo dev deps update --repo-root .` applies deterministic dependency and toolchain updates.
- `cargo dev deps audit-security --repo-root .` runs the pinned Rust and npm security audit tools.
- `cargo dev release build --dry-run --repo-root .` prints the local release artifact build plan without changing `dist/`.
- `cargo dev release build --repo-root .` builds the local release sdist and host wheel.
- `cargo dev release verify-artifacts --dist-dir dist` verifies release artifact contents against the repository policy.
- `cargo dev runtime-deps --repo-root . --base-ref <base> --head-ref <head>` reports whether the shipped CLI dependency graph changed between two git refs.

## Exit codes

Every harness command follows one exit-code contract so automation can branch without parsing output:

- `0`: clean, passed, or applied successfully
- `1`: findings — drift detected, checks failed, or environment unhealthy
- `2`: harness error — invalid input, missing file, or a command that could not run

## Output modes

Reporting commands support `--output human`, `--output json`, `--output agent`, and `--quiet`. The check, setup, dependency update, health cleanup, and release build commands support `--dry-run` to print the resolved plan without executing commands.

- `human` is the default terminal UI.
- `json` is for automation that needs a stable machine-readable report. Report payloads include a typed `summary` plus the ordered check results.
- `agent` is Markdown structured for coding agents, issue comments, and PR summaries.
- `quiet` suppresses clean success output while preserving findings, stderr errors, and non-zero exit codes.

## Design rule

When repository maintenance becomes repetitive, surprising, or easy to get wrong, prefer adding a deterministic harness command or check over adding more prose. Keep CI on repo-policy and check-only commands. Keep local machine mutation in explicit `setup`, `cleanup`, and `update` commands.
