"""Reject accepted examples of abandoned Rust interop draft syntax."""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
SCAN_ROOTS = ("docs", "internal_docs", "plans")
SKIP_PARTS = {"archive", "reviews"}

STALE_PATTERNS = {
    "extern rust": re.compile(r"\bextern\s+rust\b"),
    "from rust import": re.compile(r"\bfrom\s+rust\s+import\b"),
    "rust dynamic loading": re.compile(r"\bdlopen\b"),
    "legacy rust decorator keywords": re.compile(r"@rust\s*\([^)]*\b(crate|path)\s*="),
    "legacy native trust key": re.compile(r"(?<!python-)native\s*=\s*\["),
}

RUST_TARGET_DECORATOR = re.compile(r"^\s*@rust\s*\(")
SIFR_INTEROP_DECORATOR = re.compile(r"@rust(?:\.|\s*\()")


def main() -> int:
    failures: list[str] = []
    for root in SCAN_ROOTS:
        for path in sorted((REPO_ROOT / root).rglob("*")):
            if path.suffix not in {".md", ".mdx"}:
                continue
            if any(part in SKIP_PARTS for part in path.relative_to(REPO_ROOT).parts):
                continue
            _scan_path(path, failures)

    if failures:
        for failure in failures:
            print(f"rust interop stale draft error: {failure}", file=sys.stderr)
        return 1
    print("rust interop stale draft scan ok")
    return 0


def _scan_path(path: Path, failures: list[str]) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    fence_language: str | None = None
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("```"):
            fence_language = _next_fence_language(stripped, fence_language)
            continue
        if fence_language == "python" and SIFR_INTEROP_DECORATOR.search(line):
            display = path.relative_to(REPO_ROOT)
            failures.append(f"{display}:{line_number}: Sifr Rust interop example uses python fence")
        for label, pattern in STALE_PATTERNS.items():
            match = pattern.search(line)
            if match and not _is_rejection_context(line, match.start()):
                display = path.relative_to(REPO_ROOT)
                failures.append(f"{display}:{line_number}: accepted {label} syntax")
        if RUST_TARGET_DECORATOR.search(line) and "panic=" not in line:
            _scan_panic_surface(path, lines, line_number, failures)


def _next_fence_language(stripped: str, current: str | None) -> str | None:
    if current is not None:
        return None
    return stripped.removeprefix("```").strip().lower() or ""


def _is_rejection_context(line: str, match_start: int) -> bool:
    prefix = line[:match_start].lower()
    return any(
        marker in prefix
        for marker in (
            "rejected",
            "reject ",
            "rejects ",
            "no ",
            "not ",
            "does not use",
            "out of scope",
            "out-of-scope",
            "non-goal",
            "stale",
            "abandoned",
            "remove",
        )
    )


def _scan_panic_surface(
    path: Path,
    lines: list[str],
    decorator_line_number: int,
    failures: list[str],
) -> None:
    lookahead = lines[decorator_line_number : decorator_line_number + 6]
    definition = next((line.strip() for line in lookahead if line.strip().startswith(("def ", "async def "))), None)
    if definition is None:
        return
    if "RustPanicError" in definition or "Result[" not in definition:
        return
    display = path.relative_to(REPO_ROOT)
    failures.append(
        f"{display}:{decorator_line_number}: Rust interop example lacks RustPanicError or panic policy"
    )


if __name__ == "__main__":
    raise SystemExit(main())
