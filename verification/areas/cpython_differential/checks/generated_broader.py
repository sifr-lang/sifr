"""Run the broader nightly/release generated CPython differential corpus."""

from __future__ import annotations

from generated_suite import main


if __name__ == "__main__":
    raise SystemExit(main("generated_broader"))
