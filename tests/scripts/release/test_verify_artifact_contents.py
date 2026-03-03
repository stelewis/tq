"""Tests for artifact content verification script."""

from __future__ import annotations

import io
import sys
import tarfile
import zipfile
from typing import TYPE_CHECKING

import pytest
from scripts.release import verify_artifact_contents

if TYPE_CHECKING:
    from pathlib import Path


def test_main_exits_when_dist_dir_missing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Exit with an actionable error when dist directory is absent."""
    dist_dir = tmp_path / "missing-dist"
    monkeypatch.setattr(
        sys,
        "argv",
        ["verify_artifact_contents.py", "--dist-dir", str(dist_dir)],
    )

    with pytest.raises(SystemExit, match="Distribution directory does not exist"):
        verify_artifact_contents.main()


def test_main_reports_forbidden_members(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Report every forbidden path found in wheel and sdist artifacts."""
    dist_dir = tmp_path / "dist"
    dist_dir.mkdir()

    wheel_path = dist_dir / "pkg-0.1.0-py3-none-any.whl"
    with zipfile.ZipFile(wheel_path, mode="w") as wheel:
        wheel.writestr("tq/__init__.py", "")
        wheel.writestr("scripts/docs/generate.py", "")

    sdist_path = dist_dir / "pkg-0.1.0.tar.gz"
    with tarfile.open(sdist_path, mode="w:gz") as archive:
        _add_tar_text_member(
            archive=archive,
            member_name="pkg-0.1.0/tq/__init__.py",
            content="",
        )
        _add_tar_text_member(
            archive=archive,
            member_name="pkg-0.1.0/tests/test_x.py",
            content="",
        )

    monkeypatch.setattr(
        sys,
        "argv",
        [
            "verify_artifact_contents.py",
            "--dist-dir",
            str(dist_dir),
            "--forbidden-prefix",
            "scripts/",
            "--forbidden-prefix",
            "tests/",
        ],
    )

    with pytest.raises(
        SystemExit,
        match="Artifact content policy check failed",
    ) as error:
        verify_artifact_contents.main()

    message = str(error.value)
    assert "scripts/docs/generate.py" in message
    assert "pkg-0.1.0/tests/test_x.py" in message


def test_main_passes_when_no_violations(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Return successfully when artifacts satisfy the path policy."""
    dist_dir = tmp_path / "dist"
    dist_dir.mkdir()

    wheel_path = dist_dir / "pkg-0.1.0-py3-none-any.whl"
    with zipfile.ZipFile(wheel_path, mode="w") as wheel:
        wheel.writestr("tq/__init__.py", "")

    monkeypatch.setattr(
        sys,
        "argv",
        [
            "verify_artifact_contents.py",
            "--dist-dir",
            str(dist_dir),
            "--forbidden-prefix",
            "tests/",
        ],
    )

    verify_artifact_contents.main()


def _add_tar_text_member(
    *,
    archive: tarfile.TarFile,
    member_name: str,
    content: str,
) -> None:
    """Add UTF-8 text file content to a tar archive member."""
    payload = content.encode("utf-8")
    info = tarfile.TarInfo(name=member_name)
    info.size = len(payload)
    archive.addfile(info, io.BytesIO(payload))
