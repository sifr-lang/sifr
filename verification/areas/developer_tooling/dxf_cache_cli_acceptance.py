"""Receipt-bound cache CLI contracts; every operation uses private temporary storage."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "verification/areas/performance"))
import compiler_lanes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    receipt = Path(os.environ["DXF_RECEIPT"]).resolve()
    compiler_lanes.configure(ROOT, "contributor-dev", str(receipt))
    binary = args.binary.resolve()
    assert binary == compiler_lanes.selected_binary(binary)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    workspace = output / "workspace"
    workspace.mkdir()
    (workspace / "sifr.toml").write_text('[source]\nroot = "."\n')
    source = workspace / "main.sifr"
    cache = output / "cache"
    env = dict(os.environ, SIFR_CACHE_DIR=str(cache))
    calls = []

    def invoke(label, arguments, expected=0):
        command = [str(binary), *arguments]
        result = subprocess.run(command, cwd=workspace, env=env, text=True,
                                capture_output=True, timeout=180)
        (output / f"{label}.stdout").write_text(result.stdout)
        (output / f"{label}.stderr").write_text(result.stderr)
        calls.append({"label": label, "command": command, "returncode": result.returncode})
        assert result.returncode == expected, (label, result.stdout, result.stderr)
        return result

    def report(label, arguments):
        result = invoke(label, ["cache", *arguments])
        assert result.stderr == "", (label, result.stderr)
        return json.loads(result.stdout)

    for command in ["inspect", "prune", "prune-project"]:
        help_text = invoke(command + "-help", ["cache", command, "--help"]).stdout
        assert f"sifr cache {command}" in help_text
        assert "--no-incremental" in help_text
    for placement, arguments in enumerate([
        ["--isolated", "--no-incremental", "cache", "inspect", "--json"],
        ["cache", "--isolated", "--no-incremental", "inspect", "--json"],
        ["cache", "inspect", "--json", "--isolated", "--no-incremental"],
    ]):
        inspected = json.loads(invoke(f"global-{placement}", arguments).stdout)
        assert inspected == {"root": str(cache), "entries": [], "protected_roots": []}
    text = invoke("inspect-text", ["cache", "inspect"]).stdout
    assert text == f"cache: {cache}\n"
    for index, arguments in enumerate([
        ["cache"], ["cache", "bogus"], ["cache", "inspect", "--reserve-bytes", "1"],
        ["cache", "prune", "--json"], ["cache", "prune-project"],
        *[["cache", command, *([str(workspace)] if command == "prune-project" else []),
           "--reserve-bytes", *value]
          for command in ["prune", "prune-project"]
          for value in [[], ["bad"], ["-1"], ["18446744073709551616"]]],
    ]):
        invalid = invoke(f"invalid-{index}", arguments, 2)
        assert not invalid.stdout and "error:" in invalid.stderr
    maximum = str(2**64 - 1)
    for label, options in [("native-default", []), ("native-zero", ["--reserve-bytes", "0"]),
                           ("native-dry", ["--dry-run", "--reserve-bytes", maximum])]:
        value = report(label, ["prune", *options])
        assert set(value) == {"dry_run", "reason", "reserve_bytes", "available_bytes", "scope", "cache"}
        assert value["reason"] == "free_space_reserve" and value["scope"] == str(workspace)
        assert value["dry_run"] == (label == "native-dry")
        assert value["reserve_bytes"] == (int(maximum) if value["dry_run"] else 0)
        assert type(value["available_bytes"]) is int and value["available_bytes"] > 0
        assert value["cache"]["entries"] == []
    blocker = invoke("native-reserve-blocker", ["cache", "prune", "--reserve-bytes", maximum], 2)
    assert not blocker.stdout and "resource blocker:" in blocker.stderr
    assert "protected entries retained" in blocker.stderr

    # Real frontend publications create two generations; no fabricated cache metadata.
    for number in [1, 2]:
        source.write_text(f"def main() -> None:\n    value: int = {number}\n")
        invoke(f"seed-{number}", ["check", str(source)])
    generations = sorted(cache.glob("projects/**/generations/*"))
    generations = [path for path in generations if path.is_dir() and path.name != ".locks"]
    assert len(generations) == 2, generations
    before = {str(path): path.read_bytes() for path in cache.rglob("*") if path.is_file()}
    count_keys = {"examined_entries", "eligible_entries", "deleted_entries",
                  "eligible_generations", "deleted_generations"}

    def project(label, options):
        value = report(label, ["prune-project", str(workspace), *options])
        assert set(value) == {"workspace", "dry_run", "reserve_bytes", "available_bytes", "pressure", "cache"}
        assert value["workspace"] == str(workspace)
        assert set(value["cache"]) == count_keys
        assert all(type(count) is int and count >= 0 for count in value["cache"].values())
        assert value["pressure"] == (value["available_bytes"] < value["reserve_bytes"])
        return value

    for label, options in [("project-default", []), ("project-zero", ["--reserve-bytes", "0"])]:
        value = project(label, options)
        assert value["reserve_bytes"] == 0 and not value["pressure"] and not value["dry_run"]
        assert all(count == 0 for count in value["cache"].values())
    dry = project("project-dry", ["--dry-run", "--reserve-bytes", maximum])
    assert dry["dry_run"] and dry["pressure"]
    assert dry["cache"] == dict(examined_entries=2, eligible_entries=1, deleted_entries=0,
                                eligible_generations=1, deleted_generations=0)
    assert before == {str(path): path.read_bytes() for path in cache.rglob("*") if path.is_file()}
    deleted = project("project-delete", ["--reserve-bytes", maximum])
    assert not deleted["dry_run"] and deleted["pressure"]
    assert deleted["cache"] == dict(examined_entries=2, eligible_entries=1, deleted_entries=1,
                                    eligible_generations=1, deleted_generations=1)
    assert sum(path.exists() for path in generations) == 1
    again = project("project-idempotent", ["--reserve-bytes", maximum])
    assert again["cache"] == dict(examined_entries=1, eligible_entries=0, deleted_entries=0,
                                  eligible_generations=0, deleted_generations=0)
    invoke("after-prune", ["check", str(source)])
    # The old absolute root still selects only its own orphan namespace.
    moved = output / "moved"
    shutil.move(workspace, moved)
    workspace = moved
    orphan = report("orphan-delete", ["prune-project", str(output / "workspace"), "--reserve-bytes", maximum])
    assert orphan["pressure"] and orphan["cache"]["deleted_entries"] > 0
    assert orphan["cache"]["deleted_generations"] == 1
    invoke("after-orphan", ["check", str(moved / "main.sifr")])
    evidence = {"candidate": compiler_lanes.selection()["source_commit"],
                "lane": "contributor-dev", "claim": "functional contracts; no performance claim",
                "binary": str(binary), "binary_sha256": compiler_lanes.digest(binary),
                "receipt": str(receipt), "receipt_sha256": compiler_lanes.digest(receipt),
                "cache": str(cache), "calls": calls}
    (output / "report.json").write_text(json.dumps(evidence, indent=2) + "\n")
    print(f"PASS: {len(calls)} isolated CLI calls")


if __name__ == "__main__":
    main()
