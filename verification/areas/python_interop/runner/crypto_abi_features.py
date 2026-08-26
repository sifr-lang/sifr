from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="sifr-cffi-gen-src-") as directory:
        root = Path(directory)
        cdef = root / "probe.cdef"
        csrc = root / "probe.c"
        output = root / "probe_module.c"
        cdef.write_text("long long sifr_add(long long, long long);\n", encoding="utf-8")
        csrc.write_text(
            "long long sifr_add(long long left, long long right) "
            "{ return left + right; }\n",
            encoding="utf-8",
        )
        command = [
            sys.executable,
            "-m",
            "cffi.gen_src",
            "read-sources",
            "_sifr_cffi_probe",
            str(cdef),
            str(csrc),
            str(output),
        ]
        completed = subprocess.run(command, text=True, capture_output=True, check=False)
        if completed.returncode != 0:
            raise RuntimeError(f"cffi.gen_src failed: {completed.stderr.strip()}")
        generated = output.read_text(encoding="utf-8")
        if "_sifr_cffi_probe" not in generated or "sifr_add" not in generated:
            raise RuntimeError("cffi.gen_src output omitted the module or function")
    print("python crypto ABI features ok: cffi-gen-src=generated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
