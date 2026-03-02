"""Tests for test_quality.config module."""

from __future__ import annotations

import warnings
from pathlib import Path

import pytest

from tq.tools.test_quality.config import TestQualityConfig

# Suppress expected warnings about test_quality classes that pytest tries to collect
warnings.filterwarnings("ignore", category=pytest.PytestCollectionWarning)


def test_default_config():
    """Test default configuration values."""
    config = TestQualityConfig()

    assert config.ignore == []
    assert config.max_test_lines == 600
    assert config.ignore_init is False


def test_custom_config():
    """Test creating config with custom values."""
    config = TestQualityConfig(
        ignore=["**/test_*.py"],
        max_test_lines=500,
        ignore_init=True,
    )

    assert config.ignore == ["**/test_*.py"]
    assert config.max_test_lines == 500
    assert config.ignore_init is True


def test_from_pyproject_not_found():
    """Test loading config when pyproject.toml doesn't exist."""
    config = TestQualityConfig.from_pyproject(Path("/nonexistent/pyproject.toml"))

    # Should return default config
    assert config.ignore == []
    assert config.max_test_lines == 600
    assert config.ignore_init is False


def test_from_pyproject_no_config_section(tmp_path: Path):
    """Test loading config when pyproject.toml exists but has no config section."""
    pyproject = tmp_path / "pyproject.toml"
    pyproject.write_text(
        """
[project]
name = "test"
version = "0.1.0"
"""
    )

    config = TestQualityConfig.from_pyproject(pyproject)

    # Should return default config
    assert config.ignore == []
    assert config.max_test_lines == 600
    assert config.ignore_init is False


def test_from_pyproject_with_config(tmp_path: Path):
    """Test loading config from pyproject.toml with config section."""
    pyproject = tmp_path / "pyproject.toml"
    pyproject.write_text(
        """
[project]
name = "test"

[tool.test_quality]
ignore = ["**/integration/**", "tests/e2e/**"]
max_test_lines = 500
ignore_init = true
"""
    )

    config = TestQualityConfig.from_pyproject(pyproject)

    assert config.ignore == ["**/integration/**", "tests/e2e/**"]
    assert config.max_test_lines == 500
    assert config.ignore_init is True


def test_from_pyproject_partial_config(tmp_path: Path):
    """Test loading config with only some values specified."""
    pyproject = tmp_path / "pyproject.toml"
    pyproject.write_text(
        """
    [tool.test_quality]
max_test_lines = 700
"""
    )

    config = TestQualityConfig.from_pyproject(pyproject)

    # Specified values
    assert config.max_test_lines == 700

    # Default values
    assert config.ignore == []
    assert config.ignore_init is False


def test_find_pyproject_in_cwd(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Test finding pyproject.toml in current directory."""
    pyproject = tmp_path / "pyproject.toml"
    pyproject.write_text(
        """
    [tool.test_quality]
max_test_lines = 800
"""
    )

    monkeypatch.chdir(tmp_path)
    config = TestQualityConfig.from_pyproject()

    assert config.max_test_lines == 800


def test_find_pyproject_in_parent(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Test finding pyproject.toml in parent directory."""
    pyproject = tmp_path / "pyproject.toml"
    pyproject.write_text(
        """
    [tool.test_quality]
max_test_lines = 900
"""
    )

    subdir = tmp_path / "subdir"
    subdir.mkdir()

    monkeypatch.chdir(subdir)
    config = TestQualityConfig.from_pyproject()

    assert config.max_test_lines == 900
