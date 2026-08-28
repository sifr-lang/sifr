#!/usr/bin/env python3
"""Require local safety contracts and narrowly scoped unsafe lint allowances."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys
import tempfile

from rust_source_policy import (
    is_test_path,
    mask_rust_literals,
    mask_rust_non_code,
    strip_cfg_test_modules,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_ROOT = Path("crates/sifr_runtime/src/python")
PYTHON_MODULE = Path("crates/sifr_runtime/src/python.rs")
FILE_ALLOWLIST = {
    Path("crates/sifr_runtime/src/python/arrow_ops/abi.rs"),
    Path("crates/sifr_runtime/src/python/buffer_ops/access.rs"),
    Path("crates/sifr_runtime/src/python/buffer_ops/raw.rs"),
    Path("crates/sifr_runtime/src/python/callbacks/current.rs"),
    Path("crates/sifr_runtime/src/python/dlpack_ops/abi.rs"),
    Path("crates/sifr_runtime/src/python/dlpack_ops/argument.rs"),
}
UNSAFE_OPERATION = re.compile(r"\bunsafe\s*(?:\{|impl\b|fn\b|extern\b)")
UNSAFE_TYPE_ALIAS = re.compile(r"(?ms)^\s*type\s+\w+\s*=\s*unsafe\s+extern\b.*?;")
ITEM_UNSAFE_ALLOW = re.compile(r"(?m)^\s*#\[allow\(unsafe_code\)\]\s*$")
NARROW_ITEM_AFTER_ALLOW = re.compile(
    r"^\s*(?:(?:pub(?:\([^)]*\))?|async|const|extern\s+\"[^\"]+\"|unsafe)\s+)*fn\b"
    r"|^\s*unsafe\s+impl\b"
)


def production_text(path: Path) -> str:
    return strip_cfg_test_modules(path.read_text(encoding="utf-8"))


def has_contract(lines: list[str], line_number: int) -> bool:
    context = "\n".join(lines[max(0, line_number - 9) : line_number - 1])
    return "SAFETY:" in context or "# Safety" in context


def validate(root: Path) -> list[str]:
    errors: list[str] = []
    module_path = root / PYTHON_MODULE
    if module_path.exists() and "#![allow(unsafe_code)]" in module_path.read_text(encoding="utf-8"):
        errors.append(f"blanket unsafe allowance is forbidden in {PYTHON_MODULE.as_posix()}")
    files = [module_path]
    python_root = root / PYTHON_ROOT
    if python_root.exists():
        files.extend(sorted(python_root.rglob("*.rs")))
    seen_file_allows: set[Path] = set()
    operation_count = 0
    for path in files:
        if not path.exists():
            continue
        relative = path.relative_to(root)
        if is_test_path(relative):
            continue
        text = production_text(path)
        code = mask_rust_non_code(text)
        lines = mask_rust_literals(text).splitlines()
        type_alias_spans = [match.span() for match in UNSAFE_TYPE_ALIAS.finditer(code)]
        if "#![allow(unsafe_code)]" in text:
            seen_file_allows.add(relative)
            if relative not in FILE_ALLOWLIST:
                errors.append(f"unapproved file-level unsafe allowance: {relative.as_posix()}")
            if not text.startswith("#![allow(unsafe_code)]\n"):
                errors.append(f"misplaced file-level unsafe allowance: {relative.as_posix()}")
        for allow_match in ITEM_UNSAFE_ALLOW.finditer(code):
            following = code[allow_match.end() :].lstrip()
            if not NARROW_ITEM_AFTER_ALLOW.match(following):
                line_number = code.count("\n", 0, allow_match.start()) + 1
                errors.append(
                    "unsafe item allowance must own one function or unsafe impl: "
                    f"{relative.as_posix()}:{line_number}"
                )
        for match in UNSAFE_OPERATION.finditer(code):
            if any(start <= match.start() < end for start, end in type_alias_spans):
                continue
            line_number = code.count("\n", 0, match.start()) + 1
            line = lines[line_number - 1]
            if "Option<unsafe extern" in line:
                continue
            operation_count += 1
            if not has_contract(lines, line_number):
                errors.append(
                    f"unsafe operation lacks local SAFETY contract: {relative.as_posix()}:{line_number}"
                )
    stale_allows = sorted(FILE_ALLOWLIST - seen_file_allows)
    for path in stale_allows:
        if (root / path).exists():
            errors.append(f"stale unsafe file allowance inventory: {path.as_posix()}")
    if operation_count == 0:
        errors.append("unsafe ABI contract scan found no production operations")
    return errors


def run_self_test() -> int:
    with tempfile.TemporaryDirectory(prefix="unsafe-abi-") as tmp:
        root = Path(tmp)
        module = root / PYTHON_MODULE
        module.parent.mkdir(parents=True)
        module.write_text(
            "const QUOTE: char = '\"';\nconst BYTE_QUOTE: u8 = b'\"';\n"
            "#[allow(unsafe_code)]\nfn probe() {\n    // SAFETY: pointer is non-null.\n    unsafe {}\n}\n",
            encoding="utf-8",
        )
        if validate(root):
            print("unsafe ABI self-test baseline failed", file=sys.stderr)
            return 1
        module.write_text("#![allow(unsafe_code)]\nfn probe() { unsafe {} }\n", encoding="utf-8")
        if not any("blanket" in error for error in validate(root)):
            print("unsafe ABI self-test missed blanket allowance", file=sys.stderr)
            return 1
        module.write_text("#[allow(unsafe_code)]\nfn probe() { unsafe {} }\n", encoding="utf-8")
        if not any("lacks local" in error for error in validate(root)):
            print("unsafe ABI self-test missed absent contract", file=sys.stderr)
            return 1
        module.write_text(
            '#[allow(unsafe_code)]\nmod widened { fn probe() { unsafe {} } }\n',
            encoding="utf-8",
        )
        if not any("one function or unsafe impl" in error for error in validate(root)):
            print("unsafe ABI self-test accepted a widened item allowance", file=sys.stderr)
            return 1
        module.write_text(
            '#[allow(unsafe_code)]\nfn probe() {\n'
            '    const FAKE: &str = "// SAFETY: not a contract";\n'
            '    unsafe {}\n}\n',
            encoding="utf-8",
        )
        if not any("lacks local" in error for error in validate(root)):
            print("unsafe ABI self-test accepted a literal safety contract", file=sys.stderr)
            return 1
    print("unsafe ABI contract self-test: PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()
    errors = validate(REPO_ROOT)
    if errors:
        for error in errors:
            print(f"unsafe ABI contract error: {error}", file=sys.stderr)
        return 1
    print("unsafe ABI contracts: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
