# Developer Harness

The developer harness is the repository-owned interface for validation, setup, dependency maintenance, policy checks, environment diagnosis, and release artifacts.

```bash
cargo dev <command>
```

Use `cargo dev --help` and subcommand help for the complete command surface. The CLI is the source of truth for available options and artifact profiles.

## Choose a Workflow

| Goal | Command |
| --- | --- |
| Run the daily quality gate | `cargo dev check routine --repo-root .` |
| Run all repository checks | `cargo dev check all --repo-root .` |
| Include release artifact validation | `cargo dev check --profile full all --repo-root .` |
| Install locked repository dependencies and hooks | `cargo dev setup --repo-root .` |
| Diagnose the local toolchain | `cargo dev health doctor --repo-root .` |
| Audit dependency freshness | `cargo dev deps audit-latest --repo-root .` |
| Audit dependency vulnerabilities | `cargo dev deps audit-security --repo-root .` |
| Plan or apply dependency updates | `cargo dev deps update --dry-run --repo-root .` |
| Verify repository automation policy | `cargo dev policy verify-automation --repo-root .` |
| Audit frozen external pins | `cargo dev policy audit-external-pins --repo-root .` |
| Build local release artifacts | `cargo dev release build --repo-root .` |

Use `cargo dev runtime-deps --repo-root . --base-ref <base> --head-ref <head>` to determine whether a change alters the shipped CLI dependency graph.

Detailed pin rotation belongs in [Pin maintenance](./pin-maintenance.md). Artifact profiles and publishing contracts belong in [Docs and release](./docs-and-release.md).

## Plans and Output

Commands that mutate repository state support `--dry-run`. A dry run renders the same ordered action plan used by apply mode without executing its mutations. Use it before setup, cleanup, dependency updates, or release builds when the affected state is not already clear.

Reporting commands support three output formats:

- `human` for terminal output
- `json` for automation
- `agent` for Markdown summaries

Use `--quiet` to suppress clean success output without suppressing findings, errors, or non-zero exit codes.

## Exit Codes

| Code | Meaning |
| --- | --- |
| `0` | The command passed, found no drift, or applied its plan. |
| `1` | The command found drift, failed checks, or an unhealthy environment. |
| `2` | The harness could not complete because its input, repository state, or command execution was invalid. |

## Ownership Boundary

The harness may inspect machine state, but it mutates only repository-owned state and generated repository artifacts. It does not install or remove system tools, language runtimes, or global Cargo binaries. Local environment managers and CI setup actions own those machine-level changes; `cargo dev health doctor` reports the responsible remediation.

When maintenance becomes repetitive, surprising, or easy to perform incorrectly, encode the solution in the harness instead of adding a manual procedure.
