"""Command-line options for benchmark execution and reference capture."""

import argparse
import os


def parse_args(default_manifest, default_output_root, default_trend_baselines):
    parser = argparse.ArgumentParser(description="local benchmark runner.")
    parser.add_argument("--compiler-lane", choices=["contributor-dev", "product-installed-optimized"], default="contributor-dev")
    parser.add_argument("--compiler-receipt", default="")
    parser.add_argument("--manifest", default=str(default_manifest))
    parser.add_argument("--output-root", default=str(default_output_root))
    parser.add_argument("--groups", default="")
    parser.add_argument("--case", action="append", default=[])
    parser.add_argument("--case-limit", type=int, default=0)
    parser.add_argument(
        "--sample-scale", choices=["manifest", "smoke"], default="manifest"
    )
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--capture-baseline", action="store_true")
    parser.add_argument("--capture-work-baseline", action="store_true")
    parser.add_argument("--baseline-output", default="")
    parser.add_argument("--work-budget-output", default="")
    parser.add_argument("--capture-trend-baseline", action="store_true")
    parser.add_argument("--trend-baseline-output", default="")
    parser.add_argument("--reference-approval", default="")
    parser.add_argument("--trend-baselines", default=str(default_trend_baselines))
    parser.add_argument("--trend-json-out", default="")
    parser.add_argument("--json-out", default="")
    parser.add_argument("--invocation-id", default="")
    parser.add_argument("--require-controlled-host", action="store_true")
    parser.add_argument(
        "--controlled-host-mode",
        choices=["latency", "work"],
        default="latency",
    )
    parser.add_argument("--controlled-host-timeout-seconds", type=float, default=180.0)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--reference-profile", default=os.environ.get("SIFR_PERFORMANCE_REFERENCE", ""))
    parser.add_argument("--capture-reference-profile", default="")
    parser.add_argument("--reference-compiler-commit", default="")
    return parser.parse_args()
