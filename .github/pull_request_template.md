# Summary

Describe what this PR changes and why.

## Release Intent

Apply exactly one PR label before merge: `release:none`, `release:patch`, or `release:minor`.
Labels are the canonical release-intent signal; this template is only a reminder.

## Key Changes

- (e.g.)

## Validation

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] `cargo test --workspace --locked`
- [ ] `cargo run -p tq-docsgen --locked -- generate all`
- [ ] `cargo run -p tq-release --locked -- verify-release-policy --repo-root .`
- [ ] `cargo package --workspace --locked`
- [ ] `mise run release-build`

## Notes

Anything reviewers should pay extra attention to (risk, rollout, follow-ups).
