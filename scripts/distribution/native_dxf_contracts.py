"""Affected DX contracts against the exact native installed package.

The canonical native runner supplies validated archive identity and records every
source test command. These checks make no latency or release claim.
"""
import json
import os
from pathlib import Path
import shlex
import sys

ROOT = Path(__file__).resolve().parents[2]


def qualify_dxf(owner, binary):
    sys.path.insert(0, str(ROOT / "verification/areas/developer_tooling"))
    from dxf_cache_cli_acceptance import qualify
    from qualify_native_package import digest, require

    binary = binary.resolve()
    expected = json.loads((owner.artifacts / f"qualification-{owner.target}.json").read_text())
    require(digest(binary) == expected["binary_sha256"], "DXF binary differs from exact archive")
    # Cache harness inherits no product sysroot override and writes only its own cache.
    cache_report = qualify(binary, owner.output / "dxf-cache", {
        "candidate": owner.source, "lane": "native-exact-package",
        "target_report": str(owner.artifacts / f"qualification-{owner.target}.json"),
        "target_report_sha256": digest(owner.artifacts / f"qualification-{owner.target}.json"),
    })
    workspace = owner.output / "dxf-trace"
    workspace.mkdir()
    source = workspace / "private-source-sentinel.sifr"
    source.write_text('def main() -> None:\n    value: int = 1\n')
    destination = workspace / "trace"
    command = [str(binary), "--trace-dir", str(destination), "check", str(source)]
    owner.run("dxf-private-trace", ["sh", "-c", "umask 000; exec " + shlex.join(command)],
              cwd=workspace)
    data = (destination / "trace-v1.json").read_bytes()
    trace = json.loads(data)
    require(destination.stat().st_mode & 0o777 == 0o700, "trace directory is not private")
    require((destination / "trace-v1.json").stat().st_mode & 0o777 == 0o600, "trace file is not private")
    require(len(data) <= 32768 and trace["schema_version"] == 1
            and trace["outcome"] == "success", "trace bound/schema/outcome mismatch")
    require(b"private-source-sentinel" not in data and str(workspace).encode() not in data,
            "trace leaked source path")
    # Source-level API contracts are separate from exact package execution.
    # The driver test target is also used by the canonical metadata corpus.
    selections = [
        ("sifr_driver", "project_cache::housekeeping_tests::abandoned_stages_reclaimed_live_locks_preserved"),
        ("sifr_driver", "project_cache::housekeeping_tests::orphan_prune_requires_owner_and_inactivity"),
        ("sifr_driver", "project_cache::housekeeping_tests::pressure_prune_counts_are_truthful"),
        ("sifr_driver", "project_cache::tests::moved_workspace_misses_without_changing_diagnostics"),
        ("sifr_lsp", "dxf_embedding_tests::production_workspace_preserves_context"),
    ]
    for index, (package, name) in enumerate(selections):
        command = ["cargo", "test", "--locked", "--offline", "-p", package,
                   "--lib", name, "--", "--exact"]
        listed = owner.run(f"dxf-{index}-list", command + ["--list"], cwd=ROOT, deadline=2400)
        tests = [line[:-6] for line in listed.decode().splitlines() if line.endswith(": test")]
        require(tests == [name], f"DXF selector must match exactly one test: {name}")
        owner.run(f"dxf-{index}-run", command + ["--nocapture"], cwd=ROOT, deadline=2400)
        owner.rows[-1]["selected_tests"] = tests
        owner.rows[-1]["selected_count"] = 1
        owner.save()
    owner.report["dxf"] = {
        "binary_sha256": digest(binary), "archive_sha256": expected["archive_sha256"],
        "cache_calls": len(cache_report["calls"]), "source_tests": len(selections),
        "trace_sha256": digest(destination / "trace-v1.json"),
        "scope": "exact-package cache CLI and Unix modes; native source ownership, root movement and embedding API",
        "limits": "source API tests are not tests inside the packaged executable; no performance claim",
    }
    owner.save()
