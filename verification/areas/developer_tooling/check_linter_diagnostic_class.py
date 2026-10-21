#!/usr/bin/env python3
"""Validate typed diagnostic-class gating for lint code actions."""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCAN_ROOTS = [Path("crates/sifr_analysis/src"), Path("crates/sifr_lsp/src")]
ACTION_OWNER = Path("crates/sifr_analysis/src/host/code_actions.rs")
CONVERSION_OWNER = Path("crates/sifr_lsp/src/conversion.rs")
FORBIDDEN = [
    'starts_with("SIFR-LINT-")',
    "starts_with('SIFR-LINT-')",
]
REQUIRED_OWNERS = {
    ACTION_OWNER: ["DiagnosticClass::Policy"],
    CONVERSION_OWNER: ['"diagnosticClass"', '"policy"'],
}


def rust_files(root: Path = REPO_ROOT) -> list[Path]:
    return sorted(path for relative in SCAN_ROOTS for path in (root / relative).rglob("*.rs"))


def violations(paths: list[Path], *, root: Path = REPO_ROOT) -> list[str]:
    failures: list[str] = []
    texts = {path: path.read_text(encoding="utf-8", errors="replace") for path in paths}
    for path, text in texts.items():
        for line_number, line in enumerate(text.splitlines(), 1):
            for pattern in FORBIDDEN:
                if pattern in line:
                    failures.append(
                        f"{path.relative_to(root)}:{line_number} uses prefix diagnostic gating {pattern!r}"
                    )
    for relative, required in REQUIRED_OWNERS.items():
        text = texts.get(root / relative)
        if text is None:
            failures.append(f"{relative}: required diagnostic-class source owner is missing")
            continue
        missing = [snippet for snippet in required if snippet not in text]
        if missing:
            failures.append(f"{relative}: missing diagnostic-class contract snippets: {missing}")
    return failures


def run_self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        action = root / ACTION_OWNER
        conversion = root / CONVERSION_OWNER
        for path in (action, conversion):
            path.parent.mkdir(parents=True, exist_ok=True)
        action_text = "fn gate() { diagnostic.class == DiagnosticClass::Policy }\n"
        conversion_text = '\n'.join(REQUIRED_OWNERS[CONVERSION_OWNER])
        action.write_text(action_text, encoding="utf-8")
        conversion.write_text(conversion_text, encoding="utf-8")
        assert not violations(rust_files(root), root=root), "valid current layout rejected"

        def reject(expected: str) -> None:
            found = violations(rust_files(root), root=root)
            assert any(expected in failure for failure in found), (expected, found)

        for relative in SCAN_ROOTS:
            seed = root / relative / "prefix_gate.rs"
            seed.write_text('fn gate(code: &str) -> bool { code.starts_with("SIFR-LINT-") }\n')
            reject("uses prefix diagnostic gating")
            seed.unlink()

        former_owner = action.with_name("implementation.rs")
        former_owner.write_text(action_text, encoding="utf-8")
        action.write_text(action_text.replace("DiagnosticClass::Policy", "DiagnosticClass::Hard"))
        reject(f"{ACTION_OWNER}: missing diagnostic-class contract snippets")
        action.unlink()
        reject(f"{ACTION_OWNER}: required diagnostic-class source owner is missing")
        former_owner.unlink()
        action.write_text(action_text, encoding="utf-8")

        for snippet in REQUIRED_OWNERS[CONVERSION_OWNER]:
            conversion.write_text(conversion_text.replace(snippet, ""), encoding="utf-8")
            reject(f"{CONVERSION_OWNER}: missing diagnostic-class contract snippets")
        conversion.unlink()
        reject(f"{CONVERSION_OWNER}: required diagnostic-class source owner is missing")
        conversion.write_text(conversion_text, encoding="utf-8")
        assert not violations(rust_files(root), root=root), "restored valid layout rejected"
    print("linter diagnostic-class self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0
    found = violations(rust_files())
    if found:
        print("linter diagnostic-class: FAIL", file=sys.stderr)
        for failure in found:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("linter diagnostic-class: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
