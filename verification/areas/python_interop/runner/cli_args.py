from __future__ import annotations

import argparse


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run Sifr embedded Python interop verification.")
    parser.add_argument("--group", action="append", default=[], help="Verification group filter.")
    parser.add_argument("--tier", action="append", default=[], help="Package tier filter.")
    parser.add_argument("--gate", action="append", default=[], help="Certification gate filter.")
    parser.add_argument("--package", action="append", default=[], help="Package name filter.")
    parser.add_argument(
        "--report",
        default="../../../target/verification/areas/python_interop/latest.json",
        help="Report path relative to verification/areas/python_interop.",
    )
    parser.add_argument("--self-test", action="store_true", help="Run runner positive and negative self-tests.")
    parser.add_argument("--live-policy", action="store_true", help="Validate live container-runtime policy.")
    parser.add_argument("--live-examples", action="store_true", help="Run testcontainers-backed live examples.")
    parser.add_argument("--dataframe-examples", action="store_true", help="Run full NumPy/pandas/Polars Sifr examples.")
    parser.add_argument(
        "--buffer-examples",
        action="store_true",
        help="Run compiled declaration-first Python buffer examples.",
    )
    parser.add_argument(
        "--arrow-examples",
        action="store_true",
        help="Run compiled declaration-first Arrow C Data Interface examples.",
    )
    parser.add_argument(
        "--dlpack-examples",
        action="store_true",
        help="Run compiled declaration-first DLPack transfer examples.",
    )
    parser.add_argument("--ml-examples", action="store_true", help="Run full torch/scikit-learn Sifr examples.")
    parser.add_argument("--library-examples", action="store_true", help="Run full library-family Sifr examples.")
    parser.add_argument(
        "--async-declaration-examples",
        action="store_true",
        help="Run compiled typed async Python declaration examples.",
    )
    parser.add_argument(
        "--async-context-examples",
        action="store_true",
        help="Run compiled typed async Python context-manager examples.",
    )
    parser.add_argument(
        "--callback-examples",
        action="store_true",
        help="Run compiled typed Python callback examples.",
    )
    return parser.parse_args(argv)
