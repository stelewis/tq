# Contributing to the Test Quality Toolkit (tq)

Thanks for taking the time to help improve this project! Whether you're fixing a bug, adding a feature, improving documentation, or suggesting an idea, all contributions are welcome and appreciated.

Because `tq` is a developer tool used by multiple teams, we strive for consistent workflows, high code quality, and clear documentation. The following guide will help you get up to speed and make a successful contribution.

## Getting started

1. **Fork and clone** this repository to your GitHub account.
2. Create a descriptive feature branch, e.g.: `feature/add-check`, `fix/cli-parser`, `docs/contributing`.
3. Install the development environment and tools:

   ```sh
   uv sync                 # install project dependencies
   uv run prek install     # install pre‑commit hooks
   ```

4. Make your changes and run the full quality gate locally:

   ```sh
   uv run ruff format && uv run ruff check --fix && uv run ty check && uv run tq check && uv run pytest -q
   ```

   The same sequence of commands is executed by CI on every pull request.

## Development documentation

Longer explanations of the repository conventions are kept in `docs/developer`:

- **Code standards** – formatting, typing, packaging, and import rules.
- **Git workflow** – branch naming, commit message conventions, PR guidelines.
- **Testing standards** – how to write tests, run them, and use the test-quality tool itself.

Read or search the documents before starting larger changes; links can be found in the [Developer docs index](docs/developer/index.md).

## Issues and pull requests

- **Search first.** Before opening a new issue, look through existing issues to avoid duplicates.
- **Issue types.** Create issues for bugs, enhancements, or questions.
- **Pull Requests.** Keep PRs small and focused. Target the `main` branch. Every PR must pass all checks and receive at least one approving review before merging.
- **Commit messages.** Use [Conventional Commits](https://www.conventionalcommits.org/). A `commit-msg` hook enforces this; run `uv run cz commit` to help format messages if needed.

## Code style and tooling

- Formatting and linting are performed by [ruff](https://docs.astral.sh/ruff/).
- Type checking uses [ty](https://docs.astral.sh/ty/).
- Pre-commit hooks are managed by [prek](https://prek.j178.dev/).
- Tests run with `pytest`; quality of tests is enforced by `uv run tq check`.

You can run individual tasks directly (e.g. `uv run ruff format`).

## Documentation

Documentation in this repository is treated as first‑class. See `docs/developer/standards/docs.md` for guidelines.

- Keep docs **useful, stable, concise, and small**.
- Prefer one canonical reference per concept and avoid redundancy.
- Document contracts and workflows, not implementation details.

If you update code that has public behavior, make sure to update corresponding documentation and tests.

## Support and communication

If you need help or have questions, open an issue tagged `question`.  The maintainers monitor the repository and will respond as soon as possible.

Thanks again for contributing to `tq`! We appreciate your effort in making the toolkit better for everyone.
