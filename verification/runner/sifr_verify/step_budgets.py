"""Cache-aware profile step-budget policy and deterministic receipts."""

from __future__ import annotations

import contextlib
import hashlib
import io
import json
import os
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from unittest.mock import patch

CACHE_CLASSIFIER = "successful-input-receipt"
RECEIPT_SCHEMA_VERSION = 1


@dataclass(frozen=True)
class StepBudgetContext:
    name: str
    budget_ms: int
    enforcement: str
    cache_state: str | None = None
    cache_reason: str | None = None
    cache_fingerprint: str | None = None
    receipt_path: Path | None = None
    receipt_eligible: bool = False
    required_cache_paths: tuple[Path, ...] = ()
    area_id: str | None = None


def prepare_step_budget(
    *,
    repo_root: Path,
    profile: dict[str, Any],
    profile_name: str,
    name: str,
    env: dict[str, str],
) -> StepBudgetContext | None:
    budgets = profile.get("step_budgets", {})
    raw_budget = budgets.get(name) if isinstance(budgets, dict) else None
    if not isinstance(raw_budget, dict):
        return None
    enforcement = str(raw_budget.get("enforcement", "advisory"))
    if "budget_ms" in raw_budget:
        return StepBudgetContext(
            name=name,
            budget_ms=int(raw_budget.get("budget_ms", 0) or 0),
            enforcement=enforcement,
        )

    area_id = selected_area_id(name)
    suites = selected_suites(profile, area_id)
    binary = Path(env.get("SIFR_GCQ_BIN", repo_root / "target" / "debug" / "sifr"))
    required_paths = required_cache_paths(repo_root, area_id, binary, env)
    fingerprint, eligible, ineligible_reason = input_fingerprint(
        repo_root=repo_root,
        profile_name=profile_name,
        step_name=name,
        area_id=area_id,
        suites=suites,
        sifr_binary=binary,
        cache_paths=required_paths,
        environment=env,
    )
    receipt_path = (
        repo_root
        / "target"
        / "verification"
        / "cache-receipts"
        / profile_name
        / f"{area_id or name}.json"
    )
    if not eligible:
        state, reason = "cold", ineligible_reason
    else:
        state, reason = classify_receipt(
            receipt_path=receipt_path,
            fingerprint=fingerprint,
            required_paths=required_paths,
        )
    budget_key = "warm_budget_ms" if state == "warm" else "cold_budget_ms"
    context = StepBudgetContext(
        name=name,
        budget_ms=int(raw_budget.get(budget_key, 0) or 0),
        enforcement=enforcement,
        cache_state=state,
        cache_reason=reason,
        cache_fingerprint=fingerprint,
        receipt_path=receipt_path,
        receipt_eligible=eligible,
        required_cache_paths=required_paths,
        area_id=area_id,
    )
    env["SIFR_VERIFY_STEP_CACHE_STATE"] = state
    if area_id == "python_interop":
        env["SIFR_PYTHON_INTEROP_CACHE_STATE"] = state
    print(
        f"[sifr-lane-step-cache] name={name} state={state} reason={reason} fingerprint={fingerprint[:16]}"
    )
    return context


def enforce_step_budget(context: StepBudgetContext | None, elapsed_ms: int) -> int:
    if context is None:
        return 0
    print(f"[sifr-step-contract] name={context.name} kind=performance_budget")
    exceeded = context.budget_ms > 0 and elapsed_ms > context.budget_ms
    budget_status = "fail" if exceeded else "pass"
    print(
        "[sifr-lane-step-budget] "
        f"name={context.name} elapsed_ms={elapsed_ms} budget_ms={context.budget_ms} "
        f"enforcement={context.enforcement} status={budget_status}"
    )
    disabled = os.environ.get("SIFR_VERIFY_DISABLE_STEP_BUDGETS", "").lower() in {
        "1",
        "true",
        "yes",
    }
    if exceeded and context.enforcement == "blocking" and not disabled:
        print(
            f"sifr_verify: step budget exceeded for {context.name}: {elapsed_ms}ms > {context.budget_ms}ms",
            file=sys.stderr,
        )
        return 124
    return 0


