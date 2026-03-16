# Contributing to Test Quality Toolkit (tq)

Thanks for taking the time to help improve this project! Whether you're fixing a bug, improving documentation, adding a feature, or suggesting an idea, all contributions are welcome.

Because `tq` is a developer tool used by multiple teams, we strive for consistent workflows, high code quality, and clear documentation. The following guide will help you get up to speed and make a successful contribution.

## Getting started

1. **Fork and clone** this repository to your GitHub account.
2. Create a short, descriptive branch such as `feature/add-cli-command`, `fix/parser-bug`, or `docs/update-guide`.
3. Install the development environment and tools:

   ```sh
   uv sync                   # install project dependencies
   uv run prek install       # install pre‑commit hooks
   mise trust                # trust project mise.toml tasks/tools in this repo
   mise install              # install tool versions declared in mise.toml
   mise exec -- npm install  # install node dependencies for docs
   ```

   Optional: activate `mise` in your shell if you want tools available without `mise exec`.

4. Make your changes and run the full quality gate locally:

   ```sh
   uv run ruff format && uv run ruff check --fix && uv run ty check && uv run tq check && uv run pytest -q
   ```

   The same sequence of commands is executed by CI on every pull request.

   For docs changes, also run:

   ```sh
   mise run docs-build
   ```

## Development documentation

Longer explanations of the repository conventions are kept in `docs/developer`:

- **Code standards** - formatting, typing, packaging, and import rules.
- **Git workflow** - branch naming, commit message conventions, and PR guidelines.
- **Testing standards** - how to write tests, run them, and keep them modular.
- **Developer tools** - local commands, pre-commit hooks, CI checks, and automation.

Read or search the documents before starting larger changes; links in the [Developer docs index](docs/developer/index.md).

## Issues and pull requests

- **Search first.** Before opening a new issue, look through existing issues to avoid duplicates.
- **Issue types.** Use the Bug report, Feature request, and Question forms where they fit; use a blank issue if the topic does not fit cleanly into a form.
- **Pull requests.** Keep PRs small and focused. Target `main`. Every PR must pass all checks and receive at least one approving review before merging.
- **Commit messages.** Use [Conventional Commits](https://www.conventionalcommits.org/). A `commit-msg` hook enforces this; run `uv run cz commit` if you want help formatting messages.

## Code style and tooling

- Formatting and linting are handled by [ruff](https://docs.astral.sh/ruff/).
- Type checking uses [ty](https://docs.astral.sh/ty/).
- Pre-commit hooks are managed by [prek](https://prek.j178.dev/).
- Tests run with `pytest`; test quality is checked with `uv run tq check`.

## Documentation

Documentation in this repository is treated as first-class. See `docs/developer/standards/docs.md` for the durable rules.

- Keep docs useful, stable, concise, and small.
- Prefer one reference doc per concept and avoid duplication.
- Document contracts and workflows rather than implementation trivia.

If you update code with user-facing behavior, update the corresponding documentation and tests.

## Support and communication

If you need help or have questions, open an issue with the Question template. For suspected vulnerabilities, follow `SECURITY.md` rather than creating a public issue.

Thanks again for contributing to `Test Quality Toolkit`! We appreciate your effort in making the project better for everyone.
