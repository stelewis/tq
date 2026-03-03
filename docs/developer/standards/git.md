# Git Workflow Standards

This document defines the Git workflow for this repository.

## Goals

- **Stable trunk**: `main` is always green and shippable.
- **Linear history**: readable blame/log; avoid merge-commit noise.
- **Fast integration**: resolve conflicts early and reduce long-lived divergence.
- **Small refactor surface area**: integrate in small slices; keep changes easy to review.

## Branch Model

- **Trunk-based development**: `main` is the trunk.
- **No direct pushes to trunk**: all changes land via pull requests.
- **Short-lived branches**: create branches per change; delete after merge.

## Branch Naming

Use a short, descriptive name with a clear prefix:

- `feature/<topic>`
- `fix/<topic>`
- `refactor/<topic>`
- `chore/<topic>`
- `docs/<topic>`

## Keeping Branches Current

- Rebase your branch onto `main` early and often (daily if active).
- Prefer resolving conflicts on your branch (before review) rather than on `main`.
- Avoid large “integration” PRs created by letting a branch drift.

## Pull Requests

- Every change uses a PR, even for small fixes.
- Keep PRs small and focused (one intent; one reason to change).
- Trunk must be protected by required checks and reviews.

## Merge Strategy

Prefer one of these approaches:

- **Squash merge onto `main`** (recommended): produces one meaningful commit per PR.
- **Rebase + fast-forward onto `main`**: preserves individual commits, but requires strict commit hygiene.

Do not merge PRs with merge commits into `main`.

## Commit Hygiene

- Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for commit messages.
- Commit messages are validated with [Commitizen](https://commitizen-tools.github.io/commitizen/) in both local `commit-msg` hooks and CI commit-range checks.
- Before merge, rewrite local history via interactive rebase as needed (fixup/squash) so commits are intentional.
- Commits on `main` must be meaningful; e.g., avoid “wip”, “oops”, or “try again”.

## Feature Flags

- Merge incomplete work behind feature flags when it helps keep branches short.
- Never break trunk to “finish later”.

## Repository Settings

- Protect `main`:
  - require PRs
  - require status checks
  - require at least one review
  - require linear history
  - disallow force-pushes
- Enable either squash-merge only, or rebase-merge only.
- Tag releases with annotated tags on `main`.
