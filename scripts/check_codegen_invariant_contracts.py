#!/usr/bin/env python3
"""Require a local contract for each retained codegen panic invariant."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys
import tempfile

from rust_source_policy import is_test_path, mask_rust_non_code, strip_cfg_test_modules


REPO_ROOT = Path(__file__).resolve().parents[1]
CODEGEN_ROOT = Path("crates/sifr_codegen/src")
INVARIANT_MACRO = re.compile(
    r"\b(?:debug_assert_eq|debug_assert_ne|debug_assert|assert_eq|assert_ne|assert|panic|unreachable)!\s*\("
)


def has_contract(lines: list[str], line_number: int) -> bool:
    context = "\n".join(lines[max(0, line_number - 5) : line_number - 1])
    return "INVARIANT:" in context


def validate(root: Path) -> tuple[list[str], int]:
    errors: list[str] = []
    operation_count = 0
    source_root = root / CODEGEN_ROOT
    if not source_root.exists():
        return [f"missing codegen source root: {CODEGEN_ROOT.as_posix()}"], 0
    for path in sorted(source_root.rglob("*.rs")):
        relative = path.relative_to(root)
        if is_test_path(relative):
            continue
        production = strip_cfg_test_modules(path.read_text(encoding="utf-8"))
        code = mask_rust_non_code(production)
        lines = production.splitlines()
        for match in INVARIANT_MACRO.finditer(code):
            operation_count += 1
            line_number = code.count("\n", 0, match.start()) + 1
            if not has_contract(lines, line_number):
                errors.append(
                    "codegen panic invariant lacks local INVARIANT contract: "
                    f"{relative.as_posix()}:{line_number}"
                )
    if operation_count == 0:
        errors.append("codegen invariant scan found no production operations")
    return errors, operation_count


def run_self_test() -> int:
    with tempfile.TemporaryDirectory(prefix="codegen-invariant-", dir=REPO_ROOT / "target") as tmp:
        root = Path(tmp)
        source = root / CODEGEN_ROOT / "probe.rs"
        source.parent.mkdir(parents=True)
        source.write_text(
            "fn before() {\n    // INVARIANT: lowering proves this branch is impossible.\n"
            '    unreachable!("impossible");\n}\n'
            "#[cfg(test)]\nmod tests { fn ignored() { panic!(); } }\n"
            "const BYTE_QUOTE: u8 = b'\"';\nconst QUOTE: char = '\"';\n"
            "const APOSTROPHE: char = '\\'';\nconst SLASH: char = '\\\\';\n"
            "fn after() {\n    // INVARIANT: validation provides this metadata.\n"
            '    assert!(true);\n}\nconst TEXT: &str = "panic!()";\n',
            encoding="utf-8",
        )
        errors, count = validate(root)
        if errors or count != 2:
            print(f"codegen invariant self-test baseline failed: {errors}, count={count}", file=sys.stderr)
            return 1
        source.write_text(
            "const BYTE_QUOTE: u8 = b'\"';\nfn probe() { panic!(); }\n",
            encoding="utf-8",
        )
        errors, _ = validate(root)
        if not any("lacks local" in error for error in errors):
            print("codegen invariant self-test missed absent contract", file=sys.stderr)
            return 1
    print("codegen invariant contract self-test: PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()
    errors, operation_count = validate(REPO_ROOT)
    if errors:
        for error in errors:
            print(f"codegen invariant contract error: {error}", file=sys.stderr)
        return 1
    print(f"codegen invariant contracts: PASS ({operation_count} classified operations)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
