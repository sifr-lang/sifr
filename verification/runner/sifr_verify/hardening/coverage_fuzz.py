"""Instrumented Sifr fuzzing with explicit infrastructure and finding outcomes."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[4]
MANIFEST = ROOT / "verification/areas/fuzz_property/coverage_fuzz_manifest.json"
MAX_OUTPUT_TAIL = 8192
EXECUTIONS = re.compile(r"(?:stat::number_of_executed_units:\s*|^#)([0-9]+)", re.MULTILINE)
COVERAGE = re.compile(r"\bcov:\s*([0-9]+)")
ARTIFACT = re.compile(r"(?:Test unit written to |artifact_prefix=)(\S+)")
MINIMIZED_ARTIFACT = re.compile(r"Minimized artifact:\s*(\S+)")
OFFLINE_ERRORS = ("no matching package named", "failed to download", "offline mode", "download of")
MISSING_TOOL_ERRORS = ("no such command: `fuzz`", "toolchain 'nightly' is not installed", "toolchain `nightly")
FINDING_SIGNAL = re.compile(r"ERROR: (?:AddressSanitizer|libFuzzer)|panicked at|thread '.+' panicked")


def file_hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bounded_tail(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="replace")
    return value[-MAX_OUTPUT_TAIL:]


def invoke(argv: list[str], *, timeout: int, env: dict[str, str]) -> dict[str, Any]:
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            argv, cwd=ROOT, env=env, text=True, capture_output=True,
            check=False, timeout=timeout,
        )
        output = proc.stdout + proc.stderr
        count, coverage = counters(output)
        return {
            "exit_code": proc.returncode,
            "output_tail": bounded_tail(output),
            "executions": count, "coverage_edges": coverage,
            "timed_out": False,
            "duration_ms": round((time.perf_counter() - started) * 1000, 3),
        }
    except FileNotFoundError as error:
        return {
            "exit_code": 127, "output_tail": bounded_tail(str(error)),
            "timed_out": False, "duration_ms": 0,
        }
    except subprocess.TimeoutExpired as error:
        return {
            "exit_code": 124,
            "output_tail": bounded_tail(bounded_tail(error.stdout) + bounded_tail(error.stderr)),
            "timed_out": True,
            "duration_ms": round((time.perf_counter() - started) * 1000, 3),
        }


def classify_preflight(result: dict[str, Any]) -> str:
    output = result["output_tail"].lower()
    if result["exit_code"] == 127 or any(marker in output for marker in MISSING_TOOL_ERRORS):
        return "missing-tool"
    if result["timed_out"]:
        return "instrumented-build-timeout"
    if any(marker in output for marker in OFFLINE_ERRORS):
        return "offline-dependency-failure"
    if result["exit_code"] != 0:
        return "instrumented-build-failure"
    return "pass"


def counters(output: str) -> tuple[int, int]:
    executions = [int(value) for value in EXECUTIONS.findall(output)]
    coverage = [int(value) for value in COVERAGE.findall(output)]
    return max(executions, default=0), max(coverage, default=0)


def identity(
    manifest: dict[str, Any], target: str, corpus: Path, *, tool: str, rustc: str,
) -> dict[str, Any]:
    return {
        "manifest_sha256": file_hash(MANIFEST),
        "cargo_lock_sha256": file_hash(ROOT / "verification/fuzz/Cargo.lock"),
        "target_source_sha256": file_hash(ROOT / f"verification/fuzz/fuzz_targets/{target}.rs"),
        "corpus": {
            path.name: file_hash(path)
            for path in sorted(corpus.iterdir())
            if path.is_file()
        },
        "tool": tool.strip(),
        "rustc": rustc.strip(),
        "configuration": {
            "target": target,
            "cargo_fuzz_version": manifest["cargo_fuzz_version"],
            "corpus": manifest["targets"][target]["corpus"],
            "budgets_seconds": manifest["budgets_seconds"],
            "build_timeout_seconds": manifest["build_timeout_seconds"],
        },
    }


def variant(label: str, status: str, argv: list[str], result: dict[str, Any]) -> dict[str, Any]:
    return {"label": label, "status": status, "argv": argv, **result}


def preserve_finding(
    *,
    target: str,
    output: str,
    artifact_dir: Path,
    env: dict[str, str],
    label: str,
) -> dict[str, Any]:
    candidates = [
        ROOT / match for match in ARTIFACT.findall(output)
        if (ROOT / match).is_file() and (ROOT / match).resolve().is_relative_to(artifact_dir.resolve())
    ]
    candidates.extend(path for path in artifact_dir.iterdir() if path.is_file())
    source = next((path for path in candidates if path.is_file()), None)
    if source is None:
        return {"status": "compiler-finding-unminimized", "reason": "artifact-missing"}
    minimize_argv = [
        "cargo", "+nightly", "fuzz", "tmin", "--fuzz-dir", "verification/fuzz",
        target, str(source), "--", "-max_total_time=60",
    ]
    minimized = invoke(minimize_argv, timeout=120, env=env)
    if minimized["exit_code"] != 0:
        return {
            "status": "compiler-finding-unminimized",
            "artifact": str(source), "minimization": variant(label + ":minimize", "fail", minimize_argv, minimized),
        }
    match = MINIMIZED_ARTIFACT.search(minimized["output_tail"])
    minimized_source = ROOT / match.group(1) if match else None
    if minimized_source is None or not minimized_source.is_file():
        return {
            "status": "compiler-finding-unminimized",
            "artifact": str(source), "reason": "minimized-artifact-missing",
            "minimization": variant(label + ":minimize", "fail", minimize_argv, minimized),
        }
    digest = file_hash(minimized_source)
    suffix = ".json" if target == "diagnostics" else ""
    saved = artifact_dir / ("minimized-" + digest[:16] + suffix)
    shutil.copyfile(minimized_source, saved)
    if target == "diagnostics":
        try:
            payload = json.loads(saved.read_text(encoding="utf-8"))
            if (not isinstance(payload, dict) or payload.get("version") != 1
                    or not isinstance(payload.get("diagnostics"), list)
                    or not payload["diagnostics"]):
                raise ValueError("minimized seed is not a diagnostic envelope")
        except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as error:
            return {
                "status": "compiler-finding-unminimized",
                "artifact": str(source), "reason": f"minimized-json-invalid: {error}",
                "minimization": variant(label + ":minimize", "fail", minimize_argv, minimized),
            }
    replay_argv = [
        "cargo", "+nightly", "fuzz", "run", "--fuzz-dir", "verification/fuzz",
        target, str(saved), "--", "-runs=1",
    ]
    replays = [invoke(replay_argv, timeout=90, env=env) for _ in range(2)]
    signals = [FINDING_SIGNAL.search(item["output_tail"]) for item in replays]
    stable = (
        all(item["exit_code"] != 0 and not item["timed_out"] for item in replays)
        and all(signal is not None for signal in signals)
        and len({signal.group(0) for signal in signals if signal is not None}) == 1
    )
    return {
        "status": "compiler-finding" if stable else "compiler-finding-unconfirmed",
        "artifact": str(source),
        "minimized_seed": str(saved),
        "minimized_sha256": file_hash(saved),
        "replay_command": replay_argv,
        "minimization": variant(label + ":minimize", "pass", minimize_argv, minimized),
        "replays": [
            variant(label + f":replay-{index}", "pass" if item["exit_code"] != 0 else "fail", replay_argv, item)
            for index, item in enumerate(replays, 1)
        ],
    }


def run(
    *,
    profile: str,
    target: str | None = None,
    corpus: Path | None = None,
    budget_override: int | None = None,
) -> dict[str, Any]:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    target = target or manifest["default_target"]
    if target not in manifest["targets"]:
        raise ValueError(f"unknown fuzz target: {target}")
    if corpus is None:
        corpus = ROOT / manifest["targets"][target]["corpus"]
    if profile not in manifest["budgets_seconds"]:
        raise ValueError(f"unknown fuzz profile: {profile}")
    if not corpus.is_dir() or not any(path.is_file() for path in corpus.iterdir()):
        raise ValueError(f"empty or missing corpus: {corpus}")
    budget = budget_override if budget_override is not None else manifest["budgets_seconds"][profile]
    if budget < 1:
        raise ValueError("fuzz budget must be positive")
    label = f"sustained-fuzz:{target}:{profile}"
    env = {**os.environ, "CARGO_NET_OFFLINE": "true"}
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    tool_argv = ["cargo", "+nightly", "fuzz", "--version"]
    rustc_argv = ["rustc", "+nightly", "--version"]
    tool = invoke(tool_argv, timeout=60, env=env)
    rustc = invoke(rustc_argv, timeout=60, env=env)
    receipt: dict[str, Any] = {
        "schema_version": 1, "target": target, "profile": profile,
        "budget_seconds": budget, "variants": [],
        "input_identity": identity(
            manifest, target, corpus, tool=tool["output_tail"], rustc=rustc["output_tail"],
        ),
    }
    if tool["exit_code"] != 0 or rustc["exit_code"] != 0:
        receipt["status"] = "missing-tool"
        receipt["variants"].append(variant(label + ":tool", "missing-tool", tool_argv, tool))
        return receipt

    build_argv = [
        "cargo", "+nightly", "fuzz", "build", "--fuzz-dir", "verification/fuzz", target,
    ]
    build = invoke(build_argv, timeout=manifest["build_timeout_seconds"], env=env)
    build_status = classify_preflight(build)
    receipt["variants"].append(variant(label + ":build", build_status, build_argv, build))
    if build_status != "pass":
        receipt["status"] = build_status
        return receipt

    artifact_root = ROOT / "target/verification/fuzz/artifacts" / target
    artifact_root.mkdir(parents=True, exist_ok=True)
    artifact_dir = Path(tempfile.mkdtemp(prefix=f"{profile}-", dir=artifact_root))
    receipt["artifact_dir"] = str(artifact_dir)
    corpus_root = ROOT / "target/verification/fuzz/corpus"
    corpus_root.mkdir(parents=True, exist_ok=True)
    working_corpus = Path(tempfile.mkdtemp(prefix=f"{target}-{profile}-", dir=corpus_root))
    for seed in corpus.iterdir():
        if seed.is_file():
            shutil.copyfile(seed, working_corpus / seed.name)
    receipt["working_corpus"] = str(working_corpus)
    run_argv = [
        "cargo", "+nightly", "fuzz", "run", "--fuzz-dir", "verification/fuzz",
        target, str(working_corpus), "--", f"-max_total_time={budget}",
        f"-artifact_prefix={artifact_dir}/", "-print_final_stats=1",
    ]
    execution = invoke(run_argv, timeout=budget + manifest["target_grace_seconds"], env=env)
    count = execution.get("executions")
    coverage = execution.get("coverage_edges")
    if count is None or coverage is None:
        count, coverage = counters(execution["output_tail"])
    if execution["timed_out"]:
        status = "target-timeout"
    elif execution["exit_code"] != 0:
        has_new_artifact = any(path.is_file() for path in artifact_dir.iterdir())
        status = (
            "compiler-finding"
            if FINDING_SIGNAL.search(execution["output_tail"]) or has_new_artifact
            else "target-run-failure"
        )
    elif count < 1:
        status = "no-guided-executions"
    elif coverage < 1:
        status = "no-coverage-feedback"
    else:
        status = "pass"
    receipt["variants"].append({
        **variant(label + ":run", status, run_argv, execution),
        "executions": count, "coverage_edges": coverage,
    })
    receipt["status"] = status
    if status == "compiler-finding":
        receipt["finding"] = preserve_finding(
            target=target, output=execution["output_tail"], artifact_dir=artifact_dir,
            env=env, label=label,
        )
        receipt["status"] = receipt["finding"]["status"]
    return receipt


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("nightly", "release"), required=True)
    parser.add_argument("--target")
    parser.add_argument("--corpus", type=Path)
    parser.add_argument("--budget-seconds", type=int)
    parser.add_argument("--result-json", type=Path, required=True)
    args = parser.parse_args()
    receipt = run(
        profile=args.profile, target=args.target, corpus=args.corpus,
        budget_override=args.budget_seconds,
    )
    args.result_json.parent.mkdir(parents=True, exist_ok=True)
    args.result_json.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "result_json": str(args.result_json)}))
    return 0 if receipt["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
