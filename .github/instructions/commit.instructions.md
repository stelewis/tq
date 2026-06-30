---
applyTo: "commit-message"
---

- Commit your work as you progress in clean, atomic commits.
- Write commit messages in Conventional Commit format: `<type>(<optional-scope>): <subject>`.
- Only include a scope if it adds meaningful context; omit if the type alone is sufficient.
- Keep the subject concise and specific.
- Keep the body concise and focused on meaningful context.
- Use imperative mood.
- Do not hard-wrap any line in the commit message.
- `prek` is used as the pre-commit runner; use `uv run prek run ...` when you need to run hooks manually.
- If validation is already green, you may skip the hook check with `git commit --no-verify`.
- Do not add `Co-authored-by` trailers or sign-off lines unless explicitly requested.
- `Closes`, `Fixes`, and `Resolves` are closing keywords; use them when a commit fully resolves an issue.
- Plain references like `#123` are autolinks and do not close issues; use them where they add meaningful context.
- Allowed type set: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `build`, `ci`, `perf`, `style`, `revert`.
