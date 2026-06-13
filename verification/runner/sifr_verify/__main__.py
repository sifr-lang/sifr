"""Command line entrypoint for the Sifr verification runner foundation."""

from __future__ import annotations

import argparse
import sys

from . import areas, profiles, reports
from .errors import VerificationError
from .selftest import run_all


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="Run runner foundation self-tests.")
    parser.add_argument("--list-areas", action="store_true", help="List discovered verification areas as JSON.")
    parser.add_argument("--profile", help="Reserved profile selector for the facade cutover.")
    parser.add_argument("--case", help="Reserved case selector for failure reproduction.")
    parser.add_argument("command", nargs="?", help="Subcommand: profiles or reports.")
    parser.add_argument("command_args", nargs=argparse.REMAINDER)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.self_test:
            for name in run_all():
                print(f"verification runner self-test: {name}: pass")
            return 0
        if args.command == "profiles":
            return profiles.run_command(args.command_args)
        if args.command == "reports":
            return reports.main(args.command_args)
        if args.command == "areas":
            return areas.run_command(args.command_args)
        if args.list_areas:
            areas.print_list()
            return 0
        if args.profile:
            print(
                "profile execution has not migrated to sifr_verify yet; "
                "use scripts/run_all_tests.sh as the authoritative facade",
                file=sys.stderr,
            )
            return 2
        print("nothing to do; pass --self-test or --list-areas", file=sys.stderr)
        return 2
    except VerificationError as exc:
        print(f"sifr_verify: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
