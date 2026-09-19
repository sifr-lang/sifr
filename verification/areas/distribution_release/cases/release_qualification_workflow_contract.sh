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
unless inputs.keys.sort == ["rollback_version", "source_commit", "version"]
  abort "release qualification accepts only governed candidate inputs"
end
rollback = inputs.fetch("rollback_version")
unless rollback.fetch("required") == true && rollback.fetch("default") == "none"
  abort "rollback_version must be required with first-GA default none"
end
jobs = workflow.fetch("jobs")
unless jobs.keys.sort == ["assemble", "build", "collect", "editor", "native-packages", "transition-assemble", "transition-build", "validate"]
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
native = jobs.fetch("native-packages")
unless native.fetch("needs").sort == ["collect", "transition-assemble", "validate"]
  abort "native package evidence must follow the canonical collector"
end
transition = jobs.fetch("transition-build")
unless transition.fetch("needs").sort == ["collect", "validate"]
  abort "transition fixtures must remain outside canonical artifact collection"
end
unless transition.fetch("strategy").fetch("matrix").fetch("include") == matrix
  abort "transition fixture packages require all four native builders"
end
unless jobs.fetch("transition-assemble").fetch("needs").sort == ["transition-build", "validate"]
  abort "fixture installer must consume the complete transition matrix"
end
validation_steps = jobs.fetch("validate").fetch("steps")
unless validation_steps.any? { |step| step["id"] == "package_versions" }
  abort "version compatibility must be admitted before expensive matrix builds"
end
native_matrix = native.fetch("strategy").fetch("matrix").fetch("include")
unless native_matrix == matrix
  abort "native package execution must use all four native target runners"
end
steps = native.fetch("steps")
download = steps.find { |step| step["name"] == "Download exact indexed candidate packages and installer" }
unless download && download.fetch("with").fetch("digest-mismatch") == "error"
  abort "native qualification must reject artifact transport digest mismatch"
end
native_run = steps.find { |step| step["name"] == "Qualify exact native package modes" }
%w[--previous-version --previous-artifacts --previous-installer].each do |flag|
  abort "native update/rollback lost #{flag}" unless native_run.fetch("run").include?(flag)
end
unless native_run && native_run.fetch("run").include?("scripts/distribution/qualify_native_package.py")
  abort "native package execution protocol is required"
end
unless native_run.fetch("env").fetch("SOURCE_COMMIT") == "${{ needs.validate.outputs.source_commit }}"
  abort "native qualification must bind the validated candidate SHA"
end
unless jobs.fetch("editor").fetch("needs").sort == ["build", "validate"]
  abort "editor qualification must consume the exact built candidate"
end
editor_step = jobs.fetch("editor").fetch("steps").find {
  |step| step["name"] == "Build, test, and package VSIX"
}
abort "editor qualification step is missing" unless editor_step
editor_env = editor_step.fetch("env")
unless editor_env["ROLLBACK_VERSION"] == "${{ needs.validate.outputs.rollback_version }}"
  abort "editor qualification must bind the governed rollback_version output"
end

uploads = jobs.values.flat_map { |job| job.fetch("steps", []) }.select {
  |step| step["uses"] == "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a"
}
abort "release qualification upload step count drifted" unless uploads.length == 7
uploads.each do |upload|
  config = upload.fetch("with")
  abort "qualification artifact retention must be 30 days" unless config["retention-days"] == 30
  abort "qualification artifacts must forbid overwrite" unless config["overwrite"] == false
  unless config.fetch("name").start_with?("sifr-stable-candidate-", "sifr-native-package-", "sifr-native-transition-")
    abort "qualification artifact name lost its governed prefix"
  end
end
abort "qualification jobs must not bind a mutation environment" if jobs.values.any? {
  |job| job.key?("environment")
}
RUBY

python3 - "${workflow}" \
  "${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
  "${REPO_ROOT}/scripts/distribution/qualify_stable_target.py" <<'PY'
from pathlib import Path
import sys

text = Path(sys.argv[1]).read_text(encoding="utf-8")
builder = Path(sys.argv[2]).read_text(encoding="utf-8")
target_qualifier = Path(sys.argv[3]).read_text(encoding="utf-8")
required = (
    "[[ \"${SOURCE_COMMIT}\" =~ ^[0-9a-f]{40}$ ]]",
    "[[ \"${VERSION}\" =~ ^[0-9]+\\.[0-9]+\\.[0-9]+$ ]]",
    "[[ \"${ROLLBACK_VERSION}\" = \"none\" ||",
    "rollback_version=${ROLLBACK_VERSION}",
    "[[ \"${WORKFLOW_COMMIT}\" = \"${SOURCE_COMMIT}\" ]]",
    "contents: read",
    "actions: read",
    "submodules: recursive",
    "node-version-file: editor_integrations/vscode/.node-version",
    "npm ci --ignore-scripts --include=dev --prefix editor_integrations/vscode",
    "scripts/distribution/build_release_artifacts.sh",
    "--cargo-build",
    "scripts/distribution/qualify_stable_target.py",
    "scripts/distribution/qualify_stable_editor.py",
    "--candidate-binary \"${candidate_binary}\"",
    "--target-report",
    "--rollback-version \"${ROLLBACK_VERSION}\"",
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
installer_invocation = (
    'scripts/distribution/generate_version_installer.sh \\\n'
    '            --version "${VERSION}" \\\n'
    '            --artifact-dir target-artifacts \\\n'
    '            --out "qualification-assemble/sifr-installer-${VERSION}"\n'
)
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
if text.count("overwrite: false") != 7 or text.count("retention-days: 30") != 7:
    raise SystemExit("every qualification upload must be immutable with 30-day retention")
if "cargo build --locked --release -p sifr" not in builder:
    raise SystemExit("governed release artifact builder must use Cargo.lock")
for fragment in (
    '"emit", str(smoke_source)',
    'for temperature in ("cold", "warm")',
    '"--candidate-smoke"',
    'timeout_seconds=90',
    'timeout_seconds=120',
    'SIFR_RUST_BRIDGE_PROBE_CACHE_DIR',
):
    if fragment not in target_qualifier:
        raise SystemExit(
            f"stable target qualifier omitted generated Rust governance: {fragment}"
        )
download_counts = {
    "aarch64-apple-darwin": 1,
    "x86_64-apple-darwin": 1,
    "aarch64-unknown-linux-gnu": 1,
    "x86_64-unknown-linux-gnu": 2,
}
for target, expected_count in download_counts.items():
    exact_name = (
        "name: sifr-stable-candidate-${{ needs.validate.outputs.version }}-"
        "${{ needs.validate.outputs.source_commit }}-" + target
    )
    if text.count(exact_name) != expected_count:
        raise SystemExit(
            f"target {target} exact artifact download count drifted: "
            f"expected {expected_count}"
        )
PY

echo "release qualification workflow contract: PASS"
