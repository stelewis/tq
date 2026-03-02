"""Tests for structure_check module."""

from __future__ import annotations

import warnings
from pathlib import Path

import pytest

from tq.tools.test_quality.checks.structure_check import StructureCheck
from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import scan_files

# Suppress expected warnings about test_quality classes that pytest tries to collect
warnings.filterwarnings("ignore", category=pytest.PytestCollectionWarning)


def test_structure_check_correct_structure(tmp_path: Path):
    """Test structure check when tests are in correct locations."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create properly structured test
    (tests / "tq").mkdir()
    (tests / "tq" / "test_foo.py").write_text("# test foo")

    index = scan_files(src, tests)
    config = TestQualityConfig()
    check = StructureCheck(index, config)

    findings = check.run()
    # No structure mismatches for properly placed tests
    assert len(findings) == 0


def test_structure_check_integration_excluded(tmp_path: Path):
    """Test that integration tests are excluded from structure check."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create integration test in any structure
    (tests / "integration").mkdir()
    (tests / "integration" / "test_workflow.py").write_text("# test workflow")

    index = scan_files(src, tests)
    config = TestQualityConfig()
    check = StructureCheck(index, config)

    findings = check.run()
    # Integration tests should not be flagged for structure
    assert len(findings) == 0


def test_structure_check_e2e_excluded(tmp_path: Path):
    """Test that e2e tests are excluded from structure check."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create e2e test
    (tests / "e2e").mkdir()
    (tests / "e2e" / "test_pipeline.py").write_text("# test pipeline")

    index = scan_files(src, tests)
    config = TestQualityConfig()
    check = StructureCheck(index, config)

    findings = check.run()
    # E2E tests should not be flagged for structure
    assert len(findings) == 0


def test_structure_check_nested_correct(tmp_path: Path):
    """Test structure check with correctly nested tests."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create nested test in correct structure
    (tests / "tq").mkdir()
    (tests / "tq" / "pipeline").mkdir()
    utils_test = tests / "tq" / "pipeline" / "test_utils.py"
    utils_test.write_text("# test utils")

    index = scan_files(src, tests)
    config = TestQualityConfig()
    check = StructureCheck(index, config)

    findings = check.run()
    assert len(findings) == 0
