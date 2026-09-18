"""DX.10 B06/R06 inventory and configuration-plan negative controls."""
import copy
import json
from .paths import REPO_ROOT
import unittest
from .compiler_configuration_plan import configuration_plan
from .profiles import load_profile

class ProfilePlanTests(unittest.TestCase):
    def test_b06_r06_every_original_assertion_and_isolation_survives(self):
        for name in ("create-pr", "merge", "nightly", "release"):
            profile = load_profile(name)
            for mode in ("smoke", "full"):
                plan = configuration_plan(profile, mode)
                actual = sorted((id, list(command)) for group in plan for id, command in group.originals)
                expected = sorted((s["id"], s["command"]) for s in profile["crate_test_membership"]["suites"]
                    if mode in s["modes"] and not (s["status"] == "red-blocker" and not s["executed_in_merge"]))
                self.assertEqual(actual, expected)
                for group in plan:
                    if any(flag in group.command for flag in ("--features", "--all-features", "--no-default-features", "--target", "--ignored")):
                        self.assertEqual(group.classification, "isolated")
                        self.assertEqual(len(group.originals), 1)
                    self.assertEqual(group.preparation()[5:], list(group.command[1:group.command.index("--") if "--" in group.command else len(group.command)]))
                    self.assertEqual(group.execution()[4:], list(group.command[1:]))

    def test_b11_release_corpus_and_retained_native_selection(self):
        authority = json.loads((REPO_ROOT / "verification/runner/application_profiles.json").read_text())
        self.assertEqual(authority["policy_identity"], "sifr-application-profiles-v1")
        selection = next(s for s in authority["selections"] if s["id"] == "e2e-run-pass")
        self.assertEqual(selection["application_profiles"], ["development", "release"])
        self.assertIn("native-run", selection["assertions"])
        release = load_profile("release")
        self.assertFalse(release["e2e"].get("fixture_manifest"), "release must execute the complete run-pass corpus")
        fixtures = sorted(REPO_ROOT.glob(selection["selector"]))
        self.assertGreater(len(fixtures), 1000)
        self.assertEqual(fixtures, sorted((REPO_ROOT / "crates/sifr/tests/e2e/pass").glob("*.sifr")))

    def test_b06_isolation_mutation_cannot_be_absorbed_into_integration(self):
        profile = load_profile("merge")
        changed = copy.deepcopy(profile)
        suite = next(s for s in changed["crate_test_membership"]["suites"] if s["id"] == "sifr_frontend")
        suite["command"].extend(["--no-default-features"])
        plan = configuration_plan(changed, "full")
        group = next(g for g in plan if "sifr_frontend" in g.ids)
        self.assertEqual(group.classification, "isolated")
        self.assertIn("--no-default-features", group.preparation())
        self.assertIn("--no-default-features", group.execution())

if __name__ == "__main__": unittest.main()
