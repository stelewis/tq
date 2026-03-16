# Portable Eval Schemas

Use these examples when a skill benefits from structured test prompts or a repeatable iteration workspace.

These schemas are intentionally lightweight and platform-neutral. Adapt them if the user's environment already has a stronger convention.

## evals/evals.json

Use evals/evals.json to store realistic prompts for objective or repeatable testing.

```json
{
  "skill_name": "example-skill",
  "evals": [
    {
      "id": "summarize-quarterly-report",
      "prompt": "Summarize this quarterly report and call out major risks.",
      "expected_output": "Produces a concise summary with risk callouts and no invented figures.",
      "files": ["fixtures/q1-report.md"],
      "assertions": [
        "Mentions the three largest risk areas",
        "Does not invent financial values"
      ]
    }
  ]
}
```

Field guidance:

- skill_name: Matches the skill being tested.
- evals: The list of test cases.
- id: Stable identifier for the test case. Prefer descriptive kebab-case.
- prompt: The user request to run.
- expected_output: Plain-language description of success.
- files: Optional input files needed for the test case.
- assertions: Optional checks when the outcome is objectively verifiable.

## Iteration Workspace Layout

Store evaluation outputs outside the skill directory so each iteration stays inspectable without polluting the packaged skill.

```text
example-skill/
├── SKILL.md
└── evals/
    └── evals.json

example-skill-workspace/
└── iteration-1/
    ├── summarize-quarterly-report/
    │   ├── with_skill/
    │   ├── baseline/
    │   └── notes.md
    └── summary.md
```

Use names that describe what each eval is testing rather than generic numbered folders when practical.

## Notes Template

Keep a short notes file for each iteration so future revisions are guided by real evidence instead of memory.

```markdown
# Iteration Notes

## What improved

- The skill chose the right output structure without extra prompting.

## What failed

- The agent skipped a required risk section.

## Next change

- Add a stronger output example for risk-focused summaries.
```

## When To Skip Structure

If the task is highly subjective, lightweight notes and manual review are often enough. Do not add JSON fixtures or assertions unless they make the iteration loop clearer and faster.
