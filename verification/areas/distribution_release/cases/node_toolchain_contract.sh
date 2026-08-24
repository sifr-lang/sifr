#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

python3 - "${REPO_ROOT}" <<'PY'
import json
from pathlib import Path
import re
import sys


root = Path(sys.argv[1])
extension = root / "editor_integrations/vscode"
node_version = (extension / ".node-version").read_text(encoding="utf-8").strip()
if re.fullmatch(r"24\.[0-9]+\.[0-9]+", node_version) is None:
    raise SystemExit("extension .node-version must select one exact Node 24 release")

package = json.loads((extension / "package.json").read_text(encoding="utf-8"))
package_manager = package.get("packageManager")
match = re.fullmatch(r"npm@([0-9]+\.[0-9]+\.[0-9]+)", package_manager or "")
if match is None:
    raise SystemExit("extension packageManager must select one exact npm release")
npm_version = match.group(1)

expected_engines = {
    "node": node_version,
    "npm": npm_version,
    "vscode": package.get("engines", {}).get("vscode"),
}
if package.get("engines") != expected_engines:
    raise SystemExit("extension engines must agree with exact Node and npm selectors")

expected_dev_engines = {
    "runtime": {
        "name": "node",
        "version": node_version,
        "onFail": "error",
    },
    "packageManager": {
        "name": "npm",
        "version": npm_version,
        "onFail": "error",
    },
}
if package.get("devEngines") != expected_dev_engines:
    raise SystemExit("npm devEngines must reject Node or npm selector drift")

lock = json.loads((extension / "package-lock.json").read_text(encoding="utf-8"))
if lock.get("lockfileVersion") != 3:
    raise SystemExit("extension lock must use npm lockfile version 3")
if lock.get("packages", {}).get("", {}).get("engines") != expected_engines:
    raise SystemExit("extension lock root engines drifted from package.json")

workflow_contracts = {
    extension / ".github/workflows/ci.yml": (
        "node-version-file: .node-version",
        "npm ci --ignore-scripts --include=dev",
    ),
    root / ".github/workflows/release-qualification.yml": (
        "node-version-file: editor_integrations/vscode/.node-version",
        "npm ci --ignore-scripts --include=dev --prefix editor_integrations/vscode",
    ),
    root / ".github/workflows/release-publication.yml": (
        "node-version-file: stable-source/editor_integrations/vscode/.node-version",
        "npm ci --ignore-scripts --include=dev --prefix stable-source/editor_integrations/vscode",
    ),
}
for path, required in workflow_contracts.items():
    text = path.read_text(encoding="utf-8")
    if re.search(r"^\s*node-version:", text, flags=re.MULTILINE):
        raise SystemExit(f"{path}: legacy inline Node selector remains")
    if text.count("node-version-file:") != 1:
        raise SystemExit(f"{path}: expected one canonical Node version-file selection")
    for fragment in required:
        if fragment not in text:
            raise SystemExit(f"{path}: missing Node toolchain contract: {fragment}")

print(f"Node toolchain contract: PASS (Node {node_version}, npm {npm_version})")
PY
