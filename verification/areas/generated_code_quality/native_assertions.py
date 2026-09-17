"""Native linking/runtime assertions over the existing materialized corpus."""
import json
import os
from pathlib import Path

def run_native_gate(api, entries, args):
    run = api.run_id("native")
    root = api.TARGET_ROOT / run
    records = []
    failed = False
    for entry in api.selected_positive_entries(entries, args.group):
        record = {"id": entry.id, "source_path": entry.source_path,
                  "application_profile": "release",
                  "assertions": ["materialize", "native-compile-link", "runtime"],
                  "stages": []}
        records.append(record)
        stage = "materialize"
        try:
            if entry.native_stdout is None:
                raise RuntimeError("native selection requires independently declared stdout")
            crate = api.materialize_entry(entry, root)
            record["stages"].append({"stage": stage, "status": "pass"})
            stage = "native-compile-link"
            proc = api.run_command(["cargo", "build", "--locked", "--release",
                "--message-format=json", "--manifest-path", str(crate / "Cargo.toml")])
            executables = []
            for line in proc.stdout.splitlines():
                message = json.loads(line)
                if message.get("reason") == "compiler-artifact" and message.get("executable"):
                    executables.append(message["executable"])
            if not executables:
                raise RuntimeError("native link produced no executable")
            record["stages"].append({"stage": stage, "status": "pass"})
            stage = "runtime"
            for executable in executables:
                # Assertions run even if Cargo reports every artifact fresh.
                proc = api.run_command([executable], cwd=api.REPO_ROOT)
                record["stdout"] = proc.stdout
                record["stderr"] = proc.stderr
                if proc.stdout != entry.native_stdout or proc.stderr:
                    raise RuntimeError(f"runtime output mismatch: stdout={proc.stdout!r}; stderr={proc.stderr!r}")
            record["stages"].append({"stage": stage, "status": "pass"})
            record["status"] = "passed"
        except Exception as error:
            failed = True
            record["status"] = "failed"
            record["stages"].append({"stage": stage, "status": "fail", "reason": str(error)})
            for dependent in record["assertions"][record["assertions"].index(stage)+1:]:
                record["stages"].append({"stage": dependent, "status": "blocked"})
        evidence = api.record_evidence("native", run, records)
        if os.environ.get("SIFR_GCQ_CASE_REPORT"):
            Path(os.environ["SIFR_GCQ_CASE_REPORT"]).write_text(
                json.dumps({"evidence": str(evidence), "cases": records}, indent=2))
    if failed:
        raise RuntimeError(f"native assertions failed; evidence={run}.json")
