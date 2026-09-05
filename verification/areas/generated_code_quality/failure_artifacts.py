"""Bound retained artifacts from failed generated-code quality runs."""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path


def preserve_clippy_output(
    run_root: Path,
    entry_id: str,
    result: subprocess.CompletedProcess[str],
) -> None:
    """Preserve exact compiler output before generated workspaces are removed."""
    diagnostics = run_root / "diagnostics"
    diagnostics.mkdir(parents=True, exist_ok=True)
    (diagnostics / f"{entry_id}.stdout.jsonl").write_text(
        result.stdout,
        encoding="utf-8",
    )
    (diagnostics / f"{entry_id}.stderr.log").write_text(
        result.stderr,
        encoding="utf-8",
    )


def discard_failed_run_cargo_target(run_root: Path, cargo_target_dir: Path) -> None:
    """Keep source and diagnostic evidence while removing compiler artifacts."""
    shutil.rmtree(cargo_target_dir, ignore_errors=True)
    marker = run_root / "failed-cargo-target-cleanup.json"
    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.write_text(
        json.dumps(
            {"removed": cargo_target_dir.relative_to(run_root).as_posix()},
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
