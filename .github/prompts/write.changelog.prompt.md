---
name: write-changelog
description: Review Commitizen dry-run output and curate CHANGELOG.md into a release-quality update
argument-hint: Optional release notes, context, or emphasis for this changelog pass
agent: agent
---

Curate the project changelog for the next release.

Project context:

- This project is described in the repository README.
- The task applies to the root changelog file.
- This project uses `commitizen` for commit message standardization and changelog generation.

Required workflow:

1. Inspect the current Git state and recent commits relevant to the next release section.
2. Run `uv run cz bump --changelog --dry-run` to collect the next release changelog context.
3. Treat the Commitizen-rendered changelog text as input, not as the final answer.
4. Update [CHANGELOG.md](../../CHANGELOG.md) directly so the next release section reflects a best-practice release narrative.

Changelog curation rules:

- Use Git history, touched files, and the dry-run output together before editing.
- Write changelog entries in clear past tense; prefer clear, concise updates.
- Be selective: omit low-signal internal churn, obvious mechanical renames, and noise that does not matter to project readers.
- Add missing context when a commit header alone would be ambiguous or under-explain user-visible impact.
- Group items under the appropriate Keep a Changelog headings, but do not force empty sections.
- Prefer concise bullets, but expand when needed to explain contract, workflow, or security impact.
- Preserve SemVer discipline described in [docs/developer/versioning.md](../../docs/developer/versioning.md).
- Do not assume the generated changelog text is well-phrased, correctly scoped, or complete.

When deciding what belongs in the changelog, prioritize:

- Changes that affect project behavior, contributor workflows, CI or release automation, dependency and supply-chain posture, security guidance, or durable documentation contracts.
- Fixes whose impact would not be obvious from the commit subject alone.
- Additions or removals that change what maintainers or users can rely on.

Process expectations:

- Think critically about whether the raw commit subjects overstate, understate, or misclassify the real change.
- If multiple commits are facets of the same user-visible change, collapse them into one coherent bullet.
- If a commit subject is present-tense or imperative, rewrite it into release-note style.
- If there is not enough evidence to justify a changelog bullet, leave it out rather than inventing detail.
- After editing, briefly summarize what changed in the changelog and call out any uncertainties or gaps in evidence.

User-provided emphasis for this run:

{{$input}}
