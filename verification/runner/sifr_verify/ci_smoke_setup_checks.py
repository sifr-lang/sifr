"""Check that CI primes the selected graphs without weakening offline execution."""
import unittest
from .ci_smoke_setup import prepare


class SmokePreparationChecks(unittest.TestCase):
    def test_acquisition_then_exact_offline_graphs(self):
        calls = []
        env = {"CARGO_NET_OFFLINE": "true", "CARGO_TARGET_DIR": "/owned/target"}
        prepare(env, lambda command, *, env: calls.append((command, env.copy())))
        self.assertEqual([command for command, _ in calls], [
            ["cargo", "fetch", "--locked"],
            ["cargo", "build", "--locked", "--offline", "-p", "sifr_driver",
             "--bin", "diagnostic_rendering_harness"],
            ["cargo", "test", "--locked", "--offline", "--no-run",
             "-p", "sifr", "--test", "e2e"],
            ["cargo", "build", "--locked", "--offline", "-p", "sifr"],
        ])
        self.assertNotIn("CARGO_NET_OFFLINE", calls[0][1])
        for _, build_env in calls[1:]:
            self.assertEqual(build_env["CARGO_NET_OFFLINE"], "true")
            self.assertEqual(build_env["CARGO_TARGET_DIR"], "/owned/target")
        self.assertEqual(env["CARGO_NET_OFFLINE"], "true")


if __name__ == "__main__":
    unittest.main()
