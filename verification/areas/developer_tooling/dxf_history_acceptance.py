"""Bounded canonical history evidence plus a real CLI differential edit sequence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "verification/areas/performance"))
import compiler_lanes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--history-sizes", default="1,128,1024,4097")
    args = parser.parse_args()
    receipt = Path(os.environ["DXF_RECEIPT"])
    compiler_lanes.configure(ROOT, "contributor-dev", str(receipt))
    binary = args.binary.resolve()
    assert binary == compiler_lanes.selected_binary(binary)
    stress = Path(os.environ["DXF_HISTORY_REPORT"])
    rows = json.loads(stress.read_text())
    sizes = [int(value) for value in args.history_sizes.split(",")]
    assert [row["history_inputs"] for row in rows] == sizes
    for row in rows:
        assert row["retained_records"] == min(row["history_inputs"], 128)
        assert 0 < row["retained_bytes"] <= 64 * 1024 * 1024
        assert row["candidate_checks"] == 1 and row["interface_proofs"] == 0
        assert row["observation_count"] == 1
    args.output.mkdir(parents=True, exist_ok=False)
    workspace = args.output / "workspace"
    workspace.mkdir()
    (workspace / "sifr.toml").write_text('[source]\nroot = "."\n')
    file = workspace / "main.sifr"
    helper = workspace / "helper.sifr"
    file.write_text("from helper import value\ndef main() -> int:\n    return value()\n")
    env = dict(os.environ, SIFR_CACHE_DIR=str(args.output / "cache"))
    calls = []
    def invoke(label, fresh):
        command = [str(binary), "--diagnostic-format", "json", "--timings"]
        if fresh:
            command.append("--no-incremental")
        command += ["check", str(file)]
        start = time.monotonic()
        process = subprocess.run(command, cwd=args.output, env=env, text=True, capture_output=True)
        elapsed = time.monotonic() - start
        (args.output / (label + ".stdout")).write_text(process.stdout)
        (args.output / (label + ".stderr")).write_text(process.stderr)
        cache = next((json.loads(line.split("] ", 1)[1]) for line in process.stderr.splitlines()
                      if line.startswith("[sifr-project-cache] ")), None)
        calls.append({"command": command, "label": label, "returncode": process.returncode,
                      "wall_seconds": elapsed, "cache": cache})
        stream = process.stdout.strip() or "\n".join(
            line for line in process.stderr.splitlines()
            if not line.startswith(("[sifr-project-cache] ", "[sifr-metadata] ", "[sifr-timing] ")))
        return process.returncode, json.loads(stream), cache
    for index, body in enumerate(["1", "2", "1", '"bad"', "3"]):
        helper.write_text(f"def value() -> int:\n    return {body}\n")
        actual = invoke(f"{index}-incremental", False)
        fresh = invoke(f"{index}-fresh", True)
        assert actual[:2] == fresh[:2]
        assert actual[0] == int(body == '"bad"')
        assert actual[2]["interface_proofs"] <= 1
        if index == 1:
            assert actual[2]["status"] == "interface-restored"
        if index == 2:
            assert actual[2]["status"] == "restored"
    report = {"lane": "contributor-dev", "claim": "functional; no product speedup claim",
              "binary": str(binary), "binary_sha256": compiler_lanes.digest(binary),
              "receipt": str(receipt), "receipt_sha256": compiler_lanes.digest(receipt),
              "history_source": str(stress), "history_sha256": hashlib.sha256(stress.read_bytes()).hexdigest(),
              "history": rows, "real_edit_calls": calls,
              "limits": "History rows time retained-cap exact lookup, including manifest validation; "
                        "not constant total storage I/O. Old generation reclamation remains explicit prune."}
    (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
