---
title: VitePress docs platform and information architecture refactor
date_created: 2026-03-03
---

# Implementation Plan: VitePress docs platform and information architecture refactor

Adopt a VitePress documentation site as the canonical docs surface for `tq`, with user and developer journeys modeled after Ruff and Ty documentation ergonomics. Refactor existing docs into a single, non-duplicative information architecture with explicit canonical pages for command usage, configuration, rules, and policies. The outcome is a production-ready docs framework, migration from existing markdown, and a stable structure that scales as `tq` grows post-PyPI release.

## Architecture and design

## Goals

- Deliver a docs experience optimized for two audiences: operators/users and contributors/developers.
- Mirror Ruff/Ty ergonomics: concise overview, task-first getting started, strict reference pages, explicit exit-code behavior, stable rule documentation.
- Establish one canonical location per concept and remove duplicated guidance.
- Keep documentation as first-class product surface with deterministic build and link validation.

## Non-goals

- No custom theme fork in phase one.
- No docs i18n in phase one.
- No backward-compatible alias pages; no pre-existing docs surface exists so we can enforce a clean break and migration to the new structure.

## Design motivation

The design styling motivation for the docs site is the aesthetic in the VitePress docs themselves <https://vitepress.dev/guide/getting-started>.

VitePress GitHub repository: <https://github.com/vuejs/vitepress/>.

## Selected stack and conventions

- Framework: VitePress default theme.
- Toolchain manager: `mise` as the canonical tool/runtime manager for docs and related CLI tooling.
- Runtime pinning: project-local `mise.toml` with pinned Node version and docs tasks run via `mise run` / `mise exec`.
- Config style: typed `defineConfig` in `.vitepress/config.ts`.
- Docs root: retain repository `docs/` as VitePress source root.
- Navigation: top-level nav for primary journeys; section-specific sidebars per path prefix.
- Quality defaults: fail build on dead links, enable edit-link and last-updated metadata, deterministic sidebar ordering.
- Deployment target: GitHub Pages project site via GitHub Actions.

## Target information architecture

## User-facing

- `/` overview and value proposition.
- `/guide/installation` install and execution modes (`uv add`, `uvx`, `uv tool install`).
- `/guide/getting-started` first successful `tq check` flow.
- `/reference/cli` command surface and flags.
- `/reference/configuration` `[tool.tq]` contract and precedence.
- `/reference/exit-codes` canonical exit semantics.
- `/reference/rules/` rules index and policy overview.
- `/reference/rules/<rule-id>` one page per stable rule.
- `/reference/versioning` docs and compatibility/versioning policy for user-visible changes.

## Developer-facing

- `/developer/context` project context and design stance.
- `/developer/standards/*` enduring standards.
- `/developer/contributing` contribution workflow summary and links.
- `/developer/releasing` release workflow.
- `/developer/attestation` provenance verification.
- `/adr/` ADR index and records.

## Migration and refactor matrix

- Keep and relink:
  - `docs/developer/context.md`
  - `docs/developer/standards/code.md`
  - `docs/developer/standards/docs.md`
  - `docs/developer/standards/git.md`
  - `docs/developer/standards/policies.md`
  - `docs/developer/standards/tests.md`
  - `docs/adr/README.md`
  - `docs/adr/0001-tq-cli-config-contract.md`
- Split and migrate:
  - `docs/developer/tools/tq_check.md` into user reference pages (`cli`, `configuration`, `exit-codes`) and only retain contributor-only material under developer docs.
  - `docs/developer/tools/rules.md` into `/reference/rules/index` plus one page per rule.
- Move/rename for audience clarity:
  - `docs/developer/tools/releasing.md` to `/developer/releasing`.
  - `docs/developer/tools/attestation_verification.md` to `/developer/attestation`.
- Delete after migration:
  - `docs/developer/README.md` once replaced by VitePress nav landing pages.
  - stale plan/design docs that duplicate enduring docs.

## Rules section design (Ruff/Ty-inspired)

