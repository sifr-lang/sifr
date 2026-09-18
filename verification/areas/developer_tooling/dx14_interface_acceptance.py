"""Independent CLI processes: proven importer reuse and fresh semantic references."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def run(binary, output):
    output.mkdir(parents=True, exist_ok=False)
    project = output / "workspace"
    project.mkdir()\n    (project / "sifr.toml").write_text('[source]\\nroot = "."\\n')
    main = project / "main.sifr"
    helper = project / "helper.sifr"
    main.write_text("from helper import value\n\ndef main() -> int:\n    return value()\n")
    helper.write_text("def value() -> int:\n    return 1\n")
    env = dict(os.environ, SIFR_CACHE_DIR=str(output / "cache"))
    report = {"binary": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "rows": [], "cases": {}}

    def invoke(label, disabled=False, error=False):
        command = [str(binary), "--diagnostic-format", "json", "--timings"]
        if disabled:
            command.append("--no-incremental")
        command += ["check", str(main)]
        result = subprocess.run(command, env=env, capture_output=True, text=True)
        (output / (label + ".stdout")).write_text(result.stdout)
        (output / (label + ".stderr")).write_text(result.stderr)
        stats = next(json.loads(line.split("] ", 1)[1]) for line in result.stderr.splitlines()
                     if line.startswith("[sifr-project-cache] "))
        diagnostics = json.loads(result.stdout)
        report["rows"].append({"label": label, "command": command,
                               "returncode": result.returncode, "cache": stats})
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        assert result.returncode == int(error), (label, result.stdout, result.stderr)
        return diagnostics, stats

    _, initial = invoke("initial")
    assert initial["status"] == "published", initial
    helper.write_text("def value() -> int:\n    return 2\n")
    actual, reused = invoke("body-edit")
    assert reused["status"] == "interface-restored", reused
    assert any(row["action"] == "restored" and row["path"] == str(main)
               for row in reused["modules"]), reused
    assert any(row["action"] == "computed" and row["path"] == str(helper)
               for row in reused["modules"]), reused
    fresh, disabled = invoke("body-edit-fresh", True)
    assert actual == fresh and disabled["status"] == "disabled"
    report["cases"]["P06"] = "changed helper computed; unchanged importer check restored"

    for label, before, after, error in [
        ("default", "def value(x: int = 1) -> int:\n    return x\n",
         "def value(x: int = 2) -> int:\n    return x\n", False),
        ("constant", "N: int = 1\ndef value() -> int:\n    return N\n",
         "N: int = 2\ndef value() -> int:\n    return N\n", False),
        ("type-error", "def value() -> int:\n    return 1\n",
         'def value() -> int:\n    return "bad"\n', True),
    ]:
        helper.write_text(before)
        invoke(label + "-before")
        helper.write_text(after)
        actual, stats = invoke(label + "-after", error=error)
        fresh, _ = invoke(label + "-fresh", True, error)
        assert actual == fresh and stats["status"] != "interface-restored"
        if error:
            assert "SIFR-TYPE-" in json.dumps(actual)
    report["cases"]["P05"] = "defaults/constants/body errors match independent fresh outcomes"
    seed = 0x5EED
    reused_count = 0
    for step in range(16):
        seed = (seed * 6364136223846793005 + 1) & ((1 << 64) - 1)
        bad = step % 5 == 3
        helper.write_text('def value() -> int:\n    return "bad"\n' if bad else
                          f"def value() -> int:\n    return {seed % 10000}\n")
        actual, stats = invoke(f"edit-{step}", error=bad)
        fresh, _ = invoke(f"fresh-{step}", True, bad)
        assert actual == fresh
        reused_count += stats["status"] == "interface-restored"
    assert reused_count >= 8, reused_count
    report["cases"]["bounded-edits"] = {"steps": 16, "interface_reused": reused_count}
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report["cases"], indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.binary.resolve(), args.output.resolve())
