"""Tests for runner module."""

from __future__ import annotations

import warnings
from pathlib import Path

import pytest

from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.models import Severity
from tq.tools.test_quality.runner import TestQualityRunner

# Suppress expected warnings about test_quality classes that pytest tries to collect
warnings.filterwarnings("ignore", category=pytest.PytestCollectionWarning)


def test_runner_basic(tmp_path: Path):
    """Test basic runner operation."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source and test
    (src / "foo.py").write_text("# foo")
    (tests / "tq").mkdir()
    (tests / "tq" / "test_foo.py").write_text("# test foo")

    config = TestQualityConfig(ignore_init=True)
    runner = TestQualityRunner(src, tests, config)
    findings = runner.run()

    # Should have no findings for properly structured project
    assert isinstance(findings, list)


def test_runner_has_errors(tmp_path: Path):
    """Test runner detects errors."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source without test (error)
    (src / "foo.py").write_text("# foo")

    config = TestQualityConfig(ignore_init=True)
    runner = TestQualityRunner(src, tests, config)
    runner.run()

    assert runner.has_errors() is True


def test_runner_no_errors(tmp_path: Path):
    """Test runner when no errors present."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source with test
    (src / "foo.py").write_text("# foo")
    (tests / "tq").mkdir()
    (tests / "tq" / "test_foo.py").write_text("# test foo")

    config = TestQualityConfig(ignore_init=True)
    runner = TestQualityRunner(src, tests, config)
    runner.run()

    assert runner.has_errors() is False


def test_runner_summary(tmp_path: Path):
    """Test runner summary generation."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source without test (error)
    (src / "foo.py").write_text("# foo")

    # Create orphaned test (warning)
    (tests / "tq").mkdir()
    (tests / "tq" / "test_orphan.py").write_text("# test orphan")

    config = TestQualityConfig(ignore_init=True)
    runner = TestQualityRunner(src, tests, config)
    runner.run()

    summary = runner.get_summary()

    assert summary[Severity.ERROR.value] >= 1  # Missing test
    assert summary[Severity.WARNING.value] >= 1  # Orphaned test


def test_runner_default_config(tmp_path: Path):
    """Test runner with default config loading."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Runner should load default config if none provided
    runner = TestQualityRunner(src, tests)
    assert runner.config is not None
    assert isinstance(runner.config, TestQualityConfig)


def test_runner_redundant_ignore(tmp_path: Path):
    """Test runner detects redundant ignore patterns."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create a file
    (src / "foo.py").write_text("# foo")
    (tests / "tq").mkdir()
    (tests / "tq" / "test_foo.py").write_text("# test foo")

    # Config with pattern that doesn't match anything
    config = TestQualityConfig(
        ignore=["nonexistent/**/*.py"],
        ignore_init=True,
    )
    runner = TestQualityRunner(src, tests, config)
    findings = runner.run()

    # Should have finding about redundant ignore
    redundant_findings = [f for f in findings if f.category == "redundant_ignore"]
    assert len(redundant_findings) == 1
    assert redundant_findings[0].severity == Severity.WARNING


def test_runner_multiple_checks(tmp_path: Path):
    """Test runner executes multiple checks."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source without test (mapping error)
    (src / "missing.py").write_text("# missing")

    # Create large test file (size warning)
    large_content = "\n".join([f"def test_{i}():\n    pass" for i in range(400)])
    (tests / "tq").mkdir()
    (tests / "tq" / "test_large.py").write_text(large_content)

    config = TestQualityConfig(ignore_init=True, max_test_lines=100)
    runner = TestQualityRunner(src, tests, config)
    findings = runner.run()

    # Should have findings from multiple checks
    categories = {f.category for f in findings}
    assert "mapping_missing" in categories
    assert "large_test_file" in categories


def test_runner_with_ignore_patterns(tmp_path: Path):
    """Test runner respects ignore patterns."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source files
    (src / "foo.py").write_text("# foo")
    (src / "ignored.py").write_text("# ignored")

    config = TestQualityConfig(ignore=["**/ignored.py"], ignore_init=True)
    runner = TestQualityRunner(src, tests, config)
    findings = runner.run()

    # Should only report missing test for foo.py, not ignored.py
    missing_findings = [f for f in findings if f.category == "mapping_missing"]
    paths = [str(f.path) for f in missing_findings]

    assert any("foo.py" in p for p in paths)
    assert not any("ignored.py" in p for p in paths)
