"""Read-only development metadata doctor regression, owned by metadata-structural."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


def run_development_doctor(
    binary: Path, source_root: Path, evidence: Path, environment: dict[str, str]
) -> None:
    evidence.mkdir(parents=True, exist_ok=False)
    cache = evidence / "cache"
    env = {
        **environment,
        "SIFR_SYSROOT": str(source_root),
        "SIFR_CACHE_DIR": str(cache),
    }
    report = {
        "binary": str(binary),
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "rows": [],
    }

    def run(name: str, arguments: list[str], success: bool) -> dict:
        result = subprocess.run(
            [str(binary), *arguments],
            cwd=source_root,
            env=env,
            capture_output=True,
            timeout=180,
        )
        (evidence / f"{name}.stdout").write_bytes(result.stdout)
        (evidence / f"{name}.stderr").write_bytes(result.stderr)
        report["rows"].append({"case": name, "exit_code": result.returncode})
        (evidence / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        assert (result.returncode == 0) == success, (
            name, result.returncode, result.stdout.decode(), result.stderr.decode()
        )
        return json.loads(result.stdout)

    def metadata_error(value: dict) -> None:
        assert value["status"] == "error" and value["error_kind"] == "metadata", value
        assert "build-metadata" in value["message"], value
        assert value["compiler_build_id"] in value["message"], value
        assert str(source_root) in value["message"], value

    missing = run("missing", ["doctor", "--json"], False)
    metadata_error(missing)
    # Doctor may inspect/create storage directories, but must not produce metadata.
    assert not any(cache.rglob("*.sifrmeta")), "doctor silently produced metadata"
    run("ensure", [
        "sysroot", "build-metadata", "--source-root", str(source_root),
        "--output", str(evidence / "published.sifrmeta"),
    ], True)
    ready = run("ready", ["doctor", "--json"], True)
    assert ready["compiler_build_id"] == missing["compiler_build_id"]
    assert ready["metadata"]["status"] == "ready", ready
    selected = Path(ready["metadata"]["path"])
    assert selected.is_relative_to(cache), selected
    original = selected.read_bytes()
    try:
        selected.write_bytes(b"incomplete")
        corrupt = run("corrupt", ["doctor", "--json"], False)
        metadata_error(corrupt)
        assert corrupt["compiler_build_id"] == ready["compiler_build_id"]
        assert selected.read_bytes() == b"incomplete", "doctor silently repaired metadata"
    finally:
        selected.write_bytes(original)
    recovered = run("recovered", ["doctor", "--json"], True)
    assert recovered["metadata"] == ready["metadata"], (ready, recovered)
    report["status"] = "passed"
    report["selected_metadata_sha256"] = hashlib.sha256(original).hexdigest()
    (evidence / "report.json").write_text(json.dumps(report, indent=2) + "\n")
