#!/usr/bin/env python3
"""Build all supported postgresql components with actual source/tool provenance."""

from build_sql_components import build


if __name__ == "__main__":
    raise SystemExit(build("postgresql"))
