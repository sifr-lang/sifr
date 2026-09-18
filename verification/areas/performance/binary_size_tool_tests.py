"""Whole-file size harness controls; synthetic artifacts are not native size evidence."""
import json
import os
from pathlib import Path
import subprocess
import tempfile


def run_self_test():
    tool = Path(__file__).parent / "tools/check_codegen_binary_size.sh"
    for delta in (-1, 0, 1):
        with tempfile.TemporaryDirectory(prefix="sifr-size-tool-test-") as directory:
            root = Path(directory)
            repo = root / "repository"
            repo.mkdir()
            env = dict(os.environ)
            for key in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"):
                env.pop(key, None)
            env.update(GIT_AUTHOR_NAME="Size test", GIT_COMMITTER_NAME="Size test",
                       GIT_AUTHOR_EMAIL="test@example.invalid",
                       GIT_COMMITTER_EMAIL="test@example.invalid")

            def git(*args):
                return subprocess.check_output(
                    ["git", *args], cwd=repo, env=env, text=True,
                    stderr=subprocess.DEVNULL,
                ).strip()

            git("init", "-q")
            ruff = repo / "third_party/ruff/crates/ruff_text_size/Cargo.toml"
            ruff.parent.mkdir(parents=True)
            ruff.write_text("# Test-only materialized parser marker\n")
            (repo / "size").write_text("128")
            git("add", ".")
            git("commit", "-qm", "baseline")
            baseline = git("rev-parse", "HEAD")
            (repo / "size").write_text(str(128 + delta))
            git("add", ".")
            git("commit", "--allow-empty", "-qm", "candidate")
            candidate = git("rev-parse", "HEAD")
            tools = root / "tools"
            tools.mkdir()
            cargo = tools / "cargo"
            cargo.write_text("""#!/usr/bin/env python3
import json,os,pathlib,subprocess,sys
assert sys.argv[1:5] == ["run","-p","sifr","--"], sys.argv
assert sys.argv[5] == "build" and "--release" in sys.argv
out=pathlib.Path(sys.argv[sys.argv.index("--output")+1])
binary=out/"sifr_output/target/final/sifr_output"
binary.parent.mkdir(parents=True,exist_ok=True)
size=int(pathlib.Path("size").read_text())
binary.write_bytes(b"x"*size)
row={"cwd":os.getcwd(),"output":str(out),"size":size,
     "head":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip()}
with open(os.environ["SIFR_SIZE_TEST_LOG"],"a") as log:
 log.write(json.dumps(row)+"\\n")
""")
            cargo.chmod(0o755)
            log = root / "calls.jsonl"
            env.update(PATH=str(tools) + os.pathsep + env["PATH"],
                       SIFR_SIZE_TEST_LOG=str(log), TMPDIR=str(root))
            # Default candidate HEAD must resolve before switching the private checkout.
            result = subprocess.run(
                ["bash", str(tool), baseline], cwd=repo, env=env,
                capture_output=True, text=True,
            )
            assert result.returncode == (2 if delta > 0 else 0), result.stderr
            rows = [json.loads(line) for line in log.read_text().splitlines()]
            assert len(rows) == 2
            assert [row["head"] for row in rows] == [baseline, candidate]
            assert rows[0]["cwd"] == rows[1]["cwd"]
            assert rows[0]["output"] == rows[1]["output"]
            assert [row["size"] for row in rows] == [128, 128 + delta]
            assert f"delta_bytes={delta}\n" in result.stdout
            assert git("rev-parse", "HEAD") == candidate
            assert len(git("worktree", "list", "--porcelain").split("worktree ")) == 2
    print("Binary size tool controls: PASS (fixed paths; whole-byte increase rejects)")


if __name__ == "__main__":
    run_self_test()
