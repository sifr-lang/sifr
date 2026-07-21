from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "python_interop"
COMMON_ROOT = REPO_ROOT / "verification" / "areas" / "common"
sys.path.insert(0, str(COMMON_ROOT))

from sifr_binary import resolve_sifr_binary  # noqa: E402


def main() -> int:
    binary = resolve_sifr_binary(REPO_ROOT)
    root = REPO_ROOT / "target" / "verification" / "areas" / "python_interop" / "readonly"
    if root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True)

    library = create_package(root / "library", "readonly-library", application=False)
    library_before = snapshot(library)
    check = run_json(binary, library, "python", "check", "--json")
    require(check["application"] is False, "library report must be deferred")
    require(check["environment"]["status"] == "deferred", "library environment must defer")
    require(check["trust"] == "deferred-to-final-application", "library trust must defer")
    require(check["targets"] == [{"target": "math.sqrt", "status": "deferred"}], "library target must defer")
    first_doctor = run(binary, library, "python", "doctor", "--json")
    second_doctor = run(binary, library, "python", "doctor", "--json")
    require(first_doctor.stdout == second_doctor.stdout, "doctor output must be deterministic")
    doctor = json.loads(first_doctor.stdout)
    require(
        doctor["suggestions"][0]["patch"]
        == '@@ [python]\n+venv = ".venv"\n+pyproject = "pyproject.toml"\n+lock = "uv.lock"\n@@ [trust]\n+python = ["math"]',
        "doctor patch drifted",
    )
    require(snapshot(library) == library_before, "library inspection mutated its package")

    application = create_package(root / "application", "readonly-application", application=True)
    application_before = snapshot(application)
    app_check = run_json(binary, application, "python", "check", "--json")
    normal = run(binary, application, "check", "src/main.sifr", "--frozen")
    require(normal.returncode == 0, f"normal check failed: {normal.stderr}")
    require(app_check["application"] is True, "application report must resolve")
    require(app_check["environment"]["status"] == "resolved", "application environment must resolve")
    require(app_check["trust"] == "verified", "application trust must verify")
    require(
        app_check["targets"]
        == [
            {"target": "math.ceil", "status": "verified"},
            {"target": "math.sqrt", "status": "verified"},
        ],
        "every application target must verify",
    )
    require(snapshot(application) == application_before, "application inspection mutated its package")

    source = application / "src" / "main.sifr"
    source.write_text(source.read_text(encoding="utf-8") + "\n# snapshot change\n", encoding="utf-8")
    changed_before = snapshot(application)
    changed = run_json(binary, application, "python", "check", "--json")
    require(changed["source_digest"] != app_check["source_digest"], "source digest ignored source bytes")
    require(snapshot(application) == changed_before, "snapshot check mutated its package")

    source.write_text(application_source("math.not_a_real_target"), encoding="utf-8")
    invalid_before = snapshot(application)
    python_failure = run(binary, application, "python", "check", expected=1)
    normal_failure = run(binary, application, "check", "src/main.sifr", "--frozen", expected=1)
    require("SIFR-PYIMP-0001" in python_failure.stderr, "python check target diagnostic drifted")
    require("SIFR-PYIMP-0001" in normal_failure.stderr, "normal check target diagnostic drifted")
    require(snapshot(application) == invalid_before, "failure checks mutated their package")

    print("python interop read-only check/doctor ok: deferred=1 resolved=1 parity=2 mutations=0")
    return 0


def create_package(root: Path, name: str, *, application: bool) -> Path:
    (root / "src").mkdir(parents=True)
    (root / "src" / "lib.rs").write_text("// pure Sifr package marker\n", encoding="utf-8")
    sifr_source = root / "src" / ("main.sifr" if application else "__init__.sifr")
    sifr_source.write_text(application_source("math.sqrt"), encoding="utf-8")
    if application:
        secondary = root / "src" / "bin" / "secondary.sifr"
        secondary.parent.mkdir()
        secondary.write_text(
            "from sifr.python import PythonError\n\n\n"
            "@python(math.ceil)\n"
            "def ceil(value: float) -> Result[int, PythonError]: ...\n\n\n"
            "def main():\n    pass\n",
            encoding="utf-8",
        )
    cargo_name = f"sifr-python-{name}"
    (root / "Cargo.toml").write_text(
        f'[package]\nname = "{cargo_name}"\nversion = "0.1.0"\nedition = "2024"\n\n'
        '[package.metadata.sifr]\nmanifest = "sifr.toml"\n\n[workspace]\n',
        encoding="utf-8",
    )
    (root / "Cargo.lock").write_text(
        "# This file is automatically @generated by Cargo.\n"
        "# It is not intended for manual editing.\n"
        f'version = 4\n\n[[package]]\nname = "{cargo_name}"\nversion = "0.1.0"\n',
        encoding="utf-8",
    )
    manifest = (
        f'[package]\nname = "{name.replace("-", "_")}"\nedition = "2026"\n'
        'sifr-version = ">=0.3,<0.4"\n\n[source]\nroot = "src"\n'
    )
    if application:
        manifest += (
            '\n[python]\nvenv = ".venv"\npyproject = "pyproject.toml"\nlock = "uv.lock"\n'
            '\n[trust]\npython = ["math"]\n'
        )
        (root / ".venv").symlink_to(AREA_ROOT / ".venv", target_is_directory=True)
        (root / "pyproject.toml").symlink_to(AREA_ROOT / "pyproject.toml")
        (root / "uv.lock").symlink_to(AREA_ROOT / "uv.lock")
    (root / "sifr.toml").write_text(manifest, encoding="utf-8")
    return root


def application_source(target: str) -> str:
    return (
        "from sifr.python import PythonError\n\n\n"
        f"@python({target})\n"
        "def sqrt(value: float) -> Result[float, PythonError]: ...\n\n\n"
        "def main():\n    pass\n"
    )


def run_json(binary: Path, cwd: Path, *arguments: str) -> dict[str, object]:
    return json.loads(run(binary, cwd, *arguments).stdout)


def run(
    binary: Path,
    cwd: Path,
    *arguments: str,
    expected: int = 0,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [str(binary), *arguments],
        cwd=cwd,
        text=True,
        capture_output=True,
        check=False,
        timeout=120,
    )
    require(
        result.returncode == expected,
        f"{' '.join(arguments)} exited {result.returncode}, expected {expected}: {result.stderr}",
    )
    return result


def snapshot(root: Path) -> dict[str, tuple[str, bytes]]:
    files: dict[str, tuple[str, bytes]] = {}
    for path in sorted(root.rglob("*")):
        relative = str(path.relative_to(root))
        if path.is_symlink():
            files[relative] = ("link", str(path.readlink()).encode())
        elif path.is_file():
            files[relative] = ("file", path.read_bytes())
    return files


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


if __name__ == "__main__":
    raise SystemExit(main())
