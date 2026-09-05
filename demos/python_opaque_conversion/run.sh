#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AREA_ROOT="${REPO_ROOT}/verification/areas/python_interop"
RUNNER_ROOT="${AREA_ROOT}/runner"

# Run the biip/schwifty conversion and opaque-lifecycle example.
uv run --project "${AREA_ROOT}" --locked python - "${RUNNER_ROOT}" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path

runner_root = Path(sys.argv[1])
sys.path.insert(0, str(runner_root))

from env import discover_paths
from example_packages import _run_case, prepare_example_package
from library_examples import LIBRARY_EXAMPLE_CASES

paths = discover_paths()
case = LIBRARY_EXAMPLE_CASES["biip-schwifty"]
package_root = prepare_example_package(paths, "library", case)
result = _run_case(paths, package_root, case)

if result["status"] != "example-passed":
    print(result.get("stdout", ""), end="")
    print(result.get("stderr", ""), file=sys.stderr, end="")
    raise SystemExit("Python biip/schwifty demo failed")

print(result["stdout"], end="")
PY
