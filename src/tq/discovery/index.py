"""Immutable analysis index for source and test file discovery."""

from __future__ import annotations

from dataclasses import dataclass
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
        source_files: list[Path] | tuple[Path, ...],
        test_files: list[Path] | tuple[Path, ...],
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
        return cls(
            source_root=source_root,
            test_root=test_root,
            source_files=tuple(sorted(source_files)),
            test_files=tuple(sorted(test_files)),
        )
