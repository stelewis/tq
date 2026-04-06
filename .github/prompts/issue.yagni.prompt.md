---
name: issue-yagni
description: Audit a proposed issue for necessity, scope, and overengineering risk
argument-hint: Issue link or problem statement to audit
agent: agent
---

Perform a YAGNI audit on the issue provided by the user.

1. Analyze the problem and motivation.
2. Check whether the current architecture already covers it.
3. Assess complexity versus value.
4. Suggest a simpler path when one exists.
5. Return clear findings.
If the issue is valid, you are welcome to be supportive. But if you find it overengineered or unnecessary, help the user onto the path of progress rather than perfection.

## Principles (Non-exhaustive)

- YAGNI (You Aren't Gonna Need It)
- Minimum Viable Design
- Avoid Premature Optimization
- Done Is Better than Perfect
- Occam's Razor

Keep the user focused on shipping the simplest design that solves the real problem.
