"""DX.6 configuration and failure boundaries for preparation adapters."""
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import Mock
from .cargo_crate_setup import prepare_crate_test_binaries
from .metadata_setup import build_and_prepare
from .profile_commands import CommandFailed


class MetadataSetupTests(unittest.TestCase):
    def executor(self, events, status=0):
        def execute(command, **kwargs):
            self.assertIn("--message-format=json-render-diagnostics", command)
            for event in events:
                data = (json.dumps(event) + "\n").encode()
                kwargs["emit"]("stdout", data[:7])
                kwargs["emit"]("stdout", data[7:])
            return SimpleNamespace(returncode=status)
        return execute

    def artifact(self, path, test=False):
        return {"reason": "compiler-artifact", "target": {"name": "sifr", "kind": ["bin"]},
                "profile": {"test": test}, "executable": path}

    def test_exact_cargo_artifact_is_prepared_without_identity_override(self):
        runner = Mock()
        env = {"CARGO_TARGET_DIR": "/owned/target"}
        build_and_prepare(["cargo", "build", "--release"], env,
            self.executor([self.artifact("/test/ignored", True), self.artifact("/exact/compiler")]), runner, Mock())
        args, kwargs = runner.call_args
        self.assertEqual(args[0][:3], ["/exact/compiler", "sysroot", "build-metadata"])
        self.assertNotIn("--target", args[0])
        self.assertEqual(kwargs["env"], env)
        self.assertFalse(Path(args[0][-1]).exists())

    def test_build_failure_or_ambiguous_artifact_never_launches_producer(self):
        for events, status, error in [([], 1, CommandFailed), ([], 0, ValueError),
                ([self.artifact("/a"), self.artifact("/b")], 0, ValueError)]:
            runner = Mock()
            with self.assertRaises(error):
                build_and_prepare(["cargo", "build"], {}, self.executor(events, status), runner)
            runner.assert_not_called()

    def test_driver_preparation_uses_each_selected_test_graph_once(self):
        commands = [["test", "-p", "sifr_driver", "--lib"],
                    ["test", "-p", "sifr_driver", "--lib"],
                    ["test", "-p", "sifr_driver", "--lib", "--features", "sql"]]
        profile = {"toolchain_steps": ["cargo-test-sifr-smoke"], "crate_test_membership": {"suites": [
            {"command": command, "modes": ["smoke"], "status": "blocking", "executed_in_merge": True}
            for command in commands]}}
        calls = []
        prepare_crate_test_binaries(profile, {}, lambda command, **kw: calls.append(command))
        ensures = [command for command in calls if "--no-run" not in command]
        self.assertEqual(len(ensures), 2)
        self.assertNotIn("--features", ensures[0])
        self.assertIn("--features", ensures[1])
        self.assertTrue(all(command[-2:] == ["--exact", "--nocapture"] for command in ensures))