- Provide a dedicated rules landing page with stable rule IDs and default severities.
- Use one page per rule to keep pages focused and directly linkable.
- Generate rule pages from a source-of-truth manifest to avoid drift and duplicate edits.
- Standardize rule page sections:
  - What it does
  - Why this matters
  - Default severity
  - Trigger conditions
  - Examples
  - How to address
  - Related configuration and suppression controls
- Add “Added in” and “Behavior changes” fields to improve release-note traceability.
- Preserve kebab-case rule IDs as canonical slugs and avoid alias naming drift.
- Store canonical rule metadata in `docs/reference/rules/manifest.yaml` (or equivalent) and derive index + per-rule pages from it.

## Tasks

## Phase 1: Bootstrap VitePress platform

- Add `mise` tooling bootstrap at repository root (`mise.toml`) with pinned Node for docs tooling.
- Add Node docs toolchain at repository root (`package.json`, lockfile, docs scripts) executed through `mise`.
- Scaffold `.vitepress` config and theme defaults.
- Define nav, sidebar, edit-link, and last-updated behavior.
- Configure build outputs and cache paths; add ignores as needed.

## Phase 2: Establish canonical IA and landing pages

- Create minimal user journey pages: overview, installation, getting started.
- Create strict reference pages: CLI, configuration, exit codes, rules index.
- Create developer landing and preserve standards/ADR discoverability.

## Phase 3: Refactor and migrate existing docs

- Rewrite `tq_check` content into split canonical references.
- Introduce rules source-of-truth manifest and generate rule-index plus per-rule pages.
- Move release and attestation docs to stable developer paths.
- Remove or redirect obsolete docs to prevent duplicate sources of truth.

## Phase 4: Integrate with repository workflows

- Update root `README.md` docs links to canonical site routes.
- Add docs build check to CI workflow.
- Add GitHub Pages deployment workflow for default branch docs publishing.
- Do not add PR preview deployment workflow at present. Policy does not permit it.

### PR preview policy implications

PR preview deployment should be policy-gated because it changes repository risk and operations:

- Security posture:
  - avoid exposing write-capable deploy credentials to untrusted forked PR code,
  - avoid `pull_request_target` patterns that execute untrusted code with elevated permissions.
- Cost and quota posture:
  - extra Actions usage, artifact storage, and environment noise,
  - additional operational load despite GitHub Pages being suitable/free for OSS public repos.
- Workflow governance:
  - define who can trigger previews,
  - define retention/cleanup behavior for preview artifacts,
  - define whether previews are required or best-effort checks.
- Simplicity tradeoff:
  - previews improve review ergonomics,
  - but add CI complexity and policy surface area.

Recommended policy default for this repository:

- enable production deployment on `main` to GitHub Pages,
- defer public PR preview deployments initially,
- revisit previews once branch protection, Actions permissions, and environment rules are finalized.

## Phase 5: Harden quality and governance

- Add docs ownership/review guidance for reference sections.
- Define docs change policy for rule additions or severity changes.
- Define versioning guidance for docs contract updates.

## Verification

- VitePress dev server runs and production build succeeds.
- Dead link checks pass in CI.
- Every current stable rule has exactly one canonical page under `/reference/rules/`.
- Rule manifest and generated rule pages are synchronized by CI (no manual divergence).
- CLI/config/exit-code contract appears exactly once in canonical reference pages.
- Root `README.md` links users to the new docs surface.
- Developer standards and ADR pages remain reachable and non-duplicated.

## Risks and mitigations

- Risk: docs duplication during transition.
  - Mitigation: enforce migration matrix and delete superseded pages in the same PR.
- Risk: path churn breaks inbound links.
  - Mitigation: enforce a clean break, use targeted redirects, and do not keep legacy paths.
- Risk: docs framework drift from Python-centric tooling.
  - Mitigation: isolate Node tooling to docs scripts only, pin with `mise`, and keep Python quality gates unchanged.
- Risk: rule manifest generation adds pipeline complexity.
  - Mitigation: keep generator deterministic, validate generated output in CI, and document regeneration commands.

## Decisions confirmed

- Manage Node and docs tooling via `mise`.
- Deploy docs via GitHub Pages for OSS project documentation.
- Use a source-of-truth rules manifest and generate rule reference pages.
- Do not add PR preview deployments.
