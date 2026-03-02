"""Verify built artifacts do not contain forbidden repository paths."""

from __future__ import annotations

import argparse
import tarfile
import zipfile
from dataclasses import dataclass
from pathlib import Path

_DEFAULT_FORBIDDEN_PREFIXES = (
    "scripts/",
    "tests/",
    "docs/",
    "tmp/",
    ".github/",
)
_SDIST_SPLIT_PARTS = 2


@dataclass(frozen=True, slots=True)
class ArtifactViolation:
    """A forbidden path discovered in a build artifact."""

    artifact: Path
    member: str


def _strip_sdist_root(member: str) -> str:
    """Strip top-level sdist folder prefix from tar member path."""
    parts = member.split("/", 1)
    return parts[1] if len(parts) == _SDIST_SPLIT_PARTS else ""


def _find_wheel_violations(
    *,
    wheel_path: Path,
    forbidden_prefixes: tuple[str, ...],
) -> tuple[ArtifactViolation, ...]:
    """Find forbidden members in a wheel archive."""
    with zipfile.ZipFile(wheel_path) as wheel_file:
        return tuple(
            ArtifactViolation(artifact=wheel_path, member=member)
            for member in wheel_file.namelist()
            if any(member.startswith(prefix) for prefix in forbidden_prefixes)
        )


def _find_sdist_violations(
    *,
    sdist_path: Path,
    forbidden_prefixes: tuple[str, ...],
) -> tuple[ArtifactViolation, ...]:
    """Find forbidden members in an sdist archive."""
    violations: list[ArtifactViolation] = []
    with tarfile.open(sdist_path, "r:gz") as sdist_file:
        for member in sdist_file.getnames():
            normalized = _strip_sdist_root(member)
            if any(normalized.startswith(prefix) for prefix in forbidden_prefixes):
                violations.append(ArtifactViolation(artifact=sdist_path, member=member))
    return tuple(violations)


def _collect_violations(
    *,
    dist_dir: Path,
    forbidden_prefixes: tuple[str, ...],
) -> tuple[ArtifactViolation, ...]:
    """Collect forbidden-path violations from all built wheel and sdist files."""
    violations: list[ArtifactViolation] = []

    for wheel_path in sorted(dist_dir.glob("*.whl")):
        violations.extend(
            _find_wheel_violations(
                wheel_path=wheel_path,
                forbidden_prefixes=forbidden_prefixes,
            ),
        )

    for sdist_path in sorted(dist_dir.glob("*.tar.gz")):
        violations.extend(
            _find_sdist_violations(
                sdist_path=sdist_path,
                forbidden_prefixes=forbidden_prefixes,
            ),
        )

    return tuple(violations)


def _parse_args() -> argparse.Namespace:
    """Parse CLI args for artifact content verification."""
    parser = argparse.ArgumentParser(
        description="Verify package artifacts do not include forbidden paths.",
    )
    parser.add_argument(
        "--dist-dir",
        default="dist",
        help="Directory containing built artifacts (default: dist).",
    )
    parser.add_argument(
        "--forbidden-prefix",
        dest="forbidden_prefixes",
        action="append",
        default=None,
        help="Forbidden path prefix (repeatable).",
    )
    return parser.parse_args()


def main() -> None:
    """Run package artifact content policy checks."""
    args = _parse_args()
    dist_dir = Path(args.dist_dir)
    forbidden_prefixes = tuple(args.forbidden_prefixes or _DEFAULT_FORBIDDEN_PREFIXES)

    if not dist_dir.exists():
        msg = f"Distribution directory does not exist: {dist_dir}"
        raise SystemExit(msg)

    violations = _collect_violations(
        dist_dir=dist_dir,
        forbidden_prefixes=forbidden_prefixes,
    )

    if not violations:
        return

    lines = [
        "Artifact content policy check failed.",
        "Forbidden paths were found in build artifacts:",
        *[
            f"- {violation.artifact.name}: {violation.member}"
            for violation in violations
        ],
    ]
    raise SystemExit("\n".join(lines))


if __name__ == "__main__":
    main()
