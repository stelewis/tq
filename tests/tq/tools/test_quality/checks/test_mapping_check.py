"""Tests for mapping_check module."""

from __future__ import annotations

import warnings
from pathlib import Path

import pytest

from tq.tools.test_quality.checks.mapping_check import MappingCheck
from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import scan_files
from tq.tools.test_quality.models import Severity

# Suppress expected warnings about test_quality classes that pytest tries to collect
warnings.filterwarnings("ignore", category=pytest.PytestCollectionWarning)


def test_mapping_check_no_missing(tmp_path: Path):
    """Test mapping check when all source files have tests."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source and matching test
    (src / "foo.py").write_text("# foo")
    (tests / "tq").mkdir()
    (tests / "tq" / "test_foo.py").write_text("# test foo")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = MappingCheck(index, config)

    findings = check.run()
    assert len(findings) == 0


def test_mapping_check_missing_test(tmp_path: Path):
    """Test mapping check when a source file has no test."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source without test
    (src / "foo.py").write_text("# foo")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = MappingCheck(index, config)

    findings = check.run()
    assert len(findings) == 1
    assert findings[0].category == "mapping_missing"
    assert findings[0].severity == Severity.ERROR
    assert "foo.py" in str(findings[0].path)


def test_mapping_check_with_qualifier(tmp_path: Path):
    """Test mapping check accepts tests with qualifiers."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source
    (src / "foo.py").write_text("# foo")

    # Create test with qualifier
    (tests / "tq").mkdir()
    validation_test = tests / "tq" / "test_foo_validation.py"
    validation_test.write_text("# test foo validation")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = MappingCheck(index, config)

    findings = check.run()
    assert len(findings) == 0


def test_mapping_check_nested_structure(tmp_path: Path):
    """Test mapping check with nested directory structure."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create nested source
    (src / "pipeline").mkdir()
    (src / "pipeline" / "utils").mkdir()
    (src / "pipeline" / "utils" / "similarity.py").write_text("# similarity")

    # Create matching test
    (tests / "tq").mkdir()
    (tests / "tq" / "pipeline").mkdir()
    (tests / "tq" / "pipeline" / "utils").mkdir()
    similarity_test = tests.joinpath(
        "tq",
        "pipeline",
        "utils",
        "test_similarity.py",
    )
    similarity_test.write_text("# test similarity")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = MappingCheck(index, config)

    findings = check.run()
    assert len(findings) == 0


def test_mapping_check_multiple_tests_same_module(tmp_path: Path):
    """Test that multiple tests for the same module are allowed."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source
    (src / "foo.py").write_text("# foo")

    # Create multiple tests
    (tests / "tq").mkdir()
    (tests / "tq" / "test_foo.py").write_text("# test foo")
    validation_test = tests / "tq" / "test_foo_validation.py"
    integration_test = tests / "tq" / "test_foo_integration.py"
    validation_test.write_text("# test foo validation")
    integration_test.write_text("# test foo integration")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = MappingCheck(index, config)

    findings = check.run()
    # Should not report missing tests since at least one exists
    assert len(findings) == 0


def test_mapping_check_init_file(tmp_path: Path):
    """Test mapping check with __init__.py files."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create __init__.py in a package
    (src / "pipeline").mkdir()
    (src / "pipeline" / "__init__.py").write_text("# pipeline init")

    # Create matching test
    (tests / "tq").mkdir()
    (tests / "tq" / "pipeline").mkdir()
    (tests / "tq" / "pipeline" / "test_pipeline_init.py").write_text(
        "# test pipeline init"
    )

    index = scan_files(src, tests, ignore_init=False)
    config = TestQualityConfig(ignore_init=False)
    check = MappingCheck(index, config)

    findings = check.run()
    assert len(findings) == 0
