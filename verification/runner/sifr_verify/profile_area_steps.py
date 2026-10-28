"""Execute selected verification areas and require complete result evidence."""

from __future__ import annotations

from collections.abc import Callable
import json
from pathlib import Path

from .areas import area_by_name
from .paths import REPO_ROOT
from .process_execution import Outcome
from .profile_commands import CommandFailed, verification_python_command
from .profile_results import AreaResultError, validate_area_result


def run_selected_area(
    *,
    area: str,
    suites: list[str],
    profile_name: str,
    result_slug: str,
    command_builder: Callable[..., list[str]],
    command_runner: Callable[[list[str]], None],
) -> Path:
    """Run one selected area and validate the exact emitted suite set."""
    result_path = (
        REPO_ROOT
        / "target"
        / "verification"
        / "areas"
        / f"{result_slug}-{profile_name}-results.json"
    )
    result_path.unlink(missing_ok=True)
    args = ["--area", area]
    for suite in suites:
        args.extend(["--suite", suite])
    args.extend(["--result-json", str(result_path.relative_to(REPO_ROOT))])
    command_runner(command_builder(*args))
    validate_area_result(result_path, area=area, expected_suites=suites)
    return result_path


def run_segmented_python_interop(
    *, suites: list[str], profile_name: str,
    command_runner: Callable[[list[str]], Outcome],
    no_fail_fast: bool = False,
) -> Path:
    """Bound each suite process and certify the complete selection afterward."""
    try:
        manifest_path = area_by_name("python_interop").manifest_path
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        canonical_suites = [suite["name"] for suite in manifest["suites"]
                            if suite["name"] in suites]
    except (OSError, ValueError, TypeError, KeyError) as error:
        raise AreaResultError("invalid python_interop area manifest") from error
    if len(canonical_suites) != len(suites) or set(canonical_suites) != set(suites):
        raise AreaResultError("python_interop selection differs from its manifest")
    result_root = REPO_ROOT / "target" / "verification" / "areas"
    result_path = result_root / f"python-interop-{profile_name}-results.json"
    part_root = result_root / f"python-interop-{profile_name}-parts"
    part_root.mkdir(parents=True, exist_ok=True)
    result_path.unlink(missing_ok=True)
    part_paths = [part_root / f"{index:02d}.json" for index in range(len(canonical_suites))]
    for part in part_paths:
        part.unlink(missing_ok=True)
    base = verification_python_command("verification/areas/python_interop/runner.py")
    first_failure = None
    outcomes = {suite: "not-run" for suite in canonical_suites}
    try:
        for suite, part in zip(canonical_suites, part_paths, strict=True):
            outcomes[suite] = "incomplete"
            try:
                outcome = command_runner([*base, "--suite", suite, "--defer-certification",
                                          "--result-json", str(part.relative_to(REPO_ROOT))])
                if outcome.cause != "exit" or outcome.truncated or outcome.returncode != 0:
                    raise AreaResultError("python_interop child returned invalid execution evidence")
                validate_area_result(part, area="python_interop", expected_suites=[suite])
            except CommandFailed as error:
                outcomes[suite] = "command-failed"
                if (not no_fail_fast or error.cause not in (None, "exit")
                        or error.returncode < 0 or getattr(getattr(error, "outcome", None), "truncated", False)):
                    raise
                first_failure = first_failure or error
            else:
                outcomes[suite] = "result-passed"
    finally:
        for suite, status in outcomes.items():
            print("[sifr-python-suite] " + json.dumps({"suite": suite, "status": status}), flush=True)
    if first_failure is not None:
        # Retain every individual result. Failed/missing parts cannot enter the
        # passing-result combiner or manufacture compiled certification.
        raise first_failure
    combined = [*base]
    for suite in canonical_suites:
        combined.extend(["--suite", suite])
    for part in part_paths:
        combined.extend(["--combine-suite-result", str(part)])
    combined.extend(["--result-json", str(result_path.relative_to(REPO_ROOT))])
    outcome = command_runner(combined)
    if outcome.cause != "exit" or outcome.truncated or outcome.returncode != 0:
        raise AreaResultError("python_interop combiner returned invalid execution evidence")
    payload = validate_area_result(result_path, area="python_interop", expected_suites=suites)
    certification = payload.get("compiled_certification")
    if not isinstance(certification, dict):
        raise AreaResultError("python_interop combined result lacks compiled certification")
    return result_path


__all__ = ["AreaResultError", "run_selected_area", "run_segmented_python_interop", "validate_area_result"]
