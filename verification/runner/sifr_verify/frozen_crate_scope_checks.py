"""Frozen crate coverage survives additive enrollment without weakening old checks."""
from dataclasses import replace
import json
from types import SimpleNamespace
import unittest

from .compiler_configuration_plan import Configuration, configuration_plan
from .paths import REPO_ROOT
from .profiles import load_profile


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def assert_frozen_crate_coverage(frozen, current):
    """Compare original commands, allowing only additional package ownership."""
    retained = {name for group in frozen for name in group["ids"]}
    require(retained, "frozen crate selection must not be empty")
    require(sum(len(group["ids"]) for group in frozen) == len(retained), "duplicate frozen ownership")
    projected_order = [[name for name in group.ids if name in retained] for group in current]
    require([ids for ids in projected_order if ids] == [group["ids"] for group in frozen],
            "original configuration order or grouping changed")
    for before in frozen:
        owners = [group for group in current if set(group.ids) & set(before["ids"])]
        require(len(owners) == 1, f"missing, duplicate or split original configuration: {before['ids']}")
        after = owners[0]
        require([name for name in after.ids if name in retained] == before["ids"],
                f"original ownership/order changed: {before['ids']}")
        require(after.classification == before["classification"], "original isolation changed")
        extras = set()
        for name, command in after.originals:
            if name not in retained:
                require(after.classification == "integration", "isolated selection gained an owner")
                require(len(command) == 3 and command[:2] == ("test", "-p"),
                        "only additive package enrollment may expand an original configuration")
                extras.add(command[2])

        def project(command):
            result = []
            i = 0
            while i < len(command):
                if command[i] == "-p":
                    require(i + 1 < len(command), "incomplete package selection")
                    if command[i + 1] not in extras:
                        result.extend(command[i:i + 2])
                    i += 2
                else:
                    result.append(command[i])
                    i += 1
            return result

        require(project(after.execution()) == before["execution"],
                f"original execution changed: {before['ids']}")
        require(project(after.preparation()) == before["preparation"],
                f"original preparation changed: {before['ids']}")


def frozen_crates():
    path = REPO_ROOT / "plans/issues/active/local-correctness-gate-evidence/scope.json"
    return json.loads(path.read_text())["crate_configurations"]


class FrozenCrateCoverageTests(unittest.TestCase):
    def setUp(self):
        self.frozen = frozen_crates()
        self.current = list(configuration_plan(load_profile("merge"), "full"))

    def reject(self, groups):
        with self.assertRaises(AssertionError):
            assert_frozen_crate_coverage(self.frozen, groups)

    def test_retained_selection_and_new_owners_are_accepted(self):
        assert_frozen_crate_coverage(self.frozen, self.current)
        added = {name for g in self.current for name in g.ids} - {name for g in self.frozen for name in g["ids"]}
        self.assertTrue({"sifr_semantic_services", "sifr_metadata", "sifr_stdlib_artifact"} <= added)

    def test_loss_and_duplicate_owner_fail_with_unchanged_configuration_count(self):
        self.reject([replace(self.current[0], ids=self.current[0].ids[1:]), *self.current[1:]])
        self.reject([replace(self.current[0], ids=(*self.current[0].ids, self.current[0].ids[0])), *self.current[1:]])
        self.reject([self.current[0], self.current[0], *self.current[2:]])

    def test_order_changes_fail(self):
        self.reject([self.current[1], self.current[0], *self.current[2:]])
        first = self.current[0]
        self.reject([replace(first, ids=tuple(reversed(first.ids))), *self.current[1:]])
        command = list(first.command)
        command[1:5] = command[3:5] + command[1:3]
        self.reject([replace(first, command=tuple(command)), *self.current[1:]])

    def test_features_filters_and_isolation_cannot_change(self):
        for index, group in enumerate(self.current):
            if "--features" in group.command or "--ignored" in group.command:
                command = tuple(arg for arg in group.command if arg not in ("--features", "--ignored"))
                changed = self.current.copy(); changed[index] = replace(group, command=command)
                with self.subTest(ids=group.ids):
                    self.reject(changed)
        self.reject([replace(self.current[0], classification="isolated"), *self.current[1:]])

    def test_preparation_cannot_change_independently_of_execution(self):
        first = self.current[0]
        forged = SimpleNamespace(ids=first.ids, originals=first.originals,
                                 classification=first.classification, execution=first.execution,
                                 preparation=lambda: [arg for arg in first.preparation() if arg != "--locked"])
        self.reject([forged, *self.current[1:]])

    def test_original_configurations_cannot_merge_or_split(self):
        one, two = self.current[:2]
        merged = replace(one, ids=(*one.ids, *two.ids), originals=(*one.originals, *two.originals),
                         command=(*one.command, *two.command[1:]))
        self.reject([merged, *self.current[2:]])
        split = Configuration(one.ids[:1], one.originals[0][1], one.originals[:1], "isolated")
        self.reject([split, replace(one, ids=one.ids[1:], originals=one.originals[1:]), *self.current[1:]])

    def test_new_owner_cannot_smuggle_features_into_existing_configuration(self):
        first = self.current[0]
        extra = ("future_owner", ("test", "-p", "future_owner", "--all-features"))
        changed = replace(first, ids=(*first.ids, extra[0]), originals=(*first.originals, extra))
        self.reject([changed, *self.current[1:]])


if __name__ == "__main__":
    unittest.main()
