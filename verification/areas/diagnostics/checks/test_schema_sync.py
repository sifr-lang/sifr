"""Lock strict schema synchronization, including order-only artifact drift."""

from __future__ import annotations

import contextlib
import io
import subprocess
import unittest
from unittest.mock import Mock, patch

import schema_sync


class SchemaSyncTests(unittest.TestCase):
    def run_check(
        self,
        actual: str,
        expected: str,
        *,
        exists: bool = True,
        generator_status: int = 0,
    ) -> tuple[int, str]:
        artifact = Mock()
        artifact.exists.return_value = exists
        artifact.read_text.return_value = actual
        generated = subprocess.CompletedProcess(
            args=[], returncode=generator_status, stdout=expected,
            stderr="generator failed\n" if generator_status else "",
        )
        stderr = io.StringIO()
        with (
            patch.object(schema_sync, "SCHEMA_PATH", artifact),
            patch.object(schema_sync.subprocess, "run", return_value=generated) as run,
            contextlib.redirect_stderr(stderr),
        ):
            result = schema_sync.main()
        run.assert_called_once_with(
            ["cargo", "run", "--locked", "-q", "-p", "sifr_diagnostics",
             "--bin", "gen-diagnostic-schema"],
            cwd=schema_sync.ROOT,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        if not exists or generator_status:
            artifact.read_text.assert_not_called()
        return result, stderr.getvalue()

    def assert_out_of_sync(self, actual: str, expected: str) -> None:
        result, error = self.run_check(actual, expected)
        self.assertEqual(result, 1)
        self.assertIn("is out of sync", error)

    def test_exact_generator_output_passes(self) -> None:
        output = '{"properties":{"a":{},"b":{}}}\n'
        self.assertEqual(self.run_check(output, output), (0, ""))

    def test_object_key_order_drift_fails(self) -> None:
        self.assert_out_of_sync(
            '{"properties":{"b":{},"a":{}}}\n',
            '{"properties":{"a":{},"b":{}}}\n',
        )

    def test_schema_value_drift_fails(self) -> None:
        self.assert_out_of_sync('{"type":"string"}\n', '{"type":"integer"}\n')

    def test_array_order_drift_fails(self) -> None:
        self.assert_out_of_sync(
            '{"required":["b","a"]}\n', '{"required":["a","b"]}\n',
        )

    def test_formatting_drift_fails(self) -> None:
        self.assert_out_of_sync('{"type": "string"}\n', '{"type":"string"}\n')
        self.assert_out_of_sync('{"type":"string"}', '{"type":"string"}\n')

    def test_missing_artifact_fails(self) -> None:
        result, error = self.run_check("", "{}\n", exists=False)
        self.assertEqual(result, 1)
        self.assertIn("is out of sync", error)

    def test_generator_failure_propagates_even_with_matching_stdout(self) -> None:
        result, error = self.run_check("{}\n", "{}\n", generator_status=101)
        self.assertEqual(result, 101)
        self.assertIn("failed to invoke generator", error)
        self.assertIn("generator failed", error)


if __name__ == "__main__":
    unittest.main()
