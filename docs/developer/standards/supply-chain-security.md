# Supply-Chain Security Standards

Dependencies are part of the trusted computing base for this repository. Every new crate, Python package, Node package, GitHub Action, and build or release tool expands the attack surface, review surface, and maintenance burden.

The default posture is conservative:

- prefer mainstream, boring, widely adopted, actively maintained dependencies,
- prefer mature projects with a long public history over newer projects with little operational track record,
- prefer packages with clear ownership, disciplined release engineering, and straightforward purpose,
- avoid adding dependencies when the standard library or an existing approved package is sufficient.

The project should reject dependencies that feel speculative, weakly maintained, low-trust, or hastily assembled. That includes projects with unclear ownership, minimal maintenance history, weak release hygiene, inconsistent quality signals, or codebases that appear generated or stitched together without strong engineering discipline.

## Admission Standard

New dependencies must satisfy all of the following unless an explicit exception is documented in the pull request:

- **Clear need**: the dependency solves a real problem that we should not solve ourselves with a small amount of clear code.
- **High trust**: the dependency is widely known in its ecosystem, broadly used, and maintained by identifiable owners with a credible history.
- **Maintenance evidence**: the project shows sustained releases, active issue or PR handling, and no obvious signs of abandonment.
- **Quality evidence**: the project has readable source, coherent scope, tests, changelog or release notes, and issue tracking that indicates engineering rigor.
- **Security evidence**: the dependency has no unresolved critical security advisories that affect our use case, and it fits within our existing audit and policy tooling.
- **Operational fit**: licensing, transitive dependency cost, platform support, artifact model, and update cadence fit this repository.

If any of those are weak, the answer is normally no.

## Default Rejection Cases

Do not add a dependency when one or more of these are true unless a maintainer explicitly approves a documented exception:

- the project is very new and has little release history,
- the package is niche, obscure, or lightly adopted relative to established alternatives,
- ownership is anonymous, unstable, or difficult to verify,
- the source or release process appears sloppy, inconsistent, or low rigor,
- the value proposition is vague or duplicates a small amount of code we can own ourselves,
- the package pulls in a large or poorly understood transitive tree,
- the package has a pattern of unreviewed breaking changes, yanked releases, or unresolved security concerns,
- the project looks like generated output, AI-produced glue, or otherwise "hacked together" code without disciplined engineering.

## Review Criteria

When evaluating a dependency, review the package and its repository using these signals together. No single metric is enough.

### Trust and adoption

- registry downloads or install volume,
- reverse dependency count or broad ecosystem usage where available,
- usage by respected projects or organizations,
- name recognition within the ecosystem.

### Maintenance and ownership

- age of the project,
- release cadence over time, not just one recent release,
- named maintainers with a visible track record,
- responsiveness to issues, bugs, and security reports,
- evidence that releases are intentional and documented.

### Engineering quality

- clear README and scope,
- coherent source layout and test coverage,
- changelog or release notes with meaningful content,
- explicit compatibility and platform guidance,
- no obvious signs of copy-paste or low-discipline implementation.

### Security and supply-chain posture

- published advisories checked and understood,
- lockfile support and deterministic installation,
- reasonable transitive tree size,
- no suspicious install scripts, network behavior, or opaque binary delivery unless clearly justified,
- provenance, signatures, or attestation support when the ecosystem provides them.

## Toolchain Guidance

## Rust

Rust dependencies must be especially conservative because they affect the main product workspace.

Preferred profile:

- established crates from well-known maintainers or organizations,
- strong documentation and narrow scope,
- broad ecosystem adoption,
- clean `cargo tree` output with understandable transitives,
- compatibility with `cargo audit`, `cargo deny`, and lockfile-based review.

Required checks before adding or upgrading:

- inspect the crate on crates.io,
- inspect the source repository and recent release history,
- inspect `cargo tree` impact,
- run `cargo audit`,
- run `cargo deny check`,
- review version drift with `cargo outdated --workspace --root-deps-only` when relevant.

Useful indicators:

- crates.io downloads,
- reverse dependency count and ecosystem presence,
- RustSec advisory history,
- maintainer reputation,
- whether the crate is widely used by mature Rust projects.

Rust red flags:

- very recent crates without track record,
- forks with unclear maintenance intent,
- stringly utility crates that replace simple local code,
- hidden `unsafe` or FFI-heavy implementation without a strong reason,
- unusually large transitive trees for a small feature.

## Python

Python packages in this repository are primarily packaging, release, and developer tooling. They still affect supply-chain risk and CI trust.

Preferred profile:

- established tooling with strong community recognition,
- active maintainers and regular releases,
- clear PyPI metadata and repository links,
- stable command-line behavior and documented support policy,
- narrow, understandable dependency footprint.

Required checks before adding or upgrading:

- inspect the project on PyPI,
- inspect the source repository and release history,
- review the dependency graph through `uv.lock` and `uv tree` when available,
- confirm the package fits our `uv`-managed workflow,
- review known advisories and open security issues.

Useful indicators:

- PyPI release history,
- maintainers and project URLs,
- adoption by respected projects,
- evidence of typed, tested, and documented releases,
- minimal reliance on fragile installer-time behavior.

Python red flags:

- beta-grade tools without strong adoption,
- packages with unclear ownership transfer or thin release notes,
- sprawling dependency chains for a narrow task,
- wrappers around generated or opaque binaries with weak provenance.

## Node

Node dependencies are currently limited, and they should stay that way. The Node ecosystem has a higher baseline supply-chain risk because tiny packages can carry outsized transitive impact.

Preferred profile:

- ecosystem-standard packages with long history,
- maintainers who are well known in the framework or tooling community,
- minimal transitive expansion,
- no unnecessary postinstall behavior,
- deterministic lockfile behavior through `package-lock.json`.

Required checks before adding or upgrading:

- inspect the package on npm,
- inspect the source repository and maintainers,
- inspect transitive impact in `package-lock.json`,
- run `npm audit --package-lock-only` when npm is available,
- review install scripts and binary delivery model.

Useful indicators:

- npm weekly downloads,
- maintainer continuity,
- whether the package is the normal choice in the surrounding ecosystem,
- published provenance or attestation support where available.

Node red flags:

- tiny utility packages that replace a few lines of local code,
- deep dependency trees for simple functionality,
- install-time scripts with broad system access,
- packages that are popular only because of trend momentum rather than durable trust.

## Pull Request Expectations

Any pull request that adds or materially changes an external dependency should explain:

- why the dependency is needed,
- what alternatives were rejected and why,
- why the chosen package is trustworthy,
- what its transitive impact is,
- what checks were run.

If the dependency is not an obvious ecosystem staple, the pull request should include a short written justification covering adoption, maintenance history, ownership, and security posture.

## Existing Enforcement

These standards are supported, but not fully replaced, by automation:

- `cargo audit` for Rust advisory scanning,
- `cargo deny check` for Rust advisory, bans, license, and source policy,
- `cargo outdated --workspace --root-deps-only` for Rust version drift,
- Dependabot coverage policy for GitHub Actions surfaces,
- `detect-secrets` and `gitleaks` for secret scanning,
- lockfiles for Rust, Python, and Node dependency review.

Automation is the floor, not the bar. A dependency can pass scanners and still be a bad choice.

## Exceptions

Exceptions should be rare. When an exception is necessary, document all of the following in the pull request:

- the reason an established alternative was not sufficient,
- the specific risks we are accepting,
- the controls we will use to monitor that dependency,
- the exit path if the project degrades or a better alternative appears.

Absent that level of justification, do not merge the dependency.
