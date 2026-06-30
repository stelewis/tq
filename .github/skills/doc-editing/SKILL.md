---
name: doc-editing
description: 'Write and tighten documentation as clean, published prose rather than incremental notes, breadcrumbs, or implementation chatter. Use when reviewing docs for reader-first structure, durable ownership, low duplication, minimal cross-linking, and removal of implementation language, change-history framing, and internal implementation detail that does not belong in the document.'
argument-hint: describe the doc or doc set to tighten, the intended audience if it is not obvious, and whether you want an audit or direct edits
user-invocable: true
disable-model-invocation: false
---

# Doc Editing

Use this skill to turn documentation into durable, reader-first prose that reads like a published reference, guide, or article rather than a trail of incremental edits.

Enduring docs are reference pages, guides, standards, and contracts intended to remain valid across releases. ADRs, plan docs, design docs, and changelogs are not enduring docs.

## Problem

Documentation quality drops when pages are written from the author's change history instead of the reader's needs. Common failure modes include:

- notes to self, TODOs, and transitional commentary left in enduring docs
- milestone, sprint, or "current state" language that makes the page age immediately
- implementation details and internal shapes leaking into docs that should describe contracts or workflows
- the same rule repeated across many pages instead of being owned once and referenced only where actually needed
- breadcrumb prose that explains what changed rather than what the reader needs to know
- verbose restatements of code structure or similar details repeated across pages without adding reader value

The result is documentation that feels like a stream of consciousness instead of a coherent published surface.

## When To Use It

- The user wants docs tightened, polished, rewritten, or reviewed for quality.
- A page reads like incremental development notes instead of a stable guide.
- A documentation change added too many reminders, caveats, links, or restatements and needs consolidation.
- A doc mixes contracts, implementation details, and planning language.
- Enduring docs need to be brought up to the standard in `docs/developer/standards/docs.md`.

## Workflow

1. Define the surface. Decide whether the task covers one page, a cluster of related pages, or a whole docs section.
2. Read the local standard first. In this repo, start with `docs/developer/standards/docs.md` and any adjacent page that already owns the concept.
3. Classify each touched page as enduring, ADR, plan, design, or changelog before editing or auditing it.
4. Identify the owning responsibility of each page. Each doc should answer one class of reader question well.
5. Evaluate the text from the reader's perspective. Ask: if this page were opened in isolation, would it read like a coherent published article or like commentary on recent edits?
6. Remove editorial noise and non-owning detail: TODOs, draft markers, change-history framing, milestone language in enduring docs, internal implementation detail that the page does not own, and repeated explanations already owned elsewhere.
7. Consolidate ownership. Put the full rule in the one page that owns it. In adjacent pages, keep only the page-specific consequence and link only when it saves real search effort.
8. Rewrite for durable prose. Lead with the rule, contract, or workflow, and prefer direct statements over narrative about how the doc evolved.
9. Run the narrowest repo-owned docs validation you can identify, such as an existing Markdown lint, docs build, or link check. If the repository exposes no docs-specific validation, state that validation was skipped.

## Branching Rules

- ADRs keep decision context and tradeoffs, but still lose editorial clutter and temporary implementation chatter.
- Plans and design docs may keep temporary sequencing language when it serves the plan; enduring docs may not.
- Keep a detail only when it is necessary to the page's contract, boundary, or operator workflow. Remove recent-change evidence, defensive repetition, and low-value implementation trivia.
- If two pages explain the same rule, choose the true owner and reduce the other to the page-specific consequence.
- If a statement exists only to protect against a likely future misuse, prefer one strong statement in the owning doc over defensive repetition everywhere.

## Editing Heuristics

- Write for the next reader, not for the last editor.
- Prefer one clear statement over several weaker reminders.
- Use links sparingly; a link must earn its place.
- Treat enduring docs as finished surfaces, not scratchpads.
- Avoid leaking implementation detail or document-process commentary unless the page explicitly owns it.
- Avoid details that read like notes to self rather than durable information for readers.

## Required Output

- The owning question each touched page should answer.
- The main sources of editorial noise, duplication, or ownership confusion found.
- In audit-only mode, the recommended edits for each touched page and any rule that should move to an owning page or be reduced to a light reference.
- In edit mode, the edits made and any rule moved to an owning page or reduced to a light reference.
- Validation status, or an explicit note that no docs-specific validation exists.

## Completion Bar

- Audit-only output does not imply edits were made and gives page-specific recommendations.
- Edited docs read coherently without knowledge of the recent change history.
- Enduring docs contain no TODOs, notes to self, draft markers, milestone framing, or implementation breadcrumbs.
- Each page has a clear responsibility, sparse purposeful links, and no repeated rule ownership.
