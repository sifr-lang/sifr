from __future__ import annotations

import shlex
import subprocess
import sys
import sysconfig
import tempfile
from pathlib import Path


MODULE_NAME = "_sifr_cffi_probe"
COMPILE_FAILURE = "sifr_intentional_compile_failure"
INVOKE_PROBE = """
import importlib.util
import sys
from pathlib import Path

extension = Path(sys.argv[1]).resolve()
spec = importlib.util.spec_from_file_location("_sifr_cffi_probe", extension)
if spec is None or spec.loader is None:
    raise RuntimeError("could not load the generated CFFI extension")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
if Path(module.__file__).resolve() != extension:
    raise RuntimeError("loaded a different CFFI extension")
if module.lib.sifr_add(19, 23) != 42:
    raise RuntimeError("generated native addition returned the wrong value")
try:
    module.lib.sifr_add(1 << 80, 0)
except OverflowError:
    pass
else:
    raise RuntimeError("CFFI accepted an out-of-range long long argument")
if module.lib.sifr_add(-19, 23) != 4:
    raise RuntimeError("generated native addition failed after argument rejection")
print("cffi-native-invocation=42; overflow=rejected; recovery=4")
"""


class ProbeFailure(RuntimeError):
    def __init__(self, stage: str, detail: str) -> None:
        super().__init__(f"{stage} failed: {detail}")
        self.stage = stage
        self.detail = detail


def run_checked(stage: str, command: list[str], root: Path) -> str:
    completed = subprocess.run(
        command, cwd=root, text=True, capture_output=True, check=False
    )
    if completed.returncode != 0:
        raise ProbeFailure(stage, (completed.stdout + completed.stderr).strip())
    return completed.stdout


def config_flags(name: str) -> list[str]:
    value = sysconfig.get_config_var(name)
    if not isinstance(value, str):
        raise RuntimeError(f"Python build configuration is missing {name}")
    return shlex.split(value)


def compile_and_invoke(root: Path, *, invalid_source: bool) -> None:
    cdef = root / "probe.cdef"
    csrc = root / "probe.c"
    output = root / "probe_module.c"
    suffix = sysconfig.get_config_var("EXT_SUFFIX")
    if not isinstance(suffix, str) or not suffix:
        raise RuntimeError("Python build configuration is missing EXT_SUFFIX")
    extension = root / f"{MODULE_NAME}{suffix}"
    cdef.write_text("long long sifr_add(long long, long long);\n", encoding="utf-8")
    csrc.write_text(
        (f"#error {COMPILE_FAILURE}\n" if invalid_source else "")
        + "long long sifr_add(long long left, long long right) "
        "{ return left + right; }\n",
        encoding="utf-8",
    )
    command = [
        sys.executable,
        "-m",
        "cffi.gen_src",
        "read-sources",
        MODULE_NAME,
        str(cdef),
        str(csrc),
        str(output),
    ]
    run_checked("cffi.gen_src", command, root)
    generated = output.read_text(encoding="utf-8")
    if MODULE_NAME not in generated or "sifr_add" not in generated:
        raise RuntimeError("cffi.gen_src output omitted the module or function")
    # Compile the CLI's exact output, using this interpreter's extension ABI.
    object_file = root / "probe_module.o"
    includes = {
        sysconfig.get_path("include"),
        sysconfig.get_path("platinclude"),
    }
    run_checked(
        "compile",
        config_flags("CC")
        + config_flags("CFLAGS")
        + config_flags("CCSHARED")
        + [f"-I{path}" for path in sorted(includes)]
        + ["-c", str(output), "-o", str(object_file)],
        root,
    )
    run_checked(
        "link",
        config_flags("LDSHARED")
        + config_flags("LDFLAGS")
        + [str(object_file), "-o", str(extension)],
        root,
    )
    # A separate interpreter releases the loaded module before directory cleanup.
    invocation = run_checked(
        "invoke", [sys.executable, "-c", INVOKE_PROBE, str(extension)], root
    )
    if invocation.strip() != "cffi-native-invocation=42; overflow=rejected; recovery=4":
        raise RuntimeError("generated CFFI extension omitted invocation evidence")


def run_probe(*, invalid_source: bool = False) -> None:
    temporary = tempfile.TemporaryDirectory(prefix="sifr-cffi-gen-src-")
    root = Path(temporary.name)
    try:
        with temporary:
            compile_and_invoke(root, invalid_source=invalid_source)
    finally:
        if root.exists():
            raise RuntimeError(f"CFFI probe left temporary artifacts at {root}")


def main() -> int:
    run_probe()
    try:
        run_probe(invalid_source=True)
    except ProbeFailure as error:
        if error.stage != "compile" or COMPILE_FAILURE not in error.detail:
            raise
    else:
        raise RuntimeError("CFFI probe accepted intentionally invalid generated C")
    print(
        "python crypto ABI features ok: cffi-gen-src=compiled-loaded-invoked; "
        "overflow=rejected; compile-error=rejected; cleanup=success-and-error"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
