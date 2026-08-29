#!/usr/bin/env python3
"""Check or update every generated demo companion against current Sifr emission."""

from __future__ import annotations

import argparse
import concurrent.futures
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT / "verification" / "areas" / "common"))

from sifr_binary import resolve_sifr_binary  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sifr", type=Path)
    parser.add_argument("--update", action="store_true")
    parser.add_argument("--jobs", type=int, default=8)
    args = parser.parse_args()

    if args.sifr is not None:
        compiler = args.sifr.resolve()
        if not compiler.is_file():
            parser.error(f"Sifr compiler does not exist: {compiler}")
    else:
        compiler = resolve_sifr_binary(
            REPO_ROOT,
            explicit_env_var="SIFR_GCQ_BIN",
            default_binary=REPO_ROOT / "target/debug/sifr",
        ).resolve()

    if args.jobs < 1:
        parser.error("--jobs must be positive")

    pairs: list[tuple[Path, Path]] = []
    for emitted in sorted((REPO_ROOT / "demos").glob("**/emitted.rs")):
        source = emitted.with_name("main.sifr")
        if source.is_file():
            pairs.append((emitted, source))

    def emit(pair: tuple[Path, Path]) -> tuple[Path, subprocess.CompletedProcess[bytes]]:
        emitted, source = pair
        result = subprocess.run(
            [str(compiler), "emit", str(source)],
            cwd=REPO_ROOT,
            check=False,
            capture_output=True,
        )
        return emitted, result

    stale: list[Path] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as executor:
        results = list(executor.map(emit, pairs))
    for (emitted, source), (result_path, result) in zip(pairs, results, strict=True):
        assert emitted == result_path
        if result.returncode != 0:
            sys.stderr.buffer.write(result.stderr)
            print(f"failed to emit {source.relative_to(REPO_ROOT)}", file=sys.stderr)
            return result.returncode
        if emitted.read_bytes() == result.stdout:
            continue
        stale.append(emitted)
        if args.update:
            emitted.write_bytes(result.stdout)

    if not stale:
        print("all generated demo companions are fresh")
        return 0
    action = "updated" if args.update else "stale"
    for path in stale:
        print(f"{action}: {path.relative_to(REPO_ROOT)}")
    return 0 if args.update else 1


if __name__ == "__main__":
    raise SystemExit(main())
