# Supply-Chain Security Standards

Use this page for dependency-admission policy. For the broader repository security model, see [Security standards](./security.md).

Dependencies are part of the trusted computing base. Adding or upgrading one is a security decision, not a convenience refactor.

## Default posture

The default posture is conservative:

- prefer no new dependency when the standard library or clear local code is sufficient,
- prefer mainstream, well-established, widely used packages with clear ownership and disciplined release hygiene,
- prefer narrow scope and understandable transitive impact,
- reject low-trust, weakly maintained, opaque, or hastily assembled packages by default.

Packages that look speculative, obscure, AI-generated, or weakly reviewed should normally be rejected even if they appear to solve the immediate problem.

## Admission bar

New dependencies must satisfy all of the following unless a maintainer approves a documented exception:

- **Clear need**: the dependency solves a problem we should not own with a small amount of local code.
- **High trust**: the package is well known in its ecosystem, broadly adopted, and maintained by identifiable owners.
- **Maintenance evidence**: the project shows sustained releases, active maintenance, and no obvious abandonment.
- **Quality evidence**: the source, tests, scope, and release notes show engineering rigor.
- **Security fit**: known advisories, install behavior, provenance, and transitive impact are acceptable for this repository.
- **Operational fit**: licensing, platform support, lockfile review, and update cadence fit the repository workflow.

If one of these is weak, the default answer is no.

## Default rejection cases

Do not add a dependency when one or more of these are true unless a maintainer explicitly approves a documented exception:

- the project is very new and has little release history,
- the package is niche, obscure, or lightly adopted relative to established alternatives,
- ownership is anonymous, unstable, or difficult to verify,
- the source or release process appears sloppy, inconsistent, or low rigor,
- the value proposition is vague or duplicates a small amount of code we can own ourselves,
- the package pulls in a large or poorly understood transitive tree,
- the package has a pattern of unreviewed breaking changes, yanked releases, or unresolved security concerns,
- the project looks like generated output, AI-produced glue, or otherwise "hacked together" code without disciplined engineering.

## Review requirements

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

## Required ecosystem-specific checks

- **Rust**: inspect `cargo tree`, run `cargo audit`, run `cargo deny check`, and review `cargo outdated --workspace --root-deps-only` when version drift matters.
- **Python**: review `uv.lock` and `uv tree` when available, inspect PyPI metadata and release history, and review known advisories.
- **Node**: inspect `package-lock.json`, review install scripts and binary delivery, and run `npm audit --package-lock-only` when npm is available.

## Pull request expectations

Any pull request that adds or materially changes an external dependency should explain:

- why the dependency is needed,
- what alternatives were rejected and why,
- why the chosen package is trustworthy,
- what the transitive impact is,
- what checks were run.

If the package is not an obvious ecosystem staple, the pull request should include a short justification covering adoption, maintenance history, ownership, and security posture.

## Enforcement

Automation supports this policy but does not replace it:

- `cargo audit`
- `cargo deny check`
- `cargo outdated --workspace --root-deps-only`
- Dependabot coverage policy for GitHub Actions
- `detect-secrets` and `gitleaks`
- Rust, Python, and Node lockfiles

Automation is the floor, not the bar.

## Exceptions

Exceptions should be rare. When needed, document:

- why established alternatives were insufficient,
- the specific risks being accepted,
- the controls that will monitor the dependency,
- the exit path if the package degrades or a better option appears.

Without that level of justification, do not merge the dependency change.
