"""Retain diagnostics while reusing an invocation-owned Cargo cache."""
from pathlib import Path
import subprocess

def cargo_target_for_run(run_root: Path, shared_root: Path | None) -> Path:
    """Shared roots are prepared and owned by the calling verification profile."""
    return (shared_root if shared_root is not None else run_root) / "cargo-target"

def preserve_clippy_output(run_root: Path, entry_id: str,
                          result: subprocess.CompletedProcess[str]) -> None:
    diagnostics = run_root / "diagnostics"
    diagnostics.mkdir(parents=True, exist_ok=True)
    (diagnostics / f"{entry_id}.stdout.jsonl").write_text(result.stdout, encoding="utf-8")
    (diagnostics / f"{entry_id}.stderr.log").write_text(result.stderr, encoding="utf-8")
