#!/usr/bin/env python3
"""Observe the original selected Cargo compiler graph; preserve artifact production."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def wrapper():
    compiler, *args = sys.argv[1:]
    crate = args[args.index("--crate-name") + 1] if "--crate-name" in args else None
    selected = crate in {"syn", "sifr_codegen"} and "--target" in args and args[args.index("--target") + 1] == "x86_64-unknown-linux-gnu"
    if selected:
        if "RUSTC_BOOTSTRAP" in os.environ:
            raise RuntimeError("bootstrap leaked into original selected compiler")
        store = Path(os.environ["SIFR_BUILTIN_DEPENDENCY_INVOCATIONS"])
        value = {"schema":"sifr-maintainability-original-invocation-v1", "compiler":compiler,"args":args,"cwd":os.getcwd(),"environment":dict(os.environ)}
        key = hashlib.sha256(encoded(value)).hexdigest()
        path = store / (crate + "-" + key + ".json")
    result = subprocess.run([compiler, *args])
    if selected:
        value["compiler_status"] = result.returncode
        temporary = path.with_suffix(".tmp")
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, "wb") as output:
            output.write(encoded(value))
        os.replace(temporary, path)
    return result.returncode


if __name__ == "__main__":
    sys.exit(wrapper())
