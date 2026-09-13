#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

python3 - "${REPO_ROOT}" <<'PY'
from pathlib import Path
import re
import sys


root = Path(sys.argv[1])
extension = root / "editor_integrations/vscode"
sys.path.insert(0, str(root / "scripts"))
from check_node_toolchain import ToolchainError, validate

try:
    node_version, npm_version = validate(extension)
except ToolchainError as error:
    raise SystemExit(str(error)) from error

workflow_contracts = {
    extension / ".github/workflows/ci.yml": (
        "node-version-file: .node-version",
        "npm ci --ignore-scripts --include=dev",
        'bash scripts/setup-npm.sh "${RUNNER_TEMP}/sifr-npm"',
    ),
    root / ".github/workflows/release-qualification.yml": (
        "node-version-file: editor_integrations/vscode/.node-version",
        "npm ci --ignore-scripts --include=dev --prefix editor_integrations/vscode",
        'bash editor_integrations/vscode/scripts/setup-npm.sh "${RUNNER_TEMP}/sifr-npm"',
        "python3 scripts/check_node_toolchain.py",
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
    if text.index("setup-npm.sh") > text.index("npm ci --ignore-scripts"):
        raise SystemExit(f"{path}: exact npm must be provisioned before npm ci")
    if 'echo "${npm_bin}" >> "${GITHUB_PATH}"' not in text:
        raise SystemExit(f"{path}: private npm bin must be exported to later steps")

publication = (root / ".github/workflows/release-publication.yml").read_text(encoding="utf-8")
if re.search(r"^\s*node-version:", publication, flags=re.MULTILINE):
    raise SystemExit("publication: inline Node selector remains")
if publication.count("node-version-file:") != 1:
    raise SystemExit("publication: expected one canonical Node version-file selection")
selection = "node-version-file: stable-source/editor_integrations/vscode/.node-version"
provision = "bash stable-source/scripts/distribution/provision_marketplace_toolchain.sh"
if selection not in publication or provision not in publication:
    raise SystemExit("publication must select and provision the candidate editor toolchain")
if publication.index(selection) > publication.index(provision):
    raise SystemExit("publication must select Node before provisioning npm")
helper = (root / "scripts/distribution/provision_marketplace_toolchain.sh").read_text(encoding="utf-8")
steps = (
    'scripts/setup-npm.sh" "${RUNNER_TEMP:?}/sifr-npm"',
    'export PATH="${npm_bin}:${PATH}"',
    '"${npm_bin}" >> "${GITHUB_PATH:?}"',
    'python3 "${repo_root}/scripts/check_node_toolchain.py"',
    'npm ci --ignore-scripts --include=dev --prefix "${repo_root}/editor_integrations/vscode"',
)
for step in steps:
    if step not in helper:
        raise SystemExit(f"publication toolchain helper missing {step}")
positions = [helper.index(step) for step in steps]
if positions != sorted(positions):
    raise SystemExit("publication must provision/check exact npm before dependency installation")

demo = (root / "demos/editor_candidate_qualification/run.sh").read_text(encoding="utf-8")
preflight = 'python3 "${repo_root}/scripts/check_node_toolchain.py"'
if preflight not in demo or demo.index(preflight) > demo.index("build_release_artifacts.sh"):
    raise SystemExit("editor demo must check Node/npm before native candidate work")

print(f"Node toolchain contract: PASS (Node {node_version}, npm {npm_version})")
PY
