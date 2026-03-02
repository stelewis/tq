"""Configuration models for tq CLI composition."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from tq.engine.rule_id import RuleId
from tq.rules.qualifiers import QualifierStrategy


class ConfigValidationError(ValueError):
    """Raised when tq configuration is invalid or contradictory."""


@dataclass(frozen=True, slots=True)
class PartialTqConfig:
    """Partial tq configuration loaded from config files and CLI flags."""

    package: str | None = None
    source_root: str | None = None
    test_root: str | None = None
    ignore_init_modules: bool | None = None
    max_test_file_non_blank_lines: int | None = None
    qualifier_strategy: QualifierStrategy | None = None
    allowed_qualifiers: tuple[str, ...] | None = None
    select: tuple[RuleId, ...] | None = None
    ignore: tuple[RuleId, ...] | None = None


@dataclass(frozen=True, slots=True)
class TqConfig:
    """Resolved tq configuration used by the command runtime."""

    package: str
    source_root: Path
    test_root: Path
    ignore_init_modules: bool
    max_test_file_non_blank_lines: int
    qualifier_strategy: QualifierStrategy
    allowed_qualifiers: tuple[str, ...]
    select: tuple[RuleId, ...]
    ignore: tuple[RuleId, ...]

    @property
    def package_path(self) -> Path:
        """Return package as a path under source and test roots."""
        return Path(*self.package.split("."))

    @property
    def source_package_root(self) -> Path:
        """Return resolved source package root path."""
        return self.source_root / self.package_path


@dataclass(frozen=True, slots=True)
class CliOverrides:
    """CLI-level configuration overrides."""

    package: str | None = None
    source_root: str | None = None
    test_root: str | None = None
    ignore_init_modules: bool | None = None
    max_test_file_non_blank_lines: int | None = None
    qualifier_strategy: QualifierStrategy | None = None
    allowed_qualifiers: tuple[str, ...] | None = None
    select: tuple[RuleId, ...] | None = None
    ignore: tuple[RuleId, ...] | None = None