def record_step_success(context: StepBudgetContext | None, elapsed_ms: int) -> None:
    if (
        context is None
        or context.receipt_path is None
        or context.cache_fingerprint is None
        or not context.receipt_eligible
        or any(not cache_path_available(path) for path in context.required_cache_paths)
    ):
        return
    observations: list[dict[str, Any]] = []
    try:
        previous = json.loads(context.receipt_path.read_text(encoding="utf-8"))
        if isinstance(previous, dict) and previous.get("input_fingerprint") == context.cache_fingerprint:
            saved = previous.get("observations")
            if isinstance(saved, list):
                observations = saved[-3:]
    except (OSError, json.JSONDecodeError):
        pass
    observations.append({"cache_state": context.cache_state, "elapsed_ms": elapsed_ms})
    payload = {
        "schema_version": RECEIPT_SCHEMA_VERSION,
        "classifier": CACHE_CLASSIFIER,
        "step": context.name,
        "area": context.area_id,
        "input_fingerprint": context.cache_fingerprint,
        "observations": observations,
    }
    atomic_write_json(context.receipt_path, payload)


def classify_receipt(
    *, receipt_path: Path, fingerprint: str, required_paths: tuple[Path, ...]
) -> tuple[str, str]:
    missing_cache = [path for path in required_paths if not cache_path_available(path)]
    if missing_cache:
        return "cold", "required-cache-missing"
    try:
        payload = json.loads(receipt_path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return "cold", "receipt-missing"
    except (OSError, json.JSONDecodeError):
        return "cold", "receipt-invalid"
    if not isinstance(payload, dict):
        return "cold", "receipt-invalid"
    if (
        payload.get("schema_version") != RECEIPT_SCHEMA_VERSION
        or payload.get("classifier") != CACHE_CLASSIFIER
    ):
        return "cold", "receipt-invalid"
    if payload.get("input_fingerprint") != fingerprint:
        return "cold", "input-changed"
    return "warm", "successful-input-receipt"


def input_fingerprint(
    *,
    repo_root: Path,
    profile_name: str,
    step_name: str,
    area_id: str | None,
    suites: list[str],
    sifr_binary: Path,
    cache_paths: tuple[Path, ...],
    environment: dict[str, str],
) -> tuple[str, bool, str]:
    tracked_state = command_output(
        ["git", "status", "--porcelain", "--untracked-files=no"], repo_root
    )
    source_commit = command_output(["git", "rev-parse", "HEAD"], repo_root)
    cargo_lock_digest = file_digest(repo_root / "Cargo.lock")
    rustc_version = command_output(["rustc", "-vV"], repo_root)
    binary_digest = file_digest(sifr_binary)
    unavailable_inputs = "unavailable" in {
        source_commit,
        rustc_version,
    } or "missing" in {
        cargo_lock_digest,
        binary_digest,
    }
    eligible = tracked_state == "" and not unavailable_inputs
    payload = {
        "profile": profile_name,
        "step": step_name,
        "area": area_id,
        "source_commit": source_commit,
        "tracked_state": tracked_state,
        "suites": suites,
        "cargo_lock_sha256": cargo_lock_digest,
        "rustc": rustc_version,
        "python": sys.version,
        "sifr_binary_sha256": binary_digest,
        "cache_paths": [str(path.resolve()) for path in cache_paths],
        "build_environment": {
            key: environment.get(key)
            for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTUP_TOOLCHAIN",
                        "RUSTC", "CARGO_BUILD_TARGET", "SIFR_CACHE_DIR")
        },
    }
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    return (
        hashlib.sha256(encoded).hexdigest(),
        eligible,
        "eligible"
        if eligible
        else "input-unavailable"
        if unavailable_inputs
        else "tracked-worktree-dirty",
    )


