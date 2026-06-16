#!/usr/bin/env python3
"""Run the coverage matrix in strict closeout mode."""

from __future__ import annotations

import os

import coverage_matrix


def main() -> int:
    os.environ["SIFR_COVERAGE_MATRIX_STRICT"] = "1"
    return coverage_matrix.main()


if __name__ == "__main__":
    raise SystemExit(main())
