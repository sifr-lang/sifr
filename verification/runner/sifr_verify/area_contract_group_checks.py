"""Controls for the four explicit contract areas; no general parallel permission."""
import json
from dataclasses import replace
import unittest

from . import area_group_checks as base
from .area_group_policy import LOCAL_AREAS, execution_groups, partition_slots
from .profiles import load_profile


class ContractPolicyTests(unittest.TestCase):
    def test_selection_is_complete_explicit_and_correctness_only(self):
        self.assertEqual([g["id"] for g in execution_groups(load_profile("create-pr"), True)],
                         ["python-quality"], "create-pr does not select distribution")
        for name in ("merge",):
            profile = load_profile(name)
            groups = execution_groups(profile, True)
            group = next(g for g in groups if g["id"] == "local-contracts")
            self.assertEqual(group["areas"], list(LOCAL_AREAS))
            self.assertEqual(execution_groups(profile, False), [])
            for missing in LOCAL_AREAS:
                partial = profile | {"selected_areas": [a for a in profile["selected_areas"]
                                                       if a["area"] != missing]}
                self.assertEqual([g["id"] for g in execution_groups(partial, True)],
                                 ["python-quality"])
        for name in ("nightly", "release", "python-interop-live"):
            self.assertEqual(execution_groups(load_profile(name), True), [])
        for invalid in ([], ["coverage_matrix"] * 2, ["coverage_matrix", "python_interop"],
                        ["diagnostics", "unknown"]):
            with self.assertRaises(ValueError):
                partition_slots(invalid, 4)

    def test_lower_capacity_stays_serial_and_never_exceeds_owner(self):
        for count in (2, 3, 4):
            areas = LOCAL_AREAS[:count]
            for total in (1, 2, 3, 4, 8):
                parts, parallel = partition_slots(areas, total)
                self.assertEqual(parallel, 1 if total < 4 else count)
                self.assertLessEqual(sum(sorted(parts.values(), reverse=True)[:parallel]), total)
                self.assertEqual(set(parts), set(areas))


class ContractExecutionTests(unittest.TestCase):
    def setUp(self):
        self.fixture = base.ExecutionTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.runner = self.fixture.runner
        self.runner.profile["selected_areas"] = [
            {"area": a, "suites": ["probe"]} for a in LOCAL_AREAS]

    def invoke(self):
        return self.fixture.invoke(list(LOCAL_AREAS))

    def test_all_four_real_children_overlap_and_keep_one_slot_each(self):
        for area in LOCAL_AREAS:
            barrier = (f"end=time.monotonic()+5\n"
                       f"while not all((root/a).exists() for a in {LOCAL_AREAS!r}):\n"
                       " if time.monotonic()>end:raise RuntimeError('missing four-way overlap')\n"
                       " time.sleep(.01)\n")
            self.fixture.programs[area] = self.fixture.program(area, extra=barrier)
        self.assertEqual(self.invoke(), 0)
        self.assertEqual(self.runner.completed_steps, {"area_" + a for a in LOCAL_AREAS})
        for area in LOCAL_AREAS:
            environment = json.loads((self.fixture.root / area).read_text())
            for key in ("SIFR_VERIFY_PROCESS_SLOTS", "CARGO_BUILD_JOBS", "RUST_TEST_THREADS",
                        "RAYON_NUM_THREADS", "SIFR_E2E_CARGO_BUILD_JOBS"):
                self.assertEqual(environment[key], "1", (area, key))

    def test_low_capacity_failure_stops_queue_but_continuation_covers_every_area(self):
        self.runner.build_allocation = replace(self.runner.build_allocation, slot_limit=2)
        self.runner.no_fail_fast = False
        self.fixture.programs[LOCAL_AREAS[0]] = self.fixture.program(LOCAL_AREAS[0], code=9)
        self.assertNotEqual(self.invoke(), 0)
        self.assertFalse(any((self.fixture.root / a).exists() for a in LOCAL_AREAS[1:]))
        self.runner.completed_steps.clear()
        self.runner.no_fail_fast = True
        self.assertNotEqual(self.invoke(), 0)
        self.assertTrue(all((self.fixture.root / a).exists() for a in LOCAL_AREAS))

    def test_partial_preparation_blocks_only_its_owner_under_continuation(self):
        self.fixture.blocked["area_diagnostics"] = "compiler_identity"
        self.assertNotEqual(self.invoke(), 0)
        self.assertFalse((self.fixture.root / "diagnostics").exists())
        self.assertTrue(all((self.fixture.root / a).exists()
                            for a in LOCAL_AREAS if a != "diagnostics"))
        self.runner.completed_steps.clear()
        self.runner.no_fail_fast = False
        self.fixture.commands.clear()
        self.assertNotEqual(self.invoke(), 0)
        self.assertEqual(self.fixture.commands, [])

    def test_failure_keeps_admitted_siblings_and_missing_results_never_pass(self):
        self.runner.no_fail_fast = False
        self.fixture.programs[LOCAL_AREAS[0]] = self.fixture.program(LOCAL_AREAS[0], code=9)
        self.fixture.programs["diagnostics"] = self.fixture.program("diagnostics", document=False)
        stale = self.fixture.root / "target/verification/areas/diagnostics-merge-correctness-results.json"
        stale.parent.mkdir(parents=True)
        stale.write_text(json.dumps(base.payload("diagnostics")))
        self.assertNotEqual(self.invoke(), 0)
        self.assertEqual(len(self.runner.completed_steps), 4)
        self.assertTrue(all((self.fixture.root / a).exists() for a in LOCAL_AREAS))
        reports = list((self.fixture.root / "target/verification/area-groups").glob("*/outcomes.json"))
        outcomes = json.loads(reports[-1].read_text())["outcomes"]
        self.assertNotEqual(outcomes["diagnostics"]["status"], 0)
        self.assertEqual(outcomes["core_language"]["status"], 0)
        self.assertEqual(outcomes["distribution_release"]["status"], 0)

    def test_changed_compiler_invalidates_all_four_results(self):
        self.fixture.programs["diagnostics"] = self.fixture.program(
            "diagnostics", extra=f"Path({str(self.fixture.compiler)!r}).write_bytes(b'changed')")
        self.assertNotEqual(self.invoke(), 0)
        self.assertTrue(self.runner.stop_requested)
        reports = list((self.fixture.root / "target/verification/area-groups").glob("*/outcomes.json"))
        outcomes = json.loads(reports[-1].read_text())["outcomes"]
        self.assertEqual(len(outcomes), 4)
        self.assertTrue(all(row["status"] != 0 for row in outcomes.values()))


if __name__ == "__main__":
    unittest.main()
