from __future__ import annotations

import platform
import re
import sysconfig
from pathlib import Path

import tomllib


def validate_canonical_python(pyproject_path: Path) -> list[str]:
    pyproject = tomllib.loads(pyproject_path.read_text(encoding="utf-8"))
    requirement = pyproject["project"]["requires-python"]
    if (
        not isinstance(requirement, str)
        or re.fullmatch(r"==\d+\.\d+\.\d+", requirement) is None
    ):
        return [
            (
                f"unsupported requires-python policy {requirement!r}; "
                "expected an exact ==X.Y.Z pin"
            )
        ]
    expected = requirement.removeprefix("==")
    implementation = platform.python_implementation()
    version = platform.python_version()
    if implementation != "CPython" or version != expected:
        return [f"CPython {expected} is required, found {implementation} {version}"]
    if sysconfig.get_config_var("Py_GIL_DISABLED"):
        return [
            "the CPython differential oracle requires the canonical GIL-enabled build"
        ]
    return []
