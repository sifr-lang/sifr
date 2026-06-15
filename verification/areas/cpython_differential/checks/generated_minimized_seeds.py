"""Run the deterministic generated CPython differential seed subset."""

from __future__ import annotations

from generated_suite import main


if __name__ == "__main__":
    raise SystemExit(main("generated_minimized_seeds"))
