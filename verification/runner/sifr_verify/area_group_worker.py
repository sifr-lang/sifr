"""Internal fresh-process consumer of an already prepared area allocation."""
from __future__ import annotations

import argparse
import os

from .area_group_policy import PAIR
from .build_allocation import E2E_ENV, positive
from .profile_runner import ProfileRunner
from .process_ownership import session_environment


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("create-pr", "merge"), required=True)
    parser.add_argument("--area", choices=tuple(PAIR), required=True)
    parser.add_argument("--no-fail-fast", action="store_true")
    args = parser.parse_args(argv)
    environment = session_environment(os.environ)
    os.environ.update(environment)
    slots = positive(environment.get("SIFR_VERIFY_PROCESS_SLOTS", ""), "area process slots")
    for name in ("CARGO_BUILD_JOBS", "RAYON_NUM_THREADS", "RUST_TEST_THREADS", *E2E_ENV.values(),
                 "OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS"):
        if positive(environment.get(name, ""), name) > slots:
            raise ValueError(f"{name} exceeds the admitted area allocation")
    if (int(environment["SIFR_E2E_RUST_JOBS"])
            * int(environment["SIFR_E2E_CARGO_BUILD_JOBS"]) > slots):
        raise ValueError("nested native compilation exceeds the admitted allocation")
    runner = ProfileRunner(args.profile, ["--no-fail-fast"] if args.no_fail_fast else [],
                           correctness_only=True)
    # The parent owns admission. Construction resolves canonical selection, but
    # must not replace its prepared context or partition with a new full budget.
    runner.env = environment
    selection = next(item for item in runner.profile["selected_areas"] if item["area"] == args.area)
    runner.run_area(args.area, list(selection["suites"]))


if __name__ == "__main__":
    main()
