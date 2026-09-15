"""Selection, lock immutability and failure boundaries for fixture preparation."""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from .cargo_fixture_setup import locked_fixture_manifests, prepare_locked_fixture_caches
from .profile_commands import CommandFailed


def profile(*packages, mode="full"):
    return {
        "name": "fixture-test",
        "toolchain_steps": [f"cargo-test-sifr-{mode}"],
        "crate_test_membership": {"suites": [
            {"package": package, "modes": [mode]} for package in packages
        ]},
    }


class FixtureSetupPolicyTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.manifest = self.root / "fixture/Cargo.toml"
        self.manifest.parent.mkdir()
        self.manifest.write_text('[package]\nname="fixture"\nversion="0.0.0"\n[workspace]\n')
        self.lock = self.manifest.with_name("Cargo.lock")
        self.lock.write_text('version = 4\n')
        self.start_patch("sifr_verify.cargo_fixture_setup.REPO_ROOT", self.root)
        self.start_patch("sifr_verify.cargo_fixture_setup.OFFLINE_CRATE_TEST_FIXTURES", {
            "sifr_driver": ("fixture/Cargo.toml",),
        })

    def start_patch(self, name, value):
        patcher = patch(name, value)
        patcher.start()
        self.addCleanup(patcher.stop)

    def test_only_selected_crate_mode_prepares_its_graph_once(self):
        self.assertEqual(locked_fixture_manifests(profile("sifr_driver", "sifr_driver")), [self.manifest])
        self.assertEqual(locked_fixture_manifests(profile("sifr_codegen")), [])
        self.assertEqual(locked_fixture_manifests({"toolchain_steps": []}), [])
        payload = profile("sifr_driver")
        payload["crate_test_membership"]["suites"][0]["modes"] = ["smoke"]
        self.assertEqual(locked_fixture_manifests(payload), [])

    def test_complete_locked_fetch_preserves_graph_and_private_environment(self):
        commands = []
        env = {"CARGO_HOME": "/owned/cache"}
        records = prepare_locked_fixture_caches(
            profile("sifr_driver"), env,
            lambda args, **kw: commands.append((args, kw["env"].copy())),
        )
        self.assertEqual(commands, [([
            "cargo", "fetch", "--locked", "--manifest-path", str(self.manifest),
        ], env)])
        self.assertEqual(records[0]["manifest"], "fixture/Cargo.toml")
        self.assertEqual(set(records[0]), {"manifest", "Cargo.toml", "Cargo.lock"})

    def test_missing_lock_fails_before_any_fetch(self):
        self.lock.unlink()
        with self.assertRaises(FileNotFoundError):
            prepare_locked_fixture_caches(profile("sifr_driver"), {}, lambda *a, **k: self.fail("fetch ran"))

    def test_missing_unselected_graph_is_not_accessed(self):
        self.lock.unlink()
        self.assertEqual(prepare_locked_fixture_caches(profile("sifr_codegen"), {}, lambda *a, **k: self.fail("fetch ran")), [])

    def test_changed_manifest_or_lock_is_rejected(self):
        for path in (self.manifest, self.lock):
            with self.subTest(path=path):
                before = path.read_text()
                def mutate(*args, **kwargs):
                    path.write_text(before + "\n# unexpected change\n")
                with self.assertRaisesRegex(ValueError, "changed manifest or lock"):
                    prepare_locked_fixture_caches(profile("sifr_driver"), {}, mutate)
                path.write_text(before)

    def test_fetch_failure_stops_before_later_graphs(self):
        self.start_patch("sifr_verify.cargo_fixture_setup.OFFLINE_CRATE_TEST_FIXTURES", {
            "sifr_driver": ("fixture/Cargo.toml", "unreached/Cargo.toml"),
        })
        commands = []
        def fail(args, **kwargs):
            commands.append(args)
            raise CommandFailed(101)
        with self.assertRaises(CommandFailed):
            prepare_locked_fixture_caches(profile("sifr_driver"), {}, fail)
        self.assertEqual(len(commands), 1)
