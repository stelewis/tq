# Versioning

Policy for versioning user-visible `tq` contracts.

## Contract surfaces

This policy covers:

- CLI flags and behavior (`tq check`)
- Configuration keys and validation semantics (`[tool.tq]`)
- Rule IDs and default severities
- Exit code semantics
- JSON diagnostic schema fields

## Versioning model

`tq` is pre-`1.0`. Until `1.0`, the project uses this compatibility model:

- `patch` (`0.x.Y`): non-breaking fixes and additive changes that do not change the meaning of existing contracts
- `minor` (`0.X.0`): contract-impacting changes, including intentional breaking changes

After `1.0`, major-version SemVer semantics apply for breaking changes.

## Internal workspace crate policy

The Rust workspace uses one shared version for all internal crates.

- Internal crate APIs are current-only. They are not compatibility surfaces.
- When a workspace crate changes a public API consumed by another workspace crate, update all internal callers in the same change.
- Do not preserve old internal APIs with shims, aliases, deprecated wrappers, or dual-path call sites.
- If an internal public API changes, bump the shared workspace version as a minor pre-`1.0` release before packaging or release validation.
- Version bumps must keep `workspace.package.version` and internal `workspace.dependencies.tq-*` version fields aligned. Commitizen updates the inline `workspace.dependencies.tq-*` entries in `Cargo.toml` during `cz bump`.
- `cargo run -p tq-release --locked -- verify-release-policy --repo-root .` is the release-policy gate for this policy. It must pass before packaging.
- `cargo package --workspace --locked` is the packaging gate for this policy. If it fails because a published crate with the same version no longer matches the current internal API, the correct fix is a version bump, not compatibility code.

## Release intent from Conventional Commits

Release intent is carried by [Conventional Commit](https://www.conventionalcommits.org/) messages. CI enforces commit format with `cz check`, and Commitizen (`cz bump`) derives the next version and `CHANGELOG.md` entries from commit history at release time. Commit types are the single source of truth for release intent.

Map the change to a commit type:

- `feat:` adds a stable, contract-impacting capability and selects a `minor` bump.
- `fix:` corrects shipped behavior while preserving contract meaning and selects a `patch` bump.
- `feat!:`, `fix!:`, or a `BREAKING CHANGE:` footer mark an intentional contract break; pre-`1.0` these land as a `minor` bump.
- Non-shipping types (`ci`, `build`, `chore`, `docs`, `refactor`, `test`, `style`) do not change the published artifact and do not trigger a release on their own.

Releases aggregate merged commits: a maintainer runs `cz bump` when ready, which reads the accumulated commits, computes the version, and writes the changelog. There is no requirement to bump the version inside each PR.

### Dependency updates

Classify a dependency update by whether it ships in the published CLI:

- Runtime dependency updates in the shipped Rust CLI path are shipped changes. Commit them as `fix:` (or `feat:` if they widen behavior) so the next `cz bump` releases them.
- Tooling-only updates (`pyproject.toml`, docs toolchain, GitHub Actions, pre-commit hooks, other repo automation) are not shipped. Commit them as `chore`/`build`/`ci` so they do not force a release.

Dependabot always opens dependency PRs with a `chore(deps): ...` subject, which carries no release intent on its own. CI resolves the ambiguity: the advisory `check-runtime-deps` step runs on every pull request that touches `Cargo.lock` or `Cargo.toml`, reports in its job summary whether the shipped CLI dependency graph changed, and never blocks the merge.

Handle a Dependabot Cargo PR by reading that summary:

1. Shipped graph changed: merge with a `fix: ...` subject (for example `fix: bump <dep> to <version>`) so `cz bump` includes it in the next release. Use `feat:` instead when the update widens shipped behavior.
2. Shipped graph unchanged (dev- or tooling-only dependency): merge the Dependabot `chore(deps): ...` commit as-is; no release is needed.

Every other pull request carries its own Conventional Commit type, so its release intent is already explicit and needs no extra step.

## Change classification

### Patch changes

Choose a patch release for:

- bug fixes that preserve documented contract intent
- additive optional configuration keys that are inert by default
- rule metadata updates that do not alter rule IDs, default severities, or trigger intent

### Minor changes

Choose a minor release for:

- adding a new stable rule enabled by default behavior
- changing a stable rule's trigger intent or widening behavior beyond bug-fix scope
- changing a stable rule default severity
- changing exit code semantics
- removing or renaming CLI flags, configuration keys, or rule IDs
- changing JSON output fields in a breaking way
- changing a public API in any internal workspace crate used by another workspace crate

## Governance linkage

Rule/severity procedural requirements and reference ownership/review live in [governance.md](./governance.md).

## Contract stability commitments

- Rule IDs are stable once published.
- Severity vocabulary is stable: `error`, `warning`, `info`.
- Exit code `2` remains reserved for invalid CLI/configuration and runtime IO setup errors.
- If a contract changes, docs and runtime must land together in the same PR.
