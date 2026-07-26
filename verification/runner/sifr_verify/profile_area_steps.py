"""Execute selected verification areas and require complete result evidence."""

from __future__ import annotations

from collections.abc import Callable
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


__all__ = ["AreaResultError", "run_selected_area", "validate_area_result"]
