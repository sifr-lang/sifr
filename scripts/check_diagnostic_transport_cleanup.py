#!/usr/bin/env python3
"""Check removal of retired diagnostic transport symbols."""

from __future__ import annotations

import pathlib
import re
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
RETIRED_SYMBOLS = (
    re.compile(r"\bLoweringError\b"),
    re.compile(r"\bTypeErrorKind\b"),
    re.compile(r"\bsifr_type_system::TypeError\b"),
    re.compile(r"\bis_message_error_code\b"),
    re.compile(r"\bdiagnostic_error_code\b"),
)
RAW_HIR_ERROR_FREE_FILES = (
    pathlib.Path("crates/sifr_lowering/src/lower/bytes_methods.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/builtin_calls.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/aug_assign_lowering.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/classes.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/container_literal_specialization.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/decimal_methods.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/diagnostic_transport_tests.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/method_call_args.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/min_max_validation.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/mod.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/module_function_registry.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/nested_function_inference.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/statements.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/subscript_type.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/tuple_unpack.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/typing_and_functions.rs"),
    pathlib.Path("crates/sifr_lowering/src/lower/type_aliases.rs"),
)
RAW_CTX_ERROR = re.compile(r"\bctx\.error\s*\(")


def git_ls_files(*patterns: str) -> list[pathlib.Path]:
    result = subprocess.run(
        ["git", "ls-files", *patterns],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return [ROOT / line for line in result.stdout.splitlines() if line]


def main() -> int:
    errors: list[str] = []
    for path in git_ls_files("crates/**/*.rs"):
        if not path.exists():
            continue
        rel = path.relative_to(ROOT)
        text = path.read_text(encoding="utf-8", errors="ignore")
        for line_number, line in enumerate(text.splitlines(), 1):
            for pattern in RETIRED_SYMBOLS:
                if pattern.search(line):
                    errors.append(f"{rel}:{line_number}: retired diagnostic transport symbol")
            if rel in RAW_HIR_ERROR_FREE_FILES and RAW_CTX_ERROR.search(line):
                errors.append(f"{rel}:{line_number}: raw HIR ctx.error in migrated file")

    if errors:
        for error in errors:
            print(f"diagnostic transport cleanup: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
