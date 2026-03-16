---
name: skill-creator
description: Create new skills, refactor existing skills, and improve triggering, reliability, and maintainability through reusable resources and lightweight evaluation loops. Use when a user wants to design a skill from scratch, rewrite or harden an existing skill, set up realistic test prompts or eval fixtures, compare skill-assisted and baseline behavior, or tighten skill descriptions so the right skill triggers more often.
argument-hint: describe the skill creation or improvement task
user-invocable: true
disable-model-invocation: true
---

# Skill Creator

Use this skill to build skills deliberately instead of treating them like static prompt files. The working loop is: understand the job, design reusable resources, write or refactor the skill, test it with realistic prompts, inspect failures, improve the skill, then package it.

Be flexible about where to enter the loop. Some users need help inventing a new skill from scratch. Others already have a draft and need evaluation, refactoring, or better trigger metadata.

## What Skills Provide

1. Specialized workflows - Multi-step procedures for specific domains
2. Tool integrations - Instructions for working with specific file formats or APIs
3. Domain expertise - Company-specific knowledge, schemas, business logic
4. Bundled resources - Scripts, references, and assets for complex and repetitive tasks

## Core Principles

### Treat Context As A Budget

The context window is a public good. Skills share the context window with everything else the agent needs: system prompt, conversation history, other Skills' metadata, and the actual user request.

Assume the target agent is already capable. Include the information it needs to do the task well, not a tutorial on ideas it likely already knows.

Prefer compact examples, clear decision rules, and reusable files over long narrative explanation.

### Match The Degree Of Freedom To The Task

- Use high-level instructions when multiple approaches can work.
- Use references or pseudocode when a preferred pattern exists but adaptation still matters.
- Use prescriptive scripts or templates when the task is fragile, repetitive, or easy to get wrong.

### Keep The Skill Lean

Put core workflow guidance in SKILL.md. Put large examples, schemas, and deep reference material into files under references/. Put deterministic helpers into scripts/. Put output assets into assets/.

### Stay Platform-Neutral By Default

Do not hard-code assumptions about a specific model vendor or agent runtime unless the user explicitly needs that platform. Describe workflows in terms of capabilities the current environment may or may not provide, then adapt to the tools actually available.

When standards or platform behavior may have changed, prefer live canonical docs over stale memory. See `references/live-standards.md` for the current source list and when to fetch it.

### Follow The Principle Of Lack Of Surprise

Skills must not contain misleading behavior, malware, exploit code, or hidden side effects. A skill should do what its description leads a user to expect.

## Anatomy Of A Skill

```text
skill-name/
├── SKILL.md
├── scripts/          # Deterministic or repetitive helpers
├── references/       # Guidance loaded only when needed
├── assets/           # Files consumed in generated output
├── agents/           # Optional agent templates
└── evals/            # Optional local test prompts and fixtures
```

Only SKILL.md is required. Everything else is optional and should exist only when it helps.

### Frontmatter

Every skill must define:

- name
- description

The description is the primary trigger. It should say both what the skill does and when it should be used. Put trigger language in the description, not in a later "when to use" section that loads too late to help.

Optional fields can include fields such as `license`, `compatibility`, `metadata`, and `allowed-tools`, plus platform-specific fields (for example `argument-hint`, `user-invokable`, `disable-model-invocation`) where supported by your agent platform.

### Progressive Disclosure

Skills load in layers:

1. Metadata is always visible.
2. SKILL.md is loaded when the skill triggers.
3. References, scripts, and assets are loaded only as needed.

Keep SKILL.md focused enough that another agent can absorb it quickly and know where to go next.

## Workflow

### 1. Capture Intent

Start by extracting what you can from the current conversation before asking more questions.

Look for:

- The task the skill should enable.
- The prompts or contexts that should trigger it.
- Expected outputs, artifacts, or quality bars.
- Files, APIs, schemas, or external systems the skill must work with.
- Corrections or preferences the user has already expressed.

If details are still missing, ask a small number of concrete follow-up questions.

### 2. Plan Reusable Contents

Turn concrete examples into reusable skill contents.

Ask, for each example:

1. What would an agent repeatedly have to rediscover?
2. What part is deterministic enough to encode as a script or template?
3. What documentation belongs in references instead of SKILL.md?

Good signals for bundled resources:

- The same helper code would be rewritten every time.
- The task depends on stable schemas, policies, or APIs.
- The output needs a specific template or asset bundle.

If the skill depends on evolving standards such as Agent Skills metadata, discovery, activation, or VS Code behavior, plan a small reference file with canonical documentation URLs so future revisions can refresh against the live docs instead of drifting.

### 3. Check Live Standards When Needed

Before writing or changing a skill, decide whether the task needs current external guidance.

Fetch the canonical docs from `references/live-standards.md` when any of these are true:

- The skill uses frontmatter or packaging conventions that may evolve.
- The user asks about latest or best-practice guidance.
- The skill depends on client behavior such as discovery paths, slash-command behavior, or model-driven activation.
- You are comparing portable Agent Skills guidance with a client-specific implementation such as VS Code.

If a web-fetch tool is available, fetch the relevant URLs before finalizing the draft. If no such tool is available, use the bundled reference file as the fallback summary and note that live validation was not possible.

### 4. Initialize Or Inspect The Skill

For a new skill, start with the scaffold:

```bash
scripts/init_skill.py <skill-name> --path <output-directory>
```

For an existing skill, inspect the current SKILL.md and bundled resources before changing anything. Remove obsolete files when they encode outdated workflow or platform assumptions.

### 5. Write Or Refactor SKILL.md

When writing the body of the skill:

- Use imperative guidance.
- Explain why a step matters when that helps the target agent generalize.
- Prefer decision rules and patterns over brittle instructions that only fit one example.
- Keep framework-, domain-, or variant-specific detail in references files and link to them clearly.

Helpful references in this skill:

- references/workflows.md for sequential and conditional workflow patterns.
- references/output-patterns.md for templates and example-driven output guidance.
- references/schemas.md for portable eval and workspace layout examples.
- references/live-standards.md for canonical Agent Skills and VS Code documentation URLs.

### 6. Add Resources

Use the minimum resource surface that materially improves reliability.

#### scripts/

Add scripts for repetitive or fragile work. Test representative scripts by actually running them.

#### references/

Add references for large examples, schemas, domain knowledge, policies, or detailed procedures. Reference each file directly from SKILL.md so another agent knows when to read it.

For guidance that can drift over time, prefer a small reference file containing canonical URLs plus instructions for when to fetch them. This keeps SKILL.md short while still nudging future agents toward the live standards.

#### assets/

Add assets only when the skill needs concrete files in its output, such as templates, images, fonts, or boilerplate code.

#### evals/

Only add evals/ when the skill is explicitly being being evaluated with prompts or fixtures. This is for local development and iteration, not for the distributed skill package. The package script excludes a top-level evals/ directory on purpose.

### 7. Test The Skill

If specifically asked to improve or evaluate a skill, test with realistic prompts that a real user would plausibly write.

For objective tasks, define a small eval set. For subjective tasks, lightweight qualitative review is often enough.

Recommended loop:

1. Draft 2-5 representative prompts.
2. Save portable eval data in evals/evals.json when structured tracking is useful.
3. Run the prompts with the skill attached if the current platform supports that.
4. Compare against a useful baseline when possible.
5. Save artifacts and notes in a sibling workspace so iterations stay inspectable.

If the platform cannot run skill-attached and baseline executions separately, do a lighter manual evaluation: run the task, inspect the transcript or outputs, revise the skill, and rerun the same prompts.

Only introduce assertions when they can be checked objectively. Do not force faux precision onto subjective tasks.

### 8. Improve The Skill

When a test fails or the user gives feedback:

1. Generalize from the failure instead of overfitting to a single prompt.
2. Remove instructions that add token cost without improving outcomes.
3. Explain the why behind important behavior so the target agent can reason instead of pattern-matching blindly.
4. Look for repeated helper code or repeated reasoning steps across test cases and bundle them once.

Prefer a second clean draft over endlessly accreting one-off patches.

### 9. Validate And Package

Validate the skill structure:

```bash
scripts/quick_validate.py <path/to/skill>
```

Create a distributable archive:

```bash
scripts/package_skill.py <path/to/skill>
```

You can optionally pass an output directory:

```bash
scripts/package_skill.py <path/to/skill> ./dist
```

Packaging should happen after the skill contents are clean. The resulting .skill archive should contain only the reusable skill itself, not local evaluation artifacts.

## Skill Writing Patterns

### Strong Description Pattern

Descriptions should name both capability and triggers.

Weak:

```text
Help with dashboards.
```

Strong:

```text
Build and refine internal dashboards and lightweight data views. Use when a user mentions dashboards, metrics, KPI displays, charts, internal reporting, or wants to visualize operational data even if they do not explicitly ask for a dashboard.
```

### Output Pattern

When format matters, show a template or a concrete example instead of describing it abstractly. See references/output-patterns.md.

### Workflow Pattern

When the task has branching or staging, show the decision points and the sequence explicitly. See references/workflows.md.

### Live Standards Pattern

When the skill must stay aligned with external standards, keep the URLs in a bundled reference file and instruct the agent to fetch them when tools permit.

## What Not To Include

Do not add extra documentation that exists only to narrate the creation process.

Avoid files such as:

- README.md
- CHANGELOG.md
- installation notes
- ad hoc scratch files that are not part of the reusable skill

If a file does not help another agent execute the skill, delete it.

## Iteration Mindset

Skill creation is not complete when the first draft reads well. It is complete when realistic usage shows that the description triggers at the right times, the instructions guide the target agent well, and the bundled resources remove repeated failure modes without adding clutter.