def selected_area_id(step_name: str) -> str | None:
    # Profile steps carry an area_ prefix; area selections and caches use the
    # manifest identifier. Keep this translation at one boundary.
    return step_name.removeprefix("area_") if step_name.startswith("area_") else None


def selected_suites(profile: dict[str, Any], area_id: str | None) -> list[str]:
    if area_id is None:
        return []
    return [
        str(suite)
        for selection in profile.get("selected_areas", [])
        if isinstance(selection, dict) and selection.get("area") == area_id
        for suite in selection.get("suites", [])
    ]


def required_cache_paths(
    repo_root: Path, area_id: str | None, binary: Path, environment: dict[str, str]
) -> tuple[Path, ...]:
    if area_id == "runtime_platform":
        target = Path(environment.get("CARGO_TARGET_DIR",
                                      repo_root / "target" / "runtime_platform" / "cargo-target"))
        if not target.is_absolute():
            target = repo_root / target
        return (binary, target / "debug")
    if area_id == "python_interop":
        return (binary, repo_root / "target" / "python" / "debug")
    return (binary,)


def cache_path_available(path: Path) -> bool:
    if path.is_file():
        return True
    if not path.is_dir():
        return False
    try:
        return next(path.iterdir(), None) is not None
    except OSError:
        return False


