# Report Template

Use this structure for findings-only reviews and for reviews that include direct fixes.

## Scope

- What repository, change set, or directories were reviewed
- Which surfaces were in scope
- Any assumptions or constraints

## Findings

List findings in priority order. For each finding include:

- surface and location
- why it is costly: hidden coupling, ownership confusion, fragile behavior, or unnecessary abstraction
- concrete evidence
- recommended root-cause fix

## Fixes Applied

- list each issue fixed immediately
- state the structural reduction achieved: deletion, split responsibility, boundary cleanup, or tighter workflow
- note any related tests, docs, or automation updated with the fix

## Follow-Up Plan

- rank remaining work in execution order
- keep each item action-oriented and small enough to hand off

## Validation

- what was validated
- what remains unvalidated
- residual risks or open questions
