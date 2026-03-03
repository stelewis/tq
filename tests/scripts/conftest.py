"""Test helpers for script-module tests."""

from __future__ import annotations

import sys
from pathlib import Path


def _ensure_repo_root_on_path() -> None:
    """Make repository-root packages importable for script tests."""
    repo_root = Path(__file__).resolve().parents[2]
    root_text = str(repo_root)
    if root_text not in sys.path:
        sys.path.insert(0, root_text)


_ensure_repo_root_on_path()
