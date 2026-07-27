#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

workflow="${REPO_ROOT}/.github/workflows/release-qualification.yml"

ruby - "${workflow}" <<'RUBY'
require "yaml"

workflow = YAML.load_file(ARGV.fetch(0))
unless workflow.fetch("permissions") == {"contents" => "read", "actions" => "read"}
  abort "release qualification permissions must be contents:read and actions:read only"
end
triggers = workflow["on"] || workflow.fetch(true)
inputs = triggers.fetch("workflow_dispatch").fetch("inputs")
unless inputs.keys.sort == ["source_commit", "version"]
  abort "release qualification accepts only exact source_commit and version inputs"
end
jobs = workflow.fetch("jobs")
unless jobs.keys.sort == ["assemble", "build", "collect", "editor", "validate"]
  abort "release qualification job topology drifted"
end
matrix = jobs.fetch("build").fetch("strategy").fetch("matrix").fetch("include")
expected = {
  "aarch64-apple-darwin" => "macos-15",
  "x86_64-apple-darwin" => "macos-15-intel",
  "x86_64-unknown-linux-gnu" => "ubuntu-24.04",
  "aarch64-unknown-linux-gnu" => "ubuntu-24.04-arm",
}
actual = matrix.to_h { |row| [row.fetch("target"), row.fetch("runner")] }
abort "release qualification target/runner matrix drifted" unless actual == expected

uploads = jobs.values.flat_map { |job| job.fetch("steps", []) }.select {
  |step| step["uses"] == "actions/upload-artifact@v4"
}
abort "release qualification upload step count drifted" unless uploads.length == 4
uploads.each do |upload|
  config = upload.fetch("with")
  abort "qualification artifact retention must be 30 days" unless config["retention-days"] == 30
  abort "qualification artifacts must forbid overwrite" unless config["overwrite"] == false
  unless config.fetch("name").start_with?("sifr-stable-candidate-")
    abort "qualification artifact name lost its governed prefix"
  end
end
abort "qualification jobs must not bind a mutation environment" if jobs.values.any? {
  |job| job.key?("environment")
}
RUBY

python3 - "${workflow}" "${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" <<'PY'
from pathlib import Path
import sys

text = Path(sys.argv[1]).read_text(encoding="utf-8")
builder = Path(sys.argv[2]).read_text(encoding="utf-8")
required = (
    "[[ \"${SOURCE_COMMIT}\" =~ ^[0-9a-f]{40}$ ]]",
    "[[ \"${VERSION}\" =~ ^[0-9]+\\.[0-9]+\\.[0-9]+$ ]]",
    "[[ \"${WORKFLOW_COMMIT}\" = \"${SOURCE_COMMIT}\" ]]",
    "contents: read",
    "actions: read",
    "submodules: recursive",
    "scripts/distribution/build_release_artifacts.sh",
    "--cargo-build",
    "scripts/distribution/qualify_stable_target.py",
    "scripts/distribution/generate_version_installer.sh",
    "scripts/distribution/collect_qualification_artifacts.py",
    "Verify immutable qualification workflow contract",
    "--run-id \"${GITHUB_RUN_ID}\"",
    "--run-metadata run-metadata.json",
    "Artifact ID: \\`${ARTIFACT_ID}\\`",
)
for fragment in required:
    if fragment not in text:
        raise SystemExit(f"release qualification omitted governed fragment: {fragment}")
installer_invocation = """scripts/distribution/generate_version_installer.sh \\
            --version "${VERSION}" \\
            --artifact-dir target-artifacts \\
            --out "qualification-assemble/sifr-installer-${VERSION}"""
if installer_invocation not in text:
    raise SystemExit(
        "release qualification installer invocation must match planner regeneration"
    )
for forbidden in (
    "contents: write",
    "packages: write",
    "deployments: write",
    "gh release ",
    "vsce publish",
    "repository_dispatch",
):
    if forbidden in text:
        raise SystemExit(f"release qualification contains mutation capability: {forbidden}")
if text.count("overwrite: false") != 4 or text.count("retention-days: 30") != 4:
    raise SystemExit("every qualification upload must be immutable with 30-day retention")
if "cargo build --locked --release -p sifr" not in builder:
    raise SystemExit("governed release artifact builder must use Cargo.lock")
for target in (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
):
    exact_name = (
        "name: sifr-stable-candidate-${{ needs.validate.outputs.version }}-"
        "${{ needs.validate.outputs.source_commit }}-" + target
    )
    if text.count(exact_name) != 1:
        raise SystemExit(f"target {target} must be downloaded by exact artifact name")
PY

echo "release qualification workflow contract: PASS"
