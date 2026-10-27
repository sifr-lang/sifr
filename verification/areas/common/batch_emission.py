"""Finite compiler batches; never reuse generated outputs or correctness results."""
from __future__ import annotations

import concurrent.futures
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib


def formatter_environment(repo: Path) -> dict[str, str]:
    env = dict(os.environ)
    if "RUSTFMT" in env:
        return env
    channel = tomllib.loads((repo / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    selected = subprocess.run(
        ["rustup", "which", "--toolchain", channel, "rustfmt"],
        cwd=repo, env=env, capture_output=True, check=True,
    ).stdout.decode().strip()
    formatter = Path(selected)
    if not formatter.is_absolute() or not formatter.is_file():
        raise RuntimeError("rustup did not select an existing absolute rustfmt")
    actual = subprocess.run(
        [selected, "--version"], cwd=repo, env=env, capture_output=True, check=True,
    ).stdout
    expected = subprocess.run(
        ["rustup", "run", channel, "rustfmt", "--version"],
        cwd=repo, env=env, capture_output=True, check=True,
    ).stdout
    if not actual.strip() or actual != expected:
        raise RuntimeError("selected rustfmt does not match the pinned toolchain")
    env["RUSTFMT"] = selected
    return env


def read_results(output: Path, files: list[Path], result) -> list[subprocess.CompletedProcess]:
    report = json.loads((output / "report.json").read_text())
    if (set(report) != {"schema", "entries"} or type(report["schema"]) is not int
            or report["schema"] != 1 or not isinstance(report["entries"], list)
            or len(report["entries"]) != len(files)):
        raise RuntimeError("incomplete or invalid emission batch report")
    expected_files = {"report.json"}
    results = []
    for index, (source, entry) in enumerate(zip(files, report["entries"], strict=True)):
        if (not isinstance(entry, dict)
                or set(entry) != {"index", "source", "exit_code", "stderr"}
                or type(entry["index"]) is not int or entry["index"] != index
                or entry["source"] != str(source)
                or type(entry["exit_code"]) is not int or entry["exit_code"] not in range(4)
                or not isinstance(entry["stderr"], str)):
            raise RuntimeError("mismatched emission batch entry")
        code = entry["exit_code"]
        if (code == 0) != (entry["stderr"] == ""):
            raise RuntimeError("inconsistent emission batch diagnostics")
        data = b""
        if code == 0:
            name = f"{index}.rs"
            expected_files.add(name)
            path = output / name
            if path.is_symlink() or not path.is_file():
                raise RuntimeError("missing regular emitted Rust file")
            data = path.read_bytes()
            if not data:
                raise RuntimeError("empty emitted Rust")
        results.append(subprocess.CompletedProcess(
            ["emit", str(source)], code, data, entry["stderr"].encode(),
        ))
    if result.returncode != max(item.returncode for item in results) or result.stdout:
        raise RuntimeError("inconsistent emission batch process result")
    if {path.name for path in output.iterdir()} != expected_files:
        raise RuntimeError("unexpected emission batch outputs")
    return results


def emit_many(compiler: Path, files: list[Path], jobs: int, repo: Path):
    if not files or jobs < 1:
        raise RuntimeError("emission requires inputs and positive worker count")
    if any(not path.is_absolute() for path in files):
        raise RuntimeError("emission requires absolute source paths")
    count = min(jobs, len(files))
    allocation = os.environ.get("SIFR_VERIFY_PROCESS_SLOTS")
    if allocation is not None:
        try:
            slots = int(allocation)
        except ValueError as error:
            raise RuntimeError("SIFR_VERIFY_PROCESS_SLOTS must be a positive integer") from error
        if slots < 1:
            raise RuntimeError("SIFR_VERIFY_PROCESS_SLOTS must be a positive integer")
        count = min(count, slots)
    env = formatter_environment(repo)
    if allocation is not None:
        # Parallel finite batches each own one slot of the declared budget.
        env.update(CARGO_BUILD_JOBS="1", RAYON_NUM_THREADS="1")
    owner = Path(tempfile.mkdtemp(prefix="sifr-emission-batch-"))
    chunks = [list(range(worker, len(files), count)) for worker in range(count)]

    def run(worker):
        indices = chunks[worker]
        sources = [files[index] for index in indices]
        manifest = owner / f"{worker}.json"
        output = owner / str(worker)
        manifest.write_text(json.dumps({"schema": 1, "files": [str(path) for path in sources]}))
        result = subprocess.run(
            [str(compiler), "__emit-batch", "--manifest", str(manifest),
             "--output-dir", str(output)],
            cwd=repo, env=env, capture_output=True, check=False,
        )
        (owner / f"{worker}.stdout").write_bytes(result.stdout)
        (owner / f"{worker}.stderr").write_bytes(result.stderr)
        # Successful scalar emission also permits ordinary compiler warnings.
        # Preserve raw streams for a failed batch; only status/report/output
        # consistency determines whether compilation completed.
        values = read_results(output, sources, result)
        return list(zip(indices, values, strict=True))

    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=count) as executor:
            chunks_results = list(executor.map(run, range(count)))
        ordered = sorted(item for chunk in chunks_results for item in chunk)
        results = [value for _, value in ordered]
        if any(value.returncode for value in results):
            # Retain request manifests, diagnostics and successful siblings for diagnosis.
            print(f"emission batch evidence: {owner}", file=sys.stderr)
        else:
            shutil.rmtree(owner)
        return results
    except Exception as error:
        raise RuntimeError(f"emission batch failed; evidence={owner}: {error}") from error
