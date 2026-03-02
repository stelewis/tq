"""Tests for orphan_check module."""

from __future__ import annotations

import warnings
from pathlib import Path

import pytest

from tq.tools.test_quality.checks.orphan_check import OrphanCheck
from tq.tools.test_quality.config import TestQualityConfig
from tq.tools.test_quality.filesystem import scan_files
from tq.tools.test_quality.models import Severity

# Suppress expected warnings about test_quality classes that pytest tries to collect
warnings.filterwarnings("ignore", category=pytest.PytestCollectionWarning)


def test_orphan_check_no_orphans(tmp_path: Path):
    """Test orphan check when all tests have corresponding source."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source and test
    (src / "foo.py").write_text("# foo")
    (tests / "tq").mkdir()
    (tests / "tq" / "test_foo.py").write_text("# test foo")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = OrphanCheck(index, config)

    findings = check.run()
    assert len(findings) == 0


def test_orphan_check_literal_init_test(tmp_path: Path):
    """Test orphan check handles test___init__.py pattern."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create package __init__.py
    (src / "config").mkdir()
    (src / "config" / "__init__.py").write_text("# config init")

    # Create test for __init__.py using literal naming
    (tests / "tq").mkdir()
    (tests / "tq" / "config").mkdir()
    init_test = tests / "tq" / "config" / "test___init__.py"
    init_test.write_text("# test config init")

    index = scan_files(src, tests, ignore_init=False)
    config = TestQualityConfig(ignore_init=False)
    check = OrphanCheck(index, config)

    findings = check.run()
    # Should recognize test___init__.py as testing config/__init__.py
    assert len(findings) == 0


def test_orphan_check_multi_word_qualifier(tmp_path: Path):
    """Test orphan check handles multi-word qualifiers."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create source
    (src / "types").mkdir()
    (src / "types" / "income_record.py").write_text("# income record")

    # Create test with multi-word qualifier
    (tests / "tq").mkdir()
    (tests / "tq" / "types").mkdir()
    validation_test = tests.joinpath(
        "tq",
        "types",
        "test_income_record_validation.py",
    )
    validation_test.write_text("# test income record validation")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = OrphanCheck(index, config)

    findings = check.run()
    # Should recognize test_income_record_validation.py as testing income_record.py
    assert len(findings) == 0


def test_orphan_check_qualifier_allowlist_blocks_unknown_suffix(tmp_path: Path):
    """Unknown qualifier suffixes should be treated as orphaned when restricted."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create a source module that could be (incorrectly) matched by stripping.
    (src / "cli").mkdir()
    (src / "cli" / "commands").mkdir(parents=True)
    (src / "cli" / "commands" / "example.py").write_text("# example")

    # Create a test that *looks* like test_<module>_<qualifier>.py,
    # but where the suffix is not an allowed qualifier.
    (tests / "tq").mkdir()
    (tests / "tq" / "cli").mkdir()
    (tests / "tq" / "cli" / "commands").mkdir(parents=True)
    suffix_test = tests.joinpath(
        "tq",
        "cli",
        "commands",
        "test_example_suffix.py",
    )
    suffix_test.write_text("# test example suffix")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True, allowed_qualifiers=["validation"])
    check = OrphanCheck(index, config)

    findings = check.run()
    assert len(findings) == 1
    assert findings[0].category == "orphaned_test"
    assert findings[0].severity == Severity.WARNING
    assert "test_example_suffix.py" in str(findings[0].path)


def test_orphan_check_qualifier_allowlist_allows_known_suffix(tmp_path: Path):
    """Allowed qualifiers should still resolve to the base module."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    (src / "foo.py").write_text("# foo")

    (tests / "tq").mkdir()
    validation_test = tests / "tq" / "test_foo_validation.py"
    validation_test.write_text("# test foo validation")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True, allowed_qualifiers=["validation"])
    check = OrphanCheck(index, config)

    findings = check.run()
    assert len(findings) == 0


def test_orphan_check_orphaned_test(tmp_path: Path):
    """Test orphan check detects test without source."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create test without source
    (tests / "tq").mkdir()
    (tests / "tq" / "test_orphan.py").write_text("# test orphan")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = OrphanCheck(index, config)

    findings = check.run()
    assert len(findings) == 1
    assert findings[0].category == "orphaned_test"
    assert findings[0].severity == Severity.WARNING
    assert "orphan" in str(findings[0].path)


def test_orphan_check_integration_excluded(tmp_path: Path):
    """Test that integration tests are excluded from orphan check."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create integration test without source
    (tests / "integration").mkdir()
    (tests / "integration" / "test_workflow.py").write_text("# test workflow")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = OrphanCheck(index, config)

    findings = check.run()
    # Integration tests should not be flagged as orphaned
    assert len(findings) == 0


def test_orphan_check_e2e_excluded(tmp_path: Path):
    """Test that e2e tests are excluded from orphan check."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create e2e test without source
    (tests / "e2e").mkdir()
    (tests / "e2e" / "test_full_pipeline.py").write_text("# test pipeline")

    index = scan_files(src, tests, ignore_init=True)
    config = TestQualityConfig(ignore_init=True)
    check = OrphanCheck(index, config)

    findings = check.run()
    # E2E tests should not be flagged as orphaned
    assert len(findings) == 0


def test_orphan_check_with_qualifier(tmp_path: Path):
    """Test orphan check handles tests with qualifiers."""
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
    check = OrphanCheck(index, config)

    findings = check.run()
    # Test with qualifier should find corresponding source
    assert len(findings) == 0


def test_orphan_check_init_test(tmp_path: Path):
    """Test orphan check handles __init__.py tests."""
    src = tmp_path / "src" / "tq"
    tests = tmp_path / "tests"
    src.mkdir(parents=True)
    tests.mkdir(parents=True)

    # Create package __init__.py
    (src / "pipeline").mkdir()
    (src / "pipeline" / "__init__.py").write_text("# pipeline init")

    # Create test for __init__.py
    (tests / "tq").mkdir()
    (tests / "tq" / "pipeline").mkdir()
    (tests / "tq" / "pipeline" / "test_pipeline_init.py").write_text(
        "# test pipeline init"
    )

    index = scan_files(src, tests, ignore_init=False)
    config = TestQualityConfig(ignore_init=False)
    check = OrphanCheck(index, config)

    findings = check.run()
    # Should recognize test_pipeline_init.py as testing pipeline/__init__.py
    assert len(findings) == 0


def test_orphan_check_nested_structure(tmp_path: Path):
    """Test orphan check with nested directory structure."""
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
    check = OrphanCheck(index, config)

    findings = check.run()
    assert len(findings) == 0
