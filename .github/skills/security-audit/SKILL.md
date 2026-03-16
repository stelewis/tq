---
name: security-audit
description: Audit tq changes and repository state for codebase security and supply-chain risk. Use when a user asks for a security review, dependency or lockfile assessment, CI or release hardening, secret exposure review, or validation against this repository's code and supply-chain security standards.
argument-hint: describe the scope, changed files, dependency or workflow surface, and the risk or question to audit
user-invocable: true
disable-model-invocation: false
---

# Security Audit

Use this skill to review tq for runtime security flaws, release and CI trust issues, and dependency admission mistakes.

## When To Use It

- The user asks for a security review, audit, hardening pass, threat review, or supply-chain assessment.
- The change touches dependencies, lockfiles, `deny.toml`, GitHub Actions, release automation, archive handling, filesystem traversal, config parsing, process execution, or diagnostic redaction.
- The task is to decide whether a dependency, workflow change, or security exception meets this repository's standards.

## Workflow

1. Read [references/repo-standards.md](references/repo-standards.md) before reviewing code so the audit is anchored to repository policy, not generic instincts.
2. Classify the audit scope early: runtime code, supply chain, CI and workflows, release artifacts, secrets exposure, or a mixed review.
3. Inspect the smallest relevant surface first. For a PR or diff, start with the changed files and then pull in the governing policy and any directly affected call sites.
4. For runtime code, look for boundary validation, fail-closed behavior, path traversal and symlink escape, unsafe subprocess construction, archive extraction risk, hidden IO in domain logic, and secret leakage in logs or errors.
5. For supply-chain reviews, treat trust and ownership as first-class signals. Inspect dependency purpose, maintainer reputation, release history, transitive impact, license and source policy fit, and whether local code would be safer and simpler.
6. Run the relevant repository checks from [references/security-review-playbook.md](references/security-review-playbook.md) when the environment permits. Use the narrowest command set that can actually prove or falsify the suspected risk.
7. If the review depends on current external guidance, consult [references/live-security-sources.md](references/live-security-sources.md) and fetch the relevant sources before finalizing the findings.
8. Report findings first, ordered by severity, with the violated invariant or policy called out explicitly. If there are no findings, say that directly and note any residual risks or checks you could not run.

## Execution Rules

- Do not treat passing scanners as sufficient evidence that a dependency or workflow is safe.
- Prefer deleting or banning risky components over documenting or suppressing them.
- Cite the exact repository standard or policy that the change violates whenever one exists.
- Fix root causes. Do not settle for narrow suppressions, broad allowlists, or compatibility shims that preserve unsafe behavior.
- Keep recommendations concrete: name the command to run, the file to inspect, the dependency to reject, or the boundary to tighten.

## References

- Repository standards and enforced policy: [references/repo-standards.md](references/repo-standards.md)
- Audit workflow and command checklist: [references/security-review-playbook.md](references/security-review-playbook.md)
- Live external guidance to fetch when needed: [references/live-security-sources.md](references/live-security-sources.md)
