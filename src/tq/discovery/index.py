"""Immutable analysis index for source and test file discovery."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable
    from pathlib import Path


@dataclass(frozen=True, slots=True)
class AnalysisIndex:
    """Immutable index of discovered source and test files.

    Attributes:
        source_root: Root path of the scanned source tree.
        test_root: Root path of the scanned test tree.
        source_files: Source module paths relative to source_root.
        test_files: Test module paths relative to test_root.
    """

    source_root: Path
    test_root: Path
    source_files: tuple[Path, ...]
    test_files: tuple[Path, ...]

    @classmethod
    def create(
        cls,
        *,
        source_root: Path,
        test_root: Path,
        source_files: Iterable[Path],
        test_files: Iterable[Path],
    ) -> AnalysisIndex:
        """Create an index with deterministic file ordering.

        Args:
            source_root: Root path of the scanned source tree.
            test_root: Root path of the scanned test tree.
            source_files: Relative source module paths.
            test_files: Relative test module paths.

        Returns:
            AnalysisIndex with sorted immutable file collections.
        """
        normalized_source_root = source_root.resolve()
        normalized_test_root = test_root.resolve()

        return cls(
            source_root=normalized_source_root,
            test_root=normalized_test_root,
            source_files=tuple(sorted(set(source_files))),
            test_files=tuple(sorted(set(test_files))),
        )
