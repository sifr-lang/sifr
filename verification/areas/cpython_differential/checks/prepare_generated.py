"""Prepare selected generated programs through their normal Sifr run cache."""

from __future__ import annotations

import argparse
from dataclasses import asdict
import json
import sys

import generated_suite as generated


def prepare_suites(suite_names: list[str]) -> list[str]:
    manifest = json.loads(generated.MANIFEST.read_text())
    failures = generated.validate_python_version()
    build_info = generated.build_release_binary(manifest, failures)
    if failures:
        return failures
    # Cache population includes native compilation, not just program execution.
    timeout = int(manifest["release_binary"]["build_timeout_seconds"])
    for suite_name in sorted(set(suite_names)):
        actual_root = generated.ACTUAL_ROOT / suite_name
        for case in manifest["suites"][suite_name]["cases"]:
            _, source = generated.materialize_case(case, build_info, actual_root)
            result = generated.run_command(
                [build_info["binary"], "--sysroot", str(generated.REPO_ROOT), "run", "--release", str(source)],
                timeout,
            )
            (source.parent / "preparation.json").write_text(json.dumps({
                "case": case["id"], "release_binary": build_info,
                "build_timeout_seconds": timeout, "result": asdict(result),
            }, indent=2, sort_keys=True) + "\n")
            generated.compare_runtime(str(case["id"]), "Sifr", result, case, failures)
            print(f"[cpython-generated-preparation] suite={suite_name} case={case['id']} "
                  f"exit={result.exit_code} elapsed_ms={result.duration_ms:.0f}", flush=True)
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", required=True,
                        choices=["generated_broader", "generated_minimized_seeds"])
    failures = prepare_suites(parser.parse_args().suite)
    for failure in failures:
        print(f"cpython generated preparation error: {failure}", file=sys.stderr)
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
