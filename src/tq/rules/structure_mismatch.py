"""Structure rule for unit test placement mismatches."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from tq.engine.models import Finding, Severity
from tq.engine.rule_id import RuleId

if TYPE_CHECKING:
    from tq.engine.context import AnalysisContext


class StructureMismatchRule:
    """Emit findings for test files that do not mirror source structure."""

    @property
    def rule_id(self) -> RuleId:
        """Return stable rule identifier."""
        return RuleId("structure-mismatch")

    def evaluate(self, context: AnalysisContext) -> tuple[Finding, ...]:
        """Evaluate test structure alignment against the analysis index."""
        package_name = context.index.source_root.name
        source_files = set(context.index.source_files)
        findings: list[Finding] = []

        for test_file in context.index.test_files:
            if _is_non_unit_test_path(test_file):
                continue

            if not _is_unit_test_filename(test_file.name):
                continue

            if test_file.parts[0] != package_name:
                findings.append(
                    Finding(
                        rule_id=self.rule_id,
                        severity=Severity.WARNING,
                        message="Unit test is not located under the package test root",
                        path=context.index.test_root / test_file,
                        suggestion=(
                            "Move test under: "
                            f"{(Path(package_name) / test_file.name).as_posix()}"
                        ),
                    ),
                )
                continue

            expected_path = _expected_path_for_test_file(
                test_file=test_file,
                source_files=source_files,
                package_name=package_name,
            )
            if expected_path is None or expected_path == test_file:
                continue

            findings.append(
                Finding(
                    rule_id=self.rule_id,
                    severity=Severity.WARNING,
                    message="Test file is not in the expected location",
                    path=context.index.test_root / test_file,
                    suggestion=f"Move to: {expected_path.as_posix()}",
                ),
            )

        return tuple(findings)


def _expected_path_for_test_file(
    *,
    test_file: Path,
    source_files: set[Path],
    package_name: str,
) -> Path | None:
    """Infer canonical path when target source module can be resolved."""
    source_candidate = _resolve_source_candidate(
        test_file=test_file,
        source_files=source_files,
    )
    if source_candidate is None:
        return None

    expected_name = _expected_test_name(test_file=test_file)
    return Path(package_name) / source_candidate.parent / expected_name


def _resolve_source_candidate(
    *,
    test_file: Path,
    source_files: set[Path],
) -> Path | None:
    """Resolve source file for a test path using deterministic candidates."""
    module_stem = test_file.stem[5:]
    candidates = _candidate_source_paths(test_file=test_file, module_stem=module_stem)
    for candidate in candidates:
        if candidate in source_files:
            return candidate

    bare_name = f"{module_stem.split('_')[0]}.py"
    same_name_sources = [path for path in source_files if path.name == bare_name]
    if len(same_name_sources) == 1:
        return same_name_sources[0]

    return None


def _candidate_source_paths(*, test_file: Path, module_stem: str) -> tuple[Path, ...]:
    """Build source candidates from unit test file names."""
    relative_parts = test_file.parts[1:-1]
    direct_source = Path(*relative_parts) / f"{module_stem}.py"

    if "_" not in module_stem:
        return (direct_source,)

    prefixes: list[Path] = [direct_source]
    name_parts = module_stem.split("_")
    for index in range(len(name_parts) - 1, 0, -1):
        prefix = "_".join(name_parts[:index])
        prefixes.append(Path(*relative_parts) / f"{prefix}.py")

    return tuple(prefixes)


def _expected_test_name(*, test_file: Path) -> str:
    """Return canonical test file name preserving qualifiers."""
    return test_file.name


def _is_non_unit_test_path(test_file: Path) -> bool:
    """Check whether path is integration or end-to-end scope."""
    return "integration" in test_file.parts or "e2e" in test_file.parts


def _is_unit_test_filename(filename: str) -> bool:
    """Check if filename follows unit test naming shape."""
    return filename.startswith("test_") and filename.endswith(".py")
