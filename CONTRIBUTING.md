# Contributing to tq

Thanks for taking the time to help improve this project! Whether you're fixing a bug, improving documentation, adding a feature, or suggesting an idea, all contributions are welcome and appreciated.

The product code is a Rust workspace. Python tooling exists for packaging, repository automation, and docs-site support. Use this guide as a short entrypoint, then follow the canonical developer docs under [docs/developer/index.md](docs/developer/index.md).

Because `tq` is a developer tool used by multiple teams, we strive for consistent workflows, high code quality, and clear documentation.

## Getting Started

1. Fork and clone the repository.
2. Create a short descriptive branch such as `feature/add-cli-command`, `fix/parser-bug`, or `docs/update-guide`.
3. Install the local toolchain:

   ```sh
   uv sync                   # install project dependencies
   uv run prek install       # install pre‑commit hooks
   mise trust                # trust project mise.toml tasks/tools in this repo
   mise install              # install tool versions declared in mise.toml
   mise exec -- npm install  # install node dependencies for docs
   ```

4. Use the Rust workspace loop while developing:

   ```sh
   cargo check --workspace --all-targets --locked
   cargo run -p tq-cli --locked -- check --help
   ```

5. Before opening a PR, run the relevant validation commands for your change. The common local checks are:

   ```sh
   cargo fmt --all --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --locked
   ```

6. If you plan to rebase onto main instead of squash merging, clean up your commit history before opening a PR. See the [Git workflow docs](docs/developer/standards/git.md) for the merge policy.

7. Open a pull request targeting `main` and request a review.

For docs, packaging, or release-surface changes, use the commands in [docs/developer/tools/index.md](docs/developer/tools/index.md), [docs/developer/tools/local-workflows.md](docs/developer/tools/local-workflows.md), and [docs/developer/tools/ci.md](docs/developer/tools/ci.md).

## Development documentation

Repository conventions are kept in `docs/developer`:

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

## Documentation

Documentation is part of the product surface.

- Update docs and tests with any user-visible behavior change.
- Prefer one durable reference doc per topic.
- Keep docs focused on contracts and workflows, not implementation trivia.

## Support

If you need help, open an issue with the Question template. For suspected vulnerabilities, follow [SECURITY.md](SECURITY.md) instead of opening a public issue.

Thanks again for contributing to `tq`! We appreciate your effort in making the project better for everyone.
