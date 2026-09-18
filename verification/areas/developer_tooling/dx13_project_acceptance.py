"""DX.13 independent CLI-process acceptance and measured project-cache costs.

Consumes an explicitly built compiler. Preparation runs separately; no timing
claim includes a cold metadata producer as an unchanged checking sample.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import time

GOOD = "def main() -> None:\n    pass\n"
BAD = 'def main() -> None:\n    value: int = "bad"\n'


def run(binary, output, compiler_profile, source_root):
    output.mkdir(parents=True, exist_ok=False)
    workspace = output / "workspace"
    workspace.mkdir()
    cache = output / "cache"
    env = dict(os.environ, SIFR_CACHE_DIR=str(cache))
    report = {"binary": str(binary), "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "compiler_profile": compiler_profile, "rows": [], "cases": {}}

    def invoke(label, args, expected=0):
        rss = output / (label + ".rss")
        selected = [str(binary)] + (["--sysroot", str(source_root)] if source_root else [])
        argv = ["/usr/bin/time", "-f", "%M", "-o", str(rss), *selected, *args]
        started = time.monotonic()
        proc = subprocess.run(argv, cwd=workspace, env=env, capture_output=True, text=True)
        elapsed = time.monotonic() - started
        (output / (label + ".stdout")).write_text(proc.stdout)
        (output / (label + ".stderr")).write_text(proc.stderr)
        stats = next((json.loads(line.split("] ", 1)[1]) for line in proc.stderr.splitlines()
                      if line.startswith("[sifr-project-cache] ")), None)
        row = {"label": label, "command": argv, "returncode": proc.returncode,
               "wall_seconds": elapsed, "peak_rss_kib": int(rss.read_text().splitlines()[-1]), "cache": stats}
        report["rows"].append(row)
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        assert proc.returncode == expected, (label, proc.returncode, proc.stdout, proc.stderr)
        if "check" in args:
            stream = proc.stdout.strip() or "\n".join(line for line in proc.stderr.splitlines()
                if not line.startswith(("[sifr-project-cache] ", "[sifr-timing] ")))
            proc.diagnostics = json.loads(stream)
        return proc, stats

    if source_root:
        invoke("explicit-metadata-preparation", ["sysroot", "build-metadata", "--source-root", str(source_root),
                                                "--output", str(output / "stdlib.sifrmeta")])
    main = workspace / "main.sifr"
    main.write_text(GOOD)
    args = ["--diagnostic-format", "json", "--timings", "check", "main.sifr"]
    fresh, stats = invoke("p01-initial", args)
    assert stats["computed_checks"] == 1 and stats["status"] == "published", stats
    restored, stats = invoke("p01-new-process", args)
    assert stats["restored_checks"] == 1 and restored.diagnostics == fresh.diagnostics, stats
    report["cases"]["P01"] = "new CLI process restores completed checking"
    main.write_text(BAD)
    bad, stats = invoke("p02-error", args, 1)
    assert "SIFR-TYPE-0002" in json.dumps(bad.diagnostics) and stats["computed_checks"] == 1
    bad_restored, stats = invoke("p02-restored-error", args, 1)
    assert stats["restored_checks"] == 1 and bad.diagnostics == bad_restored.diagnostics, stats
    main.write_text(GOOD)
    fixed, stats = invoke("p02-fixed", args)
    assert stats["restored_checks"] == 1 and fixed.diagnostics == fresh.diagnostics
    main.write_text(BAD)
    reverted, stats = invoke("p02-reverted-error", args, 1)
    assert stats["restored_checks"] == 1 and reverted.diagnostics == bad.diagnostics
    report["cases"]["P02"] = "error/fix/revert agrees with independent source expectations"
    main.write_text(GOOD)
    manifest_before = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in cache.rglob("manifest.json")}
    disabled, stats = invoke("p10-disabled", ["--no-incremental", *args])
    assert stats["status"] == "disabled" and stats["restored_checks"] == 0 and disabled.diagnostics == fresh.diagnostics
    manifest_after = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in cache.rglob("manifest.json")}
    assert manifest_before == manifest_after
    (workspace / ".sifrbuildinfo").unlink()
    _, stats = invoke("p10-no-hint", args)
    assert stats["restored_checks"] == 1
    report["cases"]["P10"] = "disabled cache does no project publication; hintless cache still restores"
    emitted, _ = invoke("p08-emit-after-check", ["emit", "main.sifr"])
    uncached_emit, _ = invoke("p08-emit-fresh", ["--no-incremental", "emit", "main.sifr"])
    assert emitted.stdout == uncached_emit.stdout and "fn main" in emitted.stdout
    report["cases"]["P08"] = "missing typed/codegen families computed by normal emit"
    # Optional project storage cannot turn valid source into a language error.
    projects = cache / "projects"
    held_projects = cache / "projects-held"
    projects.rename(held_projects)
    projects.write_text("inaccessible project namespace")
    unavailable, stats = invoke("c04-unavailable-project-cache", args)
    assert stats["status"] == "unavailable" and unavailable.diagnostics == fresh.diagnostics
    projects.unlink()
    held_projects.rename(projects)
    workspace.chmod(0o500)
    try:
        readonly, stats = invoke("c04-readonly-workspace", args)
        assert stats["restored_checks"] == 1 and readonly.diagnostics == fresh.diagnostics
    finally:
        workspace.chmod(0o700)
    report["cases"]["C04"] = "inaccessible optional project namespace and read-only hintless workspace preserve source result"
    # Exercise real import observations and canonical multi-module diagnostics.
    (workspace / "sifr.toml").write_text('[source]\nroot = "."\n')
    (workspace / "helper.sifr").write_text("def value() -> int:\n    return 4\n")
    main.write_text("from helper import value\ndef main() -> None:\n    print(value())\n")
    project, _ = invoke("project-initial", args)
    project_hit, stats = invoke("project-restored", args)
    assert stats["restored_checks"] == 1 and project.diagnostics == project_hit.diagnostics
    (workspace / "helper.sifr").write_text('def value() -> int:\n    return "bad"\n')
    changed, stats = invoke("project-edited-dependency", args, 1)
    assert stats["computed_checks"] == 1 and "SIFR-TYPE-0002" in json.dumps(changed.diagnostics)
    changed_hit, stats = invoke("project-edited-restored", args, 1)
    assert stats["restored_checks"] == 1 and changed.diagnostics == changed_hit.diagnostics
    direct, _ = invoke("project-independent-error", ["--no-incremental", *args], 1)
    assert direct.diagnostics == changed.diagnostics
    report["cases"]["dependencies"] = "changed imported source invalidates and canonical diagnostics match"
    main.write_text(GOOD)
    invoke("measurement-prepare", args)
    for sample in range(5):
        invoke(f"measured-fresh-{sample}", ["--no-incremental", *args])
        _, stats = invoke(f"measured-restored-{sample}", args)
        assert stats["restored_checks"] == 1
    report["measurement"] = {lane: {"median_wall_seconds": statistics.median(row["wall_seconds"]
        for row in report["rows"] if row["label"].startswith("measured-" + lane)),
        "peak_rss_kib": max(row["peak_rss_kib"] for row in report["rows"] if row["label"].startswith("measured-" + lane))}
        for lane in ["fresh", "restored"]}
    report["note"] = "Same candidate/profile descriptive samples; no phase-end optimized percentile claim."
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report["measurement"], indent=2))

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--compiler-profile", required=True)
    parser.add_argument("--source-root", type=Path)
    args = parser.parse_args()
    run(args.binary.resolve(), args.output.resolve(), args.compiler_profile,
        args.source_root.resolve() if args.source_root else None)
