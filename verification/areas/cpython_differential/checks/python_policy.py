from __future__ import annotations

import platform
import sysconfig
from pathlib import Path

import tomllib

EXPECTED_PYTHON_VERSION = "3.14.7"
EXPECTED_REQUIREMENT = f"=={EXPECTED_PYTHON_VERSION}"


def validate_canonical_python(pyproject_path: Path) -> list[str]:
    pyproject = tomllib.loads(pyproject_path.read_text(encoding="utf-8"))
    requirement = pyproject["project"]["requires-python"]
    if requirement != EXPECTED_REQUIREMENT:
        return [
            (
                f"unsupported requires-python policy {requirement!r}; "
                f"expected {EXPECTED_REQUIREMENT!r}"
            )
        ]
    implementation = platform.python_implementation()
    version = platform.python_version()
    if implementation != "CPython" or version != EXPECTED_PYTHON_VERSION:
        return [
            (
                f"CPython {EXPECTED_PYTHON_VERSION} is required, "
                f"found {implementation} {version}"
            )
        ]
    if sysconfig.get_config_var("Py_GIL_DISABLED"):
        return [
            "the CPython differential oracle requires the canonical GIL-enabled build"
        ]
    return []
