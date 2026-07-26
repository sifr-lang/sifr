"""Reject accepted examples of abandoned Rust interop draft syntax."""

from __future__ import annotations

import re
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
SCAN_ROOTS = ("docs", "internal_docs", "plans")
SKIP_PARTS = {"archive", "reviews"}
REJECTED_FENCE = "```sifr-rejected"
INLINE_REJECTION_MARKERS = {
    ".md": "<!-- rust-interop-rejected -->",
    ".mdx": "{/* rust-interop-rejected */}",
}

STALE_PATTERNS = {
    "extern rust": re.compile(r"\bextern\s+rust\b"),
    "from rust import": re.compile(r"\bfrom\s+rust\s+import\b"),
    "rust dynamic loading": re.compile(r"\bdlopen\b"),
    "legacy rust decorator keywords": re.compile(r"@rust\s*\([^)]*\b(crate|path)\s*="),
    "legacy native trust key": re.compile(r"(?<!python-)native\s*=\s*\["),
}

RUST_TARGET_DECORATOR = re.compile(r"^\s*@rust\s*\(")
SIFR_INTEROP_DECORATOR = re.compile(r"@rust(?:\.|\s*\()")
FENCE_LINE = re.compile(r"^(?P<ticks>`{3,})(?P<info>[^`]*)$")
INLINE_CODE_SPAN = re.compile(r"(?P<ticks>`+).*?(?P=ticks)")
REJECTED_FENCE_PREFIXES = ("sifr-reject", "sifr reject", "sifr_reject")


@dataclass(frozen=True)
class Fence:
    tick_count: int
    language: str
    rejected: bool
    opening_line: int


def main() -> int:
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        return _run_self_test()
    if len(sys.argv) != 1:
        print("usage: check_stale_drafts.py [--self-test]", file=sys.stderr)
        return 2

    failures = _scan_repository(REPO_ROOT, require_all_roots=True)
    if failures:
        for failure in failures:
            print(f"rust interop stale draft error: {failure}", file=sys.stderr)
        return 1
    print("rust interop stale draft scan ok")
    return 0


def _scan_repository(repo_root: Path, *, require_all_roots: bool = False) -> list[str]:
    failures: list[str] = []
    scanned_paths = 0
    for root in SCAN_ROOTS:
        scan_root = repo_root / root
        if not scan_root.is_dir():
            if require_all_roots:
                failures.append(f"missing scan root: {root}")
            continue
        for path in sorted(scan_root.rglob("*")):
            if path.suffix not in {".md", ".mdx"}:
                continue
            if any(part in SKIP_PARTS for part in path.relative_to(repo_root).parts):
                continue
            scanned_paths += 1
            _scan_path(path, repo_root, failures)
    if require_all_roots and scanned_paths == 0:
        failures.append("scan roots contain no Markdown or MDX files")
    return failures


def _scan_path(path: Path, repo_root: Path, failures: list[str]) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    fence: Fence | None = None
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        fence_match = FENCE_LINE.fullmatch(stripped)
        if fence_match is not None:
            ticks = fence_match.group("ticks")
            info = fence_match.group("info").strip()
            if fence is not None and not info and len(ticks) >= fence.tick_count:
                fence = None
                continue
            if fence is None:
                is_rejected = len(ticks) == 3 and info == "sifr-rejected"
                if _looks_like_rejected_fence(info) and not is_rejected:
                    display = path.relative_to(repo_root)
                    failures.append(
                        f"{display}:{line_number}: malformed rejected fence; "
                        f"use exactly {REJECTED_FENCE}"
                    )
                _scan_stale_syntax(
                    path,
                    repo_root,
                    line_number,
                    line,
                    is_rejected or _line_has_rejection_marker(path, line),
                    failures,
                )
                fence = Fence(
                    tick_count=len(ticks),
                    language=info.split(maxsplit=1)[0].lower() if info else "",
                    rejected=is_rejected,
                    opening_line=line_number,
                )
                continue
        if fence is not None and fence.language == "python" and SIFR_INTEROP_DECORATOR.search(line):
            display = path.relative_to(repo_root)
            failures.append(f"{display}:{line_number}: Sifr Rust interop example uses python fence")
        explicitly_rejected = (fence is not None and fence.rejected) or _line_has_rejection_marker(path, line)
        _scan_stale_syntax(
            path,
            repo_root,
            line_number,
            line,
            explicitly_rejected,
            failures,
        )
        if (
            not explicitly_rejected
            and RUST_TARGET_DECORATOR.search(line)
            and "panic=" not in line
        ):
            _scan_panic_surface(path, repo_root, lines, line_number, failures)
    if fence is not None and fence.rejected:
        display = path.relative_to(repo_root)
        failures.append(
            f"{display}:{fence.opening_line}: unclosed {REJECTED_FENCE.removeprefix('```')} fence"
        )