def file_digest(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError:
        return "missing"


def command_output(command: list[str], cwd: Path) -> str:
    try:
        result = subprocess.run(
            command,
            cwd=cwd,
            text=True,
            capture_output=True,
            check=False,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired):
        return "unavailable"
    return result.stdout.strip() if result.returncode == 0 else "unavailable"


def atomic_write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=path.parent, delete=False
    ) as handle:
        handle.write(encoded)
        temporary = Path(handle.name)
    os.replace(temporary, path)


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-step-budget-self-test-") as directory:
        root = Path(directory)
        receipt = root / "receipt.json"
        required = root / "cache"
        required.mkdir()
        (required / "artifact").write_text("cached\n", encoding="utf-8")
        state, reason = classify_receipt(
            receipt_path=receipt, fingerprint="a" * 64, required_paths=(required,)
        )
        if (state, reason) != ("cold", "receipt-missing"):
            raise AssertionError("missing cache receipt was not classified cold")
        context = StepBudgetContext(
            name="python_interop",
            budget_ms=1_200_000,
            enforcement="blocking",
            cache_state="cold",
            cache_reason=reason,
            cache_fingerprint="a" * 64,
            receipt_path=receipt,
            receipt_eligible=True,
            required_cache_paths=(required,),
        )
        record_step_success(context, 300_000)
        if classify_receipt(
            receipt_path=receipt, fingerprint="a" * 64, required_paths=(required,)
        ) != (
            "warm",
            "successful-input-receipt",
        ):
            raise AssertionError(
                "successful exact-input receipt was not classified warm"
            )
        if classify_receipt(
            receipt_path=receipt, fingerprint="b" * 64, required_paths=(required,)
        ) != (
            "cold",
            "input-changed",
        ):
            raise AssertionError("changed input did not invalidate the cache receipt")
        receipt.write_text("{\n", encoding="utf-8")
        if classify_receipt(
            receipt_path=receipt, fingerprint="a" * 64, required_paths=(required,)
        ) != (
            "cold",
            "receipt-invalid",
        ):
            raise AssertionError("invalid cache receipt was not classified cold")
        record_step_success(context, 300_000)
        (required / "artifact").unlink()
        if classify_receipt(
            receipt_path=receipt, fingerprint="a" * 64, required_paths=(required,)
        ) != (
            "cold",
            "required-cache-missing",
        ):
            raise AssertionError(
                "missing cache artifacts did not invalidate the receipt"
            )
        # Exercise the profile step spelling rather than the area spelling.
        # The selected suites must enter the receipt identity and the Python
        # cache must be required before an old successful receipt is warm.
        binary = root / "target" / "debug" / "sifr"
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"compiler")
        (root / "Cargo.lock").write_text("lock", encoding="utf-8")
        python_cache = root / "target" / "python" / "debug"
        python_cache.mkdir(parents=True)
        (python_cache / "artifact").write_text("cached", encoding="utf-8")
        profile = {
            "step_budgets": {
                "area_python_interop": {
                    "warm_budget_ms": 600_000,
                    "cold_budget_ms": 1_200_000,
                    "enforcement": "blocking",
                }
            },
            "selected_areas": [
                {"area": "python_interop", "suites": ["tier1"]}
            ],
        }
        env = {"SIFR_GCQ_BIN": str(binary)}

        def stable_command(command: list[str], _cwd: Path) -> str:
            if command[:2] == ["git", "status"]:
                return ""
            return "fixed-input"

        with patch(__name__ + ".command_output", side_effect=stable_command):
            first = prepare_step_budget(
                repo_root=root, profile=profile, profile_name="create-pr",
                name="area_python_interop", env=env,
            )
            assert first is not None
            if first.area_id != "python_interop" or first.receipt_path is None:
                raise AssertionError("Python interop area identity was not normalized")
            if first.receipt_path.name != "python_interop.json":
                raise AssertionError("Python interop receipt used the step prefix")
            if first.required_cache_paths != (binary, python_cache):
                raise AssertionError("Python interop cache paths were not selected")
            if (first.cache_state, env.get("SIFR_PYTHON_INTEROP_CACHE_STATE")) != ("cold", "cold"):
                raise AssertionError("cold Python interop state was not exported")
            record_step_success(first, 900_000)
            warm = prepare_step_budget(
                repo_root=root, profile=profile, profile_name="create-pr",
                name="area_python_interop", env=env,
            )
            assert warm is not None
            if (warm.cache_state, env.get("SIFR_PYTHON_INTEROP_CACHE_STATE")) != ("warm", "warm"):
                raise AssertionError("warm Python interop state was not exported")
            profile["selected_areas"][0]["suites"] = ["tier2"]
            changed = prepare_step_budget(
                repo_root=root, profile=profile, profile_name="create-pr",
                name="area_python_interop", env=env,
            )
            assert changed is not None
            if changed.cache_fingerprint == warm.cache_fingerprint or (
                changed.cache_state, changed.cache_reason
            ) != ("cold", "input-changed"):
                raise AssertionError("changed Python interop suites reused a receipt")
            profile["selected_areas"][0]["suites"] = ["tier1"]
            (python_cache / "artifact").unlink()
            missing = prepare_step_budget(
                repo_root=root, profile=profile, profile_name="create-pr",
                name="area_python_interop", env=env,
            )
            assert missing is not None
            if (missing.cache_state, missing.cache_reason) != (
                "cold", "required-cache-missing"
            ) or env.get("SIFR_PYTHON_INTEROP_CACHE_STATE") != "cold":
                raise AssertionError("missing Python interop cache was classified warm")
        output = io.StringIO()
        with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
            overrun_status = enforce_step_budget(context, 1_200_001)
        if overrun_status != 124:
            raise AssertionError("cold blocking budget did not reject an overrun")
        from .reports import parse_log

        log = root / "lane.log"
        log.write_text(
            "[sifr-lane-step-cache] name=python_interop state=cold "
            "reason=receipt-missing fingerprint=aaaaaaaaaaaaaaaa\n"
            "[sifr-lane-step] name=python_interop elapsed_ms=900000 status=pass\n"
            "[sifr-lane-step-budget] name=python_interop elapsed_ms=900000 "
            "budget_ms=1200000 enforcement=blocking status=pass\n",
            encoding="utf-8",
        )
        parsed = parse_log(log)
        expected_cache = {
            "state": "cold",
            "reason": "receipt-missing",
            "fingerprint": "aaaaaaaaaaaaaaaa",
        }
        if parsed["lane_step_cache"].get("python_interop") != expected_cache:
            raise AssertionError("lane report lost the cache classification")
