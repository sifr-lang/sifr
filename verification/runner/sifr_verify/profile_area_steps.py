"""Execute selected verification areas and require complete result evidence."""

from __future__ import annotations

from collections.abc import Callable
import json
from pathlib import Path

from .paths import REPO_ROOT
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
    command_runner: Callable[[list[str]], None],
) -> Path:
    """Bound each suite process and certify the complete selection afterward."""
    manifest_path = REPO_ROOT / "verification" / "areas" / "python_interop" / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    canonical_suites = [suite["name"] for suite in manifest["suites"]
                        if suite["name"] in suites]
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
    base = [
        "uv", "run", "--project", "verification", "--locked", "python",
        "verification/areas/python_interop/runner.py",
    ]
    for suite, part in zip(canonical_suites, part_paths, strict=True):
        command_runner([*base, "--suite", suite, "--defer-certification",
                        "--result-json", str(part.relative_to(REPO_ROOT))])
    combined = [*base]
    for suite in canonical_suites:
        combined.extend(["--suite", suite])
    for part in part_paths:
        combined.extend(["--combine-suite-result", str(part)])
    combined.extend(["--result-json", str(result_path.relative_to(REPO_ROOT))])
    command_runner(combined)
    payload = validate_area_result(result_path, area="python_interop", expected_suites=suites)
    certification = payload.get("compiled_certification")
    if not isinstance(certification, dict):
        raise AreaResultError("python_interop combined result lacks compiled certification")
    return result_path


__all__ = ["AreaResultError", "run_selected_area", "run_segmented_python_interop", "validate_area_result"]
