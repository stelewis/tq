# Security Review Playbook

Use the narrowest review path that matches the change.

## 1. Runtime Code Review

Start here when the change touches Rust code, archive handling, filesystem paths, process execution, config parsing, or diagnostics.

- Identify the trust boundary: CLI input, config file, filesystem, archive, environment, subprocess, or release artifact.
- Trace how untrusted data is parsed, validated, normalized, and rejected.
- Check for fail-open behavior, silent defaults, broad error collapsing, or convenience coercions.
- Review path joins, canonicalization, extraction, and temp handling for traversal or boundary escape.
- Review errors and logs for secret disclosure or unnecessary sensitive path exposure.
- Confirm ordering and output remain deterministic when security-sensitive diagnostics are serialized or asserted.

## 2. Dependency And Lockfile Review

Start here when the change adds or upgrades crates, Python packages, Node packages, or CI tools.

- Ask whether the dependency is necessary. If a small amount of local code or an existing approved package covers the need, prefer that.
- Check trust signals: adoption, ownership, maintenance history, release cadence, issue handling, and overall code quality.
- Inspect transitive impact. Large or surprising trees are findings until justified.
- Check whether the package conflicts with repository policy, especially `deny.toml` bans, source policy, or license allowlists.
- Review version drift separately from advisories. A package can be stale or deprecated without a published RustSec advisory.

Useful commands:

- `cargo audit`
- `cargo deny check`
- `cargo tree`
- `cargo outdated --workspace --root-deps-only`

## 3. CI And Workflow Review

Start here when the change touches `.github/workflows`, `.github/actions`, release automation, or installer/bootstrap code.

- Verify all external `uses:` references are pinned to full SHAs.
- Verify Dependabot coverage still includes workflows and local composite actions.
- Review any new download, install, or execution step for trust, pinning, provenance, and shell-injection risk.
- Check whether security scanners are being weakened, bypassed, or moved behind optional paths.
- Review tool bootstrap choices separately from product runtime choices. CI security tooling may intentionally use a newer stable toolchain.
- Review `pre-commit-config.yaml` for hook quality and whether they meet the same security standards.

Useful commands:

- `cargo run -p tq-release --locked -- verify-dependabot --repo-root .`
- Review `.github/actions/setup-rust-security-tools/action.yml`
- Review `.github/workflows/ci.yml` and `.github/workflows/publish.yml`

## 4. Release And Artifact Review

Start here when the change touches packaging, publish steps, dist contents, or provenance and attestation logic.

- Verify artifact content remains minimal and excludes repository-only paths.
- Verify provenance generation and verification still gate publish.
- Verify smoke tests run against built artifacts, not only the source tree.
- Review any new packaging dependency against the same supply-chain bar as runtime code.

Useful commands:

- `cargo package --workspace --locked`
- `uv build`
- `cargo run -p tq-release --locked -- verify-artifact-contents --dist-dir dist`

## 5. Findings Format

When you report the audit:

- Put findings first, ordered by severity.
- State the concrete risk, the affected file and behavior, and the violated standard or policy.
- Name the missing validation, policy check, or safer alternative.
- If no findings are discovered, state that explicitly and mention residual risks, assumptions, or checks that could not be run.
