# Live Standards

Use this file when the audit depends on current external behavior or security guidance.

## Fetch Required Sources When Relevant

Ensure you fetch the latest version of any relevant standards or guidance if the audit depends on them and they may have changed since the last audit. Examples include:

- GitHub Actions secure use reference: <https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions>
- GitHub dependency review: <https://docs.github.com/en/code-security/supply-chain-security/understanding-your-software-supply-chain/about-dependency-review>
- OpenSSF Scorecards: <https://github.com/ossf/scorecard>

## When To Fetch

- The audit questions workflow hardening, action pinning, token permissions, or runner trust.
- The audit covers prompts, skills, agents, or MCP tooling and you need current client behavior.
- The user asks for latest best practices instead of only repository-local policy.
- The remediation depends on platform features that may have changed.

## Current High-Signal Guidance

- GitHub recommends least-privilege `GITHUB_TOKEN` permissions, careful secrets handling, and treating workflow expressions as untrusted input.
- Full-length commit SHA pinning is the strongest immutable reference for third-party actions.
- Dependabot and dependency review reduce drift and help surface risky dependency changes in workflows and package manifests.
- Scorecards can highlight risky supply-chain patterns such as unpinned actions or weak token permissions, but findings still need human review.
