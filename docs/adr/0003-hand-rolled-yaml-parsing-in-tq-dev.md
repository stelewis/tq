---
id: 0003
title: "Hand-rolled YAML parsing in the developer harness"
status: accepted
date: 2026-07-11
tags: [security, supply-chain, tooling]
supersedes: null
superseded_by: null
---

## Context

The `tq-dev` harness reads four repository-owned YAML surfaces: composite GitHub Action definitions (`inputs` defaults and step `with` values), `uses:` references in workflows, `repo`/`rev` pairs in `.pre-commit-config.yaml`, and workflow job/permission structure. These checks verify pinned tool versions, frozen commit SHAs, bounded jobs, and least-privilege automation.

The Rust YAML ecosystem does not currently offer a parser that meets the [supply-chain security standards](../developer/standards/supply-chain-security.md). `serde_yaml` is deprecated and unmaintained. Several successors exist, but none combines broad adoption, verifiable ownership, and a sustained maintenance record; this repository previously admitted an insecure crate that presented itself as a `serde_yaml` successor, which is exactly the failure mode the admission bar exists to prevent.

## Decision

Parse these YAML surfaces with deliberately limited, line-oriented readers owned by `tq-dev`, rather than admitting a YAML dependency. `parse.rs` owns lexical parsing and typed external references; `workflow_policy.rs` owns workflow structure invariants. Policy checks and drift audits consume these shared boundaries instead of reparsing YAML independently.

The parsers support only the shapes this repository commits: two-space indentation, scalar values on the same line as their key, single- or double-quoted scalars, and `#` comments. They do not support block scalars, flow collections, anchors, aliases, or multi-line values.

Review this if a future YAML crate clears the admission bar, or if the repository's YAML surfaces change to require a more complete parser.

## Consequences

- No YAML crate enters the trusted computing base for repository automation.
- The repository's own YAML must stay within the supported subset. The `cargo dev policy verify-pins` and `cargo dev policy verify-automation` gates exercise these parsers against the live files, so a formatting change that breaks parsing fails loudly rather than silently skipping a check.
- Parsing is strict where it matters (missing keys, defaults, permissions, and job timeouts are hard errors) but structurally naive: unusual formatting could misparse. This risk is bounded because the inputs are repository-owned files that pass through code review, not untrusted input.
- If a YAML crate later clears the dependency admission bar, replacing these readers is a contained change because all YAML reading lives behind `parse.rs`.

## Alternatives considered

- **`serde_yaml`**: deprecated and archived; fails the maintenance requirement.
- **Successor crates (`serde_yml`, others)**: low-trust ownership and weak release rigor; one was previously admitted by mistake and removed. Fails the admission bar.
- **`yaml-serde`**: actively maintained fork of `serde_yaml` by the YAML organization; potential future candidate.
- **`yaml-rust2`**: maintained fork with moderate adoption, but a full YAML implementation is a large transitive surface for three narrow, repository-owned read paths. The value does not justify the trust cost today.
- **Shelling out to Python/Ruby YAML loaders**: reintroduces an interpreter dependency into policy checks and makes the harness non-deterministic across environments; this is what the Rust harness replaced.

## Related

- [Supply-chain security standards](../developer/standards/supply-chain-security.md)
- `crates/tq-dev/src/parse.rs`
- [We lost `serde_yaml`, what's the next one?](https://www.reddit.com/r/rust/comments/1bo5dle/we_lost_serdeyaml_whats_the_next_one/)