def _looks_like_rejected_fence(info: str) -> bool:
    return info.lower().startswith(REJECTED_FENCE_PREFIXES)


def _line_has_rejection_marker(path: Path, line: str) -> bool:
    marker = INLINE_REJECTION_MARKERS[path.suffix]
    code_spans = tuple(INLINE_CODE_SPAN.finditer(line))
    for marker_match in re.finditer(re.escape(marker), line):
        if not any(
            span.start() <= marker_match.start() < span.end()
            for span in code_spans
        ):
            return True
    return False


def _scan_stale_syntax(
    path: Path,
    repo_root: Path,
    line_number: int,
    line: str,
    explicitly_rejected: bool,
    failures: list[str],
) -> None:
    if explicitly_rejected:
        return
    for label, pattern in STALE_PATTERNS.items():
        if pattern.search(line):
            display = path.relative_to(repo_root)
            failures.append(f"{display}:{line_number}: accepted {label} syntax")


def _scan_panic_surface(
    path: Path,
    repo_root: Path,
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
    display = path.relative_to(repo_root)
    failures.append(
        f"{display}:{decorator_line_number}: Rust interop example lacks RustPanicError or panic policy"
    )


def _run_self_test() -> int:
    cases = (
        (
            "canonical accepted Sifr",
            "```sifr\n@rust(bridge.hash, panic=trusted_no_panic)\ndef hash(data: bytes) -> bytes: ...\n```\n",
            (),
        ),
        (
            "exact rejected fence",
            "```sifr-rejected\nextern rust crate\nfrom rust import hash\ndlopen(\"libhash\")\n"
            "@rust(crate=hash, path=hash.bytes)\nnative = [\"hash\"]\n```\n",
            (),
        ),
        (
            "same-line inline marker",
            "No `extern rust` lane. <!-- rust-interop-rejected -->\n"
            "No runtime `dlopen`. <!-- rust-interop-rejected -->\n",
            (),
        ),
        (
            "marker spelling inside inline code is not active",
            "The marker is `<!-- rust-interop-rejected -->`, but `extern rust` is stale.\n",
            ("accepted extern rust syntax",),
        ),
        (
            "nearby lexical words",
            "No stale or rejected syntax: `extern rust` is not accepted.\n",
            ("accepted extern rust syntax",),
        ),
        (
            "adjacent rejection prose",
            "The next example is rejected and must not compile.\n\n"
            "```sifr\nextern rust crate\n```\n",
            ("accepted extern rust syntax",),
        ),
        (
            "marker on another line",
            "`extern rust` is abandoned.\n<!-- rust-interop-rejected -->\n",
            ("accepted extern rust syntax",),
        ),
        (
            "malformed rejected fence",
            "```sifr-rejected title\nextern rust crate\n```\n",
            ("malformed rejected fence", "accepted extern rust syntax"),
        ),
        (
            "wrong tick count rejected fence",
            "````sifr-rejected\nextern rust crate\n````\n",
            ("malformed rejected fence", "accepted extern rust syntax"),
        ),
        (
            "unclosed rejected fence",
            "```sifr-rejected\nextern rust crate\n",
            ("unclosed sifr-rejected fence",),
        ),
        (
            "nested opener does not reject accepted fence",
            "```sifr\n```sifr-rejected\nextern rust crate\n```\n",
            ("accepted extern rust syntax",),
        ),
        (
            "nested opener does not close rejected fence",
            "```sifr-rejected\n```sifr\nextern rust crate\n```\n",
            (),
        ),
        (
            "python fence remains invalid",
            "```python\n@rust(bridge.hash)  <!-- rust-interop-rejected -->\n"
            "def hash(data: bytes) -> bytes: ...\n```\n",
            ("Sifr Rust interop example uses python fence",),
        ),
        (
            "stale spelling on fence info line",
            "```text extern rust\nplain content\n```\n",
            ("accepted extern rust syntax",),
        ),
        (
            "accepted panic surface remains checked",
            "```sifr\n@rust(bridge.hash)\n"
            "def hash(data: bytes) -> Result[bytes, HashError]: ...\n```\n",
            ("Rust interop example lacks RustPanicError or panic policy",),
        ),
        (
            "rejected panic surface is exempt",
            "```sifr-rejected\n@rust(bridge.hash)\n"
            "def hash(data: bytes) -> Result[bytes, HashError]: ...\n```\n",
            (),
        ),
    )
    with tempfile.TemporaryDirectory(prefix="sifr-rust-interop-stale-drafts-") as temp_dir:
        root = Path(temp_dir)
        for name, content, expected in cases:
            case_root = root / name.replace(" ", "-")
            path = case_root / "docs" / "case.md"
            path.parent.mkdir(parents=True)
            path.write_text(content, encoding="utf-8")
            failures = _scan_repository(case_root)
            if len(failures) != len(expected) or any(
                fragment not in failure for fragment, failure in zip(expected, failures, strict=True)
            ):
                print(
                    f"self-test {name!r} expected {expected!r}, got {failures!r}",
                    file=sys.stderr,
                )
                return 1

        excluded_root = root / "scan-exclusions"
        for relative in ("plans/archive/stale.md", "plans/reviews/stale.mdx"):
            path = excluded_root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("extern rust crate\n", encoding="utf-8")
        if failures := _scan_repository(excluded_root):
            print(f"self-test scan exclusions unexpectedly failed: {failures}", file=sys.stderr)
            return 1

        mdx_cases = (
            (
                "mdx marker",
                "No `extern rust` lane. {/* rust-interop-rejected */}\n",
                (),
            ),
            (
                "html marker is not valid in mdx",
                "No `extern rust` lane. <!-- rust-interop-rejected -->\n",
                ("accepted extern rust syntax",),
            ),
        )
        for name, content, expected in mdx_cases:
            case_root = root / name.replace(" ", "-")
            path = case_root / "docs" / "case.mdx"
            path.parent.mkdir(parents=True)
            path.write_text(content, encoding="utf-8")
            failures = _scan_repository(case_root)
            if len(failures) != len(expected) or any(
                fragment not in failure for fragment, failure in zip(expected, failures, strict=True)
            ):
                print(
                    f"self-test {name!r} expected {expected!r}, got {failures!r}",
                    file=sys.stderr,
                )
                return 1

        required_root = root / "required-scan-roots"
        (required_root / "docs").mkdir(parents=True)
        (required_root / "docs" / "case.md").write_text(
            "accepted prose\n",
            encoding="utf-8",
        )
        required_failures = _scan_repository(required_root, require_all_roots=True)
        expected_required = ["missing scan root: internal_docs", "missing scan root: plans"]
        if required_failures != expected_required:
            print(
                "self-test required scan roots expected "
                f"{expected_required!r}, got {required_failures!r}",
                file=sys.stderr,
            )
            return 1

    case_count = len(cases) + len(mdx_cases) + 2
    print(f"rust interop stale draft self-test ok: cases={case_count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
