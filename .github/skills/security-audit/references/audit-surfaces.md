# Audit Surfaces

Use this checklist to avoid auditing only the application code while missing the automation and trust surfaces around it.

## Source Code And Runtime Boundaries

- Trace untrusted input from CLI, config, environment, files, archives, APIs, and generated text.
- Check boundary validation, fail-closed behavior, type strictness, and unsafe defaults.
- Review subprocess calls for string-built shells, unvalidated arguments, and hidden side effects.
- Review filesystem access for traversal, symlink escape, unsafe extraction, and writes outside expected roots.
- Review secret handling in code, logs, errors, fixtures, snapshots, and tests.

## Configuration And Manifests

- Review pyproject.toml, lockfiles, config files, CI settings, and setup scaffolding for insecure defaults.
- Check that security tooling is enabled early enough to matter and that critical checks are not optional without justification.
- Check whether secure defaults are enforced in repository setup or merely documented.

## Dependencies And Lockfiles

- Inventory direct dependencies, development dependencies, pre-commit hooks, Actions, build backends, release tools, and any fetched binaries.
- For each new or risky dependency, review necessity, maintainer trust, release hygiene, install scripts, binary delivery, and transitive size.
- Check update coverage through Dependabot or equivalent automation.
- Flag marginal dependencies that should be replaced with small local code.

## GitHub Actions And Workflows

- Review workflow-level and job-level permissions.
- Review action references for trust level, pinning, and local-vs-third-party boundaries.
- Check inline shell steps for expression injection and unsafe interpolation of event payloads.
- Review secrets use, environment promotion, artifact handling, and write-capable automation.
- Review runner model. Prefer GitHub-hosted runners unless there is a strong, controlled reason otherwise.

## Hooks, Local Automation, And Build Tooling

- Review pre-commit hooks, local scripts, task runners, and setup helpers as supply-chain inputs.
- Check whether hooks execute remote code, install extra dependencies, or mutate lockfiles unexpectedly.
- Review build tools and publish steps for hidden network access, credential use, or overly broad permissions.

## Release And Distribution Surface

- Review publishing workflows, tags, release automation, signing, provenance, and artifact generation.
- Check whether release credentials are short-lived and scoped, and whether release steps are auditable.
- Check for protections around who can publish, approve, or trigger release flows.

## Agent And Automation Surface

- Review prompts, instructions, skills, custom agents, MCP servers, and tool auto-approval settings.
- Treat prompt files and repository-authored text as potentially vulnerable to prompt injection or unsafe tool use.
- Check whether agent workflows can read outside the repository, expose secrets, or execute fetched text as commands.
- Review any bundled scripts or external integrations for hidden side effects or excessive trust.

## Reporting Rule

When you finish a surface, either record a finding or state why it passed. Do not silently skip categories.
