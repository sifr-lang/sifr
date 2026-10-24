"""Command contracts for version-owned PostgreSQL builds; no live database claims."""

from __future__ import annotations

import os
import unittest
from types import SimpleNamespace
from unittest.mock import patch

import run_postgresql_migration_matrix as migrations
import run_postgresql_parser_matrix as parsers
import run_postgresql_schema_tool_matrix as schemas
import test_postgresql_build_inputs as build_inputs_tests


class PostgreSqlBuildSelectionTests(unittest.TestCase):
    def test_parser_matrix_executes_every_major_and_the_component_host(self) -> None:
        with patch.dict(os.environ, {"OWNED_TEST_INPUT": "retained"}, clear=True):
            with patch.object(parsers.subprocess, "run", return_value=SimpleNamespace(returncode=0)) as run:
                self.assertEqual(parsers.main(), 0)
            self.assertEqual(os.environ, {"OWNED_TEST_INPUT": "retained"})
        self.assertEqual(len(run.call_args_list), 7)
        for major, call in zip(range(13, 19), run.call_args_list[:6], strict=True):
            command = call.args[0]
            self.assertEqual(command[command.index("--features") + 1], f"postgresql-{major}")
            self.assertIn("--tests", command)
            self.assertEqual(command[-2:], ["--skip", "every_checked_in_component_executes_in_the_capability_free_host"])
            self.assertEqual(call.kwargs["env"]["SIFR_POSTGRESQL_TEST_MAJOR"], str(major))
            self.assertNotIn("SIFR_POSTGRESQL_MAJOR", call.kwargs["env"])
        host = run.call_args_list[-1].args[0]
        self.assertIn("every_checked_in_component_executes_in_the_capability_free_host", host)
        self.assertEqual(host[-2:], ["--", "--exact"])
        self.assertEqual(host[host.index("--features") + 1], "postgresql-18")

    def test_failed_parser_major_is_not_reported_as_a_pass(self) -> None:
        with patch.object(parsers.subprocess, "run", side_effect=[
            SimpleNamespace(returncode=0), SimpleNamespace(returncode=7),
        ]) as run:
            self.assertEqual(parsers.main(), 7)
        self.assertEqual(run.call_count, 2)

    def test_live_schema_commands_select_the_dependency_version(self) -> None:
        for major in range(13, 19):
            with (
                patch.object(schemas, "ensure_image"),
                patch.object(schemas, "wait_ready"),
                patch.object(schemas, "psql"),
                patch.object(schemas, "image_digest", return_value="pinned-test-digest"),
                patch.object(schemas, "run", return_value=SimpleNamespace(stdout="127.0.0.1:5432")) as run,
                patch.object(schemas.subprocess, "check_output", return_value="1000"),
                patch.object(schemas.subprocess, "run"),
                patch.dict(os.environ, {}, clear=True),
            ):
                result = schemas.run_server(major)
            self.assertEqual(result["status"], "passed")
            calls = [c for c in run.call_args_list if c.args[0][0] == "cargo"]
            self.assertEqual(len(calls), 1)
            command = calls[0].args[0]
            self.assertEqual(command[command.index("--features") + 1], f"sifr_sql_postgresql/postgresql-{major}")
            self.assertNotIn("SIFR_POSTGRESQL_MAJOR", calls[0].kwargs["env"])
            self.assertEqual(calls[0].kwargs["env"]["SIFR_POSTGRESQL_SCHEMA_TOOL_TEST_MAJOR"], str(major))

    def test_live_migration_commands_select_the_dependency_version(self) -> None:
        for major in range(13, 19):
            with (
                patch.object(migrations, "wait_ready"),
                patch.object(migrations.subprocess, "check_output", return_value="1000"),
                patch.object(migrations, "run", return_value=SimpleNamespace(stdout="127.0.0.1:5432")),
                patch.object(migrations.subprocess, "run", return_value=SimpleNamespace(returncode=0)) as run,
                patch.dict(os.environ, {}, clear=True),
            ):
                migrations.run_server({"major": major, "image": "postgres", "image_digest": "pinned-test-digest"})
            calls = [c for c in run.call_args_list if c.args[0][0] == "cargo"]
            self.assertEqual(len(calls), 1)
            command = calls[0].args[0]
            self.assertEqual(command[command.index("--features") + 1], f"sifr_sql_postgresql/postgresql-{major}")
            self.assertNotIn("SIFR_POSTGRESQL_MAJOR", calls[0].kwargs["env"])
            self.assertEqual(calls[0].kwargs["env"]["SIFR_POSTGRESQL_MIGRATION_TEST_MAJOR"], str(major))


def load_tests(loader, tests, _pattern):
    tests.addTests(loader.loadTestsFromTestCase(build_inputs_tests.PostgreSqlBuildInputsTests))
    return tests


if __name__ == "__main__":
    unittest.main()
