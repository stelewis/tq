# Developer Harness

The developer harness is the repository-owned entrypoint for deterministic maintenance. Use it when work touches validation, repository setup, pinned tool policy, dependency freshness, security audits, release packaging, or runtime dependency policy.

Run commands through the Cargo alias:

```bash
cargo dev <group> <command>
```

## Command Groups

### `check`

Runs deterministic validation tasks from the harness-owned check catalog.

| Command | Purpose |
| --- | --- |
| `cargo dev check routine --repo-root .` | Fast daily Rust, actionlint, ShellCheck, and automation-policy gate. |
| `cargo dev check all --repo-root .` | Broad repository gate. |
| `cargo dev check --profile full all --repo-root .` | Release-sensitive gate, including workflow lint, automation policy, and release build validation. |
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

`deps update` updates repository-owned dependency state: Rust pin files when a stable toolchain update exists, the uv lockfile, npm dependencies, and frozen pre-commit hook pins. It does not update installed executables or language runtimes; those belong to the environment manager that installed them.

Use `--dry-run` with `deps update` to print the action plan without applying it.

### `health`

Checks the local developer environment.

| Command | Purpose |
| --- | --- |
| `cargo dev health doctor --repo-root .` | Checks local tools and native build prerequisites against repository pins. |

`health doctor` reads tool versions from repository-owned metadata and checks the corresponding executables on `PATH`. Missing or mismatched tools must be installed or updated through the environment manager that owns the developer machine. The harness reports environment state but does not provision global tools or language runtimes.

### `policy`

Verifies repository policy invariants.

| Command | Purpose |
| --- | --- |
| `cargo dev policy verify-pins --repo-root .` | Verifies repository-owned tool pins and configuration surfaces. |
| `cargo dev policy verify-automation --repo-root .` | Verifies tracked path coverage, workflow hardening, immutable refs, and active Dependabot coverage. |
| `cargo dev policy verify-release --repo-root .` | Runs the aggregate release policy gate. |
| `cargo dev policy verify-dependabot --repo-root .` | Verifies Dependabot coverage. |
| `cargo dev policy verify-workspace-version --repo-root .` | Verifies workspace version consistency. |
| `cargo dev policy verify-action-pins --repo-root .` | Verifies external GitHub Actions use full commit SHAs. |
| `cargo dev policy verify-pre-commit-pins --repo-root .` | Verifies external pre-commit repositories use full commit SHAs. |
| `cargo dev policy audit-external-pins --repo-root .` | Reports drift in frozen GitHub Action and pre-commit pins. |

Policy verification commands are CI-safe and do not mutate the repository.

### `release`

Builds and verifies release artifacts.

| Command | Purpose |
| --- | --- |
| `cargo dev release build --repo-root .` | Builds the local release sdist and host wheel. |
| `cargo dev release build-tool-requirements --repo-root .` | Prints the exact Python build tool requirements from `.github/dev-tools.toml` as GitHub output. |
| `cargo dev release verify-artifacts --dist-dir dist --profile <expected-profile>` | Verifies the expected artifact set, platform tags, and archive contents. |

Artifact profiles cover an sdist, each platform wheel, a platform wheel with an sdist, and the complete publishable release set. Use `cargo dev release verify-artifacts --help` for the accepted profile names.

Use `--dry-run` with `release build` to print the artifact build plan without changing `dist/`.

### `setup`

Installs locked repository dependencies and pre-commit hooks:

```bash
cargo dev setup --repo-root .
```

The setup plan runs `uv sync --locked`, `npm ci --ignore-scripts`, and hook installation. It does not install or update system tools, language runtimes, or global Cargo binaries. Use `--dry-run` to inspect the exact repository mutations.

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

When repository maintenance becomes repetitive, surprising, or easy to get wrong, add a deterministic harness command or check instead of more prose. The harness owns repository state, validation, diagnosis, and policy. Developer environment managers and CI setup actions own machine provisioning.
