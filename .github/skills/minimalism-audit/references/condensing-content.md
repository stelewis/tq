# Condensing Content

Use this guide when the audit finds a surface that should stay but is carrying too much text.

## Goal

Optimize for fast, correct comprehension. Good minimalism helps both humans and models.

- Humans benefit from lower cognitive load, clearer ownership, and less decision fatigue.
- Models benefit from explicit structure, fewer contradictions, and less irrelevant text competing for context.

Do not invent a separate writing style for models. Prefer clear engineering prose that is easy to scan, structurally explicit, and free of avoidable repetition.

## What To Keep

Keep content that the reader cannot recover cheaply from code, filenames, types, or the navigation tree:

- contracts and invariants
- architectural boundaries and ownership
- security or supply-chain constraints
- exact output schemas or workflow steps when ambiguity would cause failure
- non-obvious edge cases, units, side effects, or failure modes

## What To Delete Or Shrink

Delete, shorten, or move content that mostly repeats what is already obvious:

- definitions of common concepts the model and reader already know
- prose that restates names, signatures, schema fields, or visible control flow
- multiple files explaining the same policy or workflow
- broad rationale repeated in every prompt, skill, or standard
- decorative links and examples that add little signal

## Keep It Concise

Prefer the smallest surface that still preserves correctness:

- keep prose small, direct, targeted, and bounded to one responsibility
- avoid verbosity in docs or docstrings
- keep model-specific prompts and instructions especially tight because they consume shared context on every request
- do not add explanatory prose when a clear name, type, schema, or section heading already carries the meaning
- prefer one explicit rule over several overlapping sentences that say nearly the same thing

Concise does not mean cryptic. Remove redundancy and filler, not the contract, constraint, or edge case.

## Condensing Patterns

## 1. Replace explanation with structure

Prefer:

- precise names
- narrow sections with clear headings
- stable schemas and typed interfaces
- short lists over long paragraphs when the content is procedural

If the code, schema, or heading already makes the meaning obvious, cut the sentence.

## 2. Keep one owner per idea

When a concept appears in multiple places:

- keep the source where the responsibility naturally lives
- delete duplicates when possible
- leave only a short pointer when crossing a non-obvious boundary

## 3. State constraints, not atmosphere

Prefer concrete constraints over broad prose:

- what must happen
- what must not happen
- what format is required
- what is out of scope

This keeps prompts and instructions precise without adding narrative padding.

## 4. Use examples only when they remove ambiguity

Good examples:

- show exact output format
- demonstrate a subtle edge case
- clarify a non-obvious workflow branch

Bad examples:

- repeat the rule without adding information
- show a trivial case the reader can infer
- drift away from the rule they are meant to teach

Examples must comply with the instructions. Inconsistent examples train the model to ignore the rule.

## 5. Separate entrypoints from detail

For skills, prompts, and docs:

- keep the entrypoint short
- move bulky checklists, examples, and specialty detail into referenced files
- avoid deep chains of references

## 6. Specify format only when format matters

If a task needs structured output, define it clearly:

- required sections or fields
- ordering if it matters
- exact failure output if special cases exist

If format does not matter, do not over-specify it.

## Review Questions

Before keeping a paragraph, ask:

1. Is this non-obvious and durable?
2. Can the reader infer this from code, types, filenames, or structure?
3. Does this paragraph define a contract, constraint, or edge case?
4. Would deleting it make the task materially harder or less correct?
5. Is this the canonical place for this information?

If the answer is mostly no, cut or move it.

## Rewrite Tactics

- collapse repeated rationale into one sentence
- replace paragraphs of guidance with a checklist
- merge near-duplicate prompts or docs into a parameterized workflow
- replace commentary about style with one explicit rule and one compliant example
- remove throat-clearing and setup language before the first real instruction

## Completion Bar

The edited content should:

- preserve the durable contract
- reduce tokens, drift risk, or maintenance surface
- remain easy for a human to scan quickly
- avoid adding a second layer of explanation about the explanation
