"""Prepare and verify the exact environment consumed by Rust/Python fixtures."""
from pathlib import Path
import json
import sys

from env import discover_paths, require_canonical_python


def main():
    area = discover_paths().area_root
    version = require_canonical_python(area)
    expected = area / ".venv"
    if Path(sys.prefix).resolve() != expected.resolve() or sys.prefix == sys.base_prefix:
        raise SystemExit(f"Python fixture environment must be {expected}, found {sys.prefix}")
    print(json.dumps({"python_version": version, "interpreter": sys.executable,
                      "environment": sys.prefix, "status": "prepared"}, sort_keys=True))


if __name__ == "__main__":
    main()
