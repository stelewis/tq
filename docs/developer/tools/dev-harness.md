# Developer Harness

The developer harness is the repository-owned entrypoint for deterministic maintenance. Use it when work touches validation, setup, pinned tools, dependency freshness, security audits, cleanup, release packaging, or runtime dependency policy.

Run commands through the Cargo alias:

```bash
cargo dev <group> <command>
```

## Command Groups

### `check`

Runs deterministic validation tasks from the harness-owned check catalog.

| Command | Purpose |
| --- | --- |
| `cargo dev check routine --repo-root .` | Fast daily Rust gate. |
| `cargo dev check all --repo-root .` | Broad repository gate. |
| `cargo dev check --profile full all --repo-root .` | Release-sensitive gate, including release build validation. |
| `cargo dev check docs --repo-root .` | Documentation synchronization gate. |
| `cargo dev check release-policy --repo-root .` | Release policy gate. |
| `cargo dev check release-build --repo-root .` | Release artifact build gate. |

Use `--dry-run` to print the resolved check plan.

### `deps`

Audits and updates repository dependency state.

| Command | Purpose |
| --- | --- |
| `cargo dev deps audit-latest --repo-root .` | Reports dependency and toolchain drift without changing files. |
| `cargo dev deps audit-maintenance-tools --repo-root .` | Reports drift in pinned Rust maintenance tools. |
| `cargo dev deps audit-security --repo-root .` | Runs Rust, Python, and npm dependency vulnerability audits. |
| `cargo dev deps update --repo-root .` | Applies deterministic dependency and toolchain updates. |

`deps update` updates repository-owned dependency state: Rust pin files when a stable toolchain update exists, the selected Rust toolchain, mise-installed tools, the uv lockfile, npm dependencies, and frozen pre-commit hook pins. It does not run `uv self update`, because uv executable updates depend on how the developer installed uv. It does not run `uv python upgrade`, because that mutates uv-managed Python installations, is limited to patch upgrades, and is still a preview uv feature. Local Python availability belongs to `setup` and `health doctor`.

Use `--dry-run` with `deps update` to print the action plan without applying it.

### `health`

Checks or cleans the local developer environment.

| Command | Purpose |
| --- | --- |
| `cargo dev health doctor --repo-root .` | Checks local tools and native build prerequisites against repository pins. |
| `cargo dev health cleanup --repo-root .` | Removes repository-owned harness caches. |

`health doctor` checks the installed uv executable against the repository uv pin. If uv is missing or mismatched, update it through the package manager that owns the installation. For Homebrew installs, uv reports `brew update && brew upgrade uv`. Use `setup` to install the pinned Python version through uv when it is missing.

Use `--dry-run` with `health cleanup` to print the cleanup plan without removing files.

### `policy`

Verifies repository policy invariants.

| Command | Purpose |
| --- | --- |
| `cargo dev policy verify-pins --repo-root .` | Verifies repository-owned tool pins and configuration surfaces. |
| `cargo dev policy verify-release --repo-root .` | Runs the aggregate release policy gate. |
| `cargo dev policy verify-dependabot --repo-root .` | Verifies Dependabot coverage. |
| `cargo dev policy verify-workspace-version --repo-root .` | Verifies workspace version consistency. |
| `cargo dev policy audit-external-pins --repo-root .` | Reports drift in frozen GitHub Action and pre-commit pins. |

Policy verification commands are CI-safe and do not mutate the repository.

### `release`

Builds and verifies release artifacts.

| Command | Purpose |
| --- | --- |
| `cargo dev release build --repo-root .` | Builds the local release sdist and host wheel. |
| `cargo dev release build-tool-requirements --repo-root .` | Prints the exact Python build tool requirements from `.github/dev-tools.toml` as GitHub output. |
| `cargo dev release verify-artifacts --dist-dir dist` | Verifies release artifact contents against repository policy. |

Use `--dry-run` with `release build` to print the artifact build plan without changing `dist/`.

### `setup`

Installs pinned developer toolchain prerequisites:

```bash
cargo dev setup --repo-root .
```

Use `--dry-run` to print the setup plan, including pinned Rust maintenance-tool installs that are currently needed.

### `runtime-deps`

Reports whether the shipped CLI dependency graph changed between two git refs:

```bash
cargo dev runtime-deps --repo-root . --base-ref <base> --head-ref <head>
```

Use this before release-facing commits to decide whether the change needs a release-bearing Conventional Commit type.

## Shared Options

Reporting commands support:

| Option | Purpose |
| --- | --- |
| `--repo-root <path>` | Resolves repository files from the given root. Defaults to `.`. |
| `--output human` | Prints aligned terminal output. This is the default. |
| `--output json` | Prints stable JSON for automation. |
| `--output agent` | Prints Markdown for coding agents, issue comments, and PR summaries. |
| `--quiet` | Suppresses clean success output while preserving findings, stderr errors, and non-zero exit codes. |

Mutating commands support `--dry-run` when they have an action plan. The rendered plan is the same ordered action list that apply mode executes.

## Exit Codes

Every harness command follows the same exit-code contract:

| Code | Meaning |
| --- | --- |
| `0` | Clean, passed, or applied successfully. |
| `1` | Findings, failed checks, or unhealthy environment. |
| `2` | Harness error, invalid input, missing file, or command execution failure. |

## Design Rule

When repository maintenance becomes repetitive, surprising, or easy to get wrong, add a deterministic harness command or check instead of more prose. Keep CI on read-only policy and check commands. Keep local machine mutation in explicit `setup`, `cleanup`, and `update` commands.
