"""Audited runtime closure for isolated, stdlib-only Python validation recipes.

Unknown dependencies disable reuse. Compiler/runtime adapters and external
services require their own declarations; this module cannot bless them.
"""
from __future__ import annotations

import json
from pathlib import Path
import sys
import sysconfig

from .execution_identity import EvidenceError, artifact_identity, digest
from .process_execution import execute


def isolated_python(script: str) -> list[str]:
    if not sys.platform.startswith("linux"):
        raise EvidenceError("stdlib-only checkpoint recipe is declared for Linux")
    # -I/-S remove site packages, import-path/environment injection. The unusable
    # pycache prefix plus -B forces source reads without consuming mutable .pyc.
    return [sys.executable, "-I", "-S", "-B", "-X", "pycache_prefix=/dev/null", script]


def stdlib_identity(script: str, environment: dict[str, str], *, cwd: Path) -> dict:
    if not sys.platform.startswith("linux"):
        raise EvidenceError("no audited checkpoint loader closure on this host")
    root = Path(sysconfig.get_path("stdlib")).resolve(strict=True)
    files = []
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root)
        if set(relative.parts).intersection({"site-packages", "__pycache__"}):
            continue
        if path.is_file():
            files.append(artifact_identity(path))
    # The caller separately pins the audited recipe's exact script digest. Its
    # top level contains declarations/imports only; run_name does not call main.
    # Observe libraries after those imports in the actual isolated interpreter.
    # Dormant optional extensions are not a native runtime dependency of this
    # recipe; missing optional Tk/etc must not be mistaken for a loaded library.
    program = (
        "import json,runpy,sys\nfrom pathlib import Path\n"
        "runpy.run_path(sys.argv[1],run_name='checkpoint_runtime_probe')\n"
        "paths=set()\n"
        "for line in Path('/proc/self/maps').read_text().splitlines():\n"
        " fields=line.split(None,5)\n"
        " if len(fields)==6 and fields[5].startswith('/'):\n"
        "  paths.add(fields[5])\n"
        "print(json.dumps(sorted(paths)))\n"
    )
    command = [*isolated_python("-c"), program, script]
    result = execute(command, cwd=cwd, env=environment, deadline_seconds=30)
    if result.returncode or result.cause != "exit" or result.truncated:
        raise EvidenceError("isolated checkpoint runtime probe failed")
    try:
        libraries = json.loads(result.stdout)
    except ValueError as error:
        raise EvidenceError("unknown checkpoint loader observation") from error
    if (not isinstance(libraries, list) or not libraries or any(
            not isinstance(path, str) or not Path(path).is_absolute() or path.endswith(" (deleted)")
            for path in libraries)):
        raise EvidenceError("checkpoint native loader dependencies are unknown")
    return {"protocol": "linux-isolated-stdlib-source-v1", "stdlib_root": str(root),
            "stdlib_files": files, "native_dependencies": [artifact_identity(Path(path)) for path in sorted(set(libraries))],
            "loader_environment": {name: digest(environment[name]) if name in environment else None for name in
                                   ("LD_LIBRARY_PATH", "LD_PRELOAD", "LD_AUDIT", "LANG", "LC_ALL", "LC_CTYPE", "TZ")}}
