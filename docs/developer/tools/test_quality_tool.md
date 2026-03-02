# Test Quality Tool

The project includes an automated test quality checker that validates:

1. **Mapping**: Each source module has at least one test file
2. **Structure**: Test files are in the correct directory structure
3. **Size**: Test files don't exceed the configured maximum lines (default: 600)
4. **Orphans**: Test files correspond to existing source modules

*Note: Cross-module tests, duplicated coverage, misnamed-by-semantics, redundant-by-semantics, and vacuous tests are not presently checked due to noisy heuristics.*

Run the checker:

```bash
uv run check_test_quality
```

The tool exits with a non-zero code if errors are found.

## Configuration

Configure the test quality checker in `pyproject.toml`:

```toml
[tool.test_quality]
# Patterns to ignore when scanning source and test files
ignore = ["**/deprecated/**", "experimental/**"]

# Maximum allowed non-blank lines in a test file
max_test_lines = 600

# Whether to ignore __init__.py files in mapping checks
ignore_init = true

# Optional: restrict which suffixes are treated as valid qualifiers.
#
# Without this allowlist, the tool treats any test_foo_<suffix>.py as a
# qualified test for foo.py (by stripping <suffix>).
# With this allowlist, only listed suffixes are treated as qualifiers.
# If you add a new qualifier-style test, add its suffix here.
allowed_qualifiers = ["validation", "smoke"]
```
