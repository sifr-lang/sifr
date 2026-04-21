#!/usr/bin/env python3
from __future__ import annotations

import argparse
import difflib
import json
from dataclasses import asdict, dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
LEETCODE_DIR = REPO_ROOT / "audits" / "leetcode"
HELPER_PY_FILES = {"run_audit.py", "convert_all.py"}


@dataclass
class PairDiff:
    stem: str
    py_relpath: str
    sifr_relpath: str
    py_lines: int
    sifr_lines: int
    changed_py_lines: int
    changed_sifr_lines: int
    changed_total_lines: int
    length_delta: int
    similarity_ratio: float


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Rank LeetCode audit pairs by line-level diff size."
    )
    parser.add_argument(
        "--top",
        type=int,
        default=25,
        help="How many ranked pairs to print.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Optional JSON output path.",
    )
    return parser.parse_args()


def collect_pairs() -> tuple[list[PairDiff], list[str], list[str]]:
    py_files = {
        path.stem: path
        for path in LEETCODE_DIR.glob("*.py")
        if path.name not in HELPER_PY_FILES
    }
    sifr_files = {path.stem: path for path in LEETCODE_DIR.glob("*.sifr")}

    paired_stems = sorted(py_files.keys() & sifr_files.keys())
    py_only = sorted(py_files.keys() - sifr_files.keys())
    sifr_only = sorted(sifr_files.keys() - py_files.keys())

    pair_diffs: list[PairDiff] = []
    for stem in paired_stems:
        py_path = py_files[stem]
        sifr_path = sifr_files[stem]
        py_lines = py_path.read_text().splitlines()
        sifr_lines = sifr_path.read_text().splitlines()
        matcher = difflib.SequenceMatcher(a=py_lines, b=sifr_lines)

        changed_py_lines = 0
        changed_sifr_lines = 0
        for tag, i1, i2, j1, j2 in matcher.get_opcodes():
            if tag == "equal":
                continue
            changed_py_lines += i2 - i1
            changed_sifr_lines += j2 - j1

        pair_diffs.append(
            PairDiff(
                stem=stem,
                py_relpath=str(py_path.relative_to(REPO_ROOT)),
                sifr_relpath=str(sifr_path.relative_to(REPO_ROOT)),
                py_lines=len(py_lines),
                sifr_lines=len(sifr_lines),
                changed_py_lines=changed_py_lines,
                changed_sifr_lines=changed_sifr_lines,
                changed_total_lines=changed_py_lines + changed_sifr_lines,
                length_delta=abs(len(py_lines) - len(sifr_lines)),
                similarity_ratio=matcher.ratio(),
            )
        )

    pair_diffs.sort(
        key=lambda item: (
            -item.changed_total_lines,
            item.similarity_ratio,
            item.stem,
        )
    )
    return pair_diffs, py_only, sifr_only


def main() -> int:
    args = parse_args()
    pair_diffs, py_only, sifr_only = collect_pairs()

    print(
        f"paired={len(pair_diffs)} py_only={len(py_only)} sifr_only={len(sifr_only)}"
    )
    print("top ranked by changed_total_lines")
    for index, item in enumerate(pair_diffs[: args.top], start=1):
        print(
            f"{index:>2}. {item.stem} "
            f"changed_total={item.changed_total_lines} "
            f"changed_py={item.changed_py_lines} "
            f"changed_sifr={item.changed_sifr_lines} "
            f"ratio={item.similarity_ratio:.3f} "
            f"lines={item.py_lines}/{item.sifr_lines}"
        )

    if py_only:
        print(f"py_only: {', '.join(py_only[:10])}")
    if sifr_only:
        print(f"sifr_only: {', '.join(sifr_only[:10])}")

    if args.output:
        payload = {
            "name": "leetcode_pair_diff_scan",
            "root": str(LEETCODE_DIR.relative_to(REPO_ROOT)),
            "summary": {
                "paired_cases": len(pair_diffs),
                "py_only_cases": len(py_only),
                "sifr_only_cases": len(sifr_only),
            },
            "py_only": py_only,
            "sifr_only": sifr_only,
            "pairs": [asdict(item) for item in pair_diffs],
        }
        args.output.write_text(json.dumps(payload, indent=2) + "\n")
        print(f"wrote {args.output}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
