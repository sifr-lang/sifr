#!/usr/bin/env python3
"""Focused identity and rejection regressions for canonical code coverage."""

from __future__ import annotations

from contextlib import redirect_stderr
from io import StringIO
import unittest
from unittest.mock import patch

import code_coverage


class ReferenceIdentityTests(unittest.TestCase):
    def test_canonical_paths_and_spacing(self) -> None:
        for reference in (
            "DiagnosticCode::TYPE_MISMATCH",
            "sifr_diagnostics::DiagnosticCode::TYPE_MISMATCH",
            "::sifr_diagnostics::DiagnosticCode::TYPE_MISMATCH",
            "DiagnosticCode \n :: \t TYPE_MISMATCH",
            "r#DiagnosticCode::r#TYPE_MISMATCH",
            "(DiagnosticCode::TYPE_MISMATCH).as_str()",
        ):
            with self.subTest(reference=reference):
                self.assertEqual(
                    code_coverage.diagnostic_constant_references(reference),
                    ["TYPE_MISMATCH"],
                )

    def test_provider_and_other_type_identifiers_are_not_canonical(self) -> None:
        for owner in (
            "MysqlDiagnosticCode", "SqliteDiagnosticCode", "OtherDiagnosticCode",
            "_DiagnosticCode", "my_DiagnosticCode", "éDiagnosticCode",
            "DiagnosticCodes", "DiagnosticCodeExtra", "DiagnosticCode_",
        ):
            for member in ("UnsupportedMode", "TYPE_MISMATCH"):
                with self.subTest(owner=owner, member=member):
                    self.assertEqual(
                        code_coverage.diagnostic_constant_references(f"{owner}::{member}"),
                        [],
                    )

    def test_complete_member_tokens(self) -> None:
        for member in (
            "TYPE_MISMATCH", "TYPE_MISMATCHExtra", "TYPE_MISMATCH_extra",
            "TYPE_MISMATCH2", "TYPE_MISMATCHé", "UnsupportedMode", "_UNKNOWN",
        ):
            with self.subTest(member=member):
                self.assertEqual(
                    code_coverage.diagnostic_constant_references(f"DiagnosticCode::{member}"),
                    [member],
                )

    def test_associated_functions_are_not_constant_uses(self) -> None:
        self.assertEqual(
            code_coverage.diagnostic_constant_references(
                'DiagnosticCode::new("SIFR-TYPE-0002", Severity::Error)'
            ),
            [],
        )

    def test_mixed_references_preserve_each_canonical_use(self) -> None:
        self.assertEqual(
            code_coverage.diagnostic_constant_references(
                "MysqlDiagnosticCode::UnsupportedMode, DiagnosticCode::TYPE_MISMATCH, "
                "SqliteDiagnosticCode::Parse, DiagnosticCode::UNKNOWN, "
                "DiagnosticCode::TYPE_MISMATCH"
            ),
            ["TYPE_MISMATCH", "UNKNOWN", "TYPE_MISMATCH"],
        )


class CoverageRejectionTests(unittest.TestCase):
    def run_coverage(
        self, source: str, active_reference: str = "DiagnosticCode::TYPE_MISMATCH"
    ) -> tuple[int, str]:
        # Supply source text only: exercise the real registry parser and main
        # rejection/coverage logic without changing a compiler file or fixture.
        source_path = code_coverage.ROOT / "crates/sifr_sql_mysql/src/analyzer.rs"
        registry = '''
pub const TYPE_MISMATCH: Self = Self::new("SIFR-TYPE-0002", Severity::Error);
pub const TYPE_LEGACY: Self = Self::new("SIFR-TYPE-0999", Severity::Error);
ACTIVE_DIAGNOSTIC_CODES: &[DiagnosticCode] = &[ACTIVE_REFERENCE];
active_entry!("SIFR-TYPE-0002", Severity::Error, "crates/sifr_sql_mysql/src/analyzer.rs");
'''.replace("ACTIVE_REFERENCE", active_reference)

        def read_source(path):
            self.assertIn(path, (code_coverage.CODES_RS, source_path))
            return registry if path == code_coverage.CODES_RS else source

        stderr = StringIO()
        with (
            patch.object(code_coverage, "non_test_compiler_sources", return_value=[source_path]),
            patch.object(code_coverage, "read_rust_with_local_sources", side_effect=read_source),
            redirect_stderr(stderr),
        ):
            result = code_coverage.main()
        return result, stderr.getvalue()

    def test_active_use_passes_alongside_providers(self) -> None:
        self.assertEqual(self.run_coverage(
            "MysqlDiagnosticCode::UnsupportedMode; SqliteDiagnosticCode::Parse; "
            "DiagnosticCode::TYPE_MISMATCH;"
        ), (0, ""))

    def test_unknown_canonical_names_are_rejected_in_full(self) -> None:
        for name in ("UNKNOWN", "UnsupportedMode", "TYPE_MISMATCHExtra", "_UNKNOWN"):
            with self.subTest(name=name):
                result, errors = self.run_coverage(
                    f"DiagnosticCode::TYPE_MISMATCH; DiagnosticCode::{name};"
                )
                self.assertEqual(result, 1)
                self.assertEqual(errors.splitlines(), [
                    "diagnostic code coverage: crates/sifr_sql_mysql/src/analyzer.rs: "
                    f"references unknown DiagnosticCode::{name}"
                ])

    def test_non_active_canonical_name_is_rejected(self) -> None:
        result, errors = self.run_coverage(
            "DiagnosticCode::TYPE_MISMATCH; DiagnosticCode::TYPE_LEGACY;"
        )
        self.assertEqual(result, 1)
        self.assertIn("references non-active DiagnosticCode::TYPE_LEGACY", errors)

    def test_provider_and_partial_names_do_not_supply_required_use(self) -> None:
        for source in (
            "", "MysqlDiagnosticCode::TYPE_MISMATCH", "_DiagnosticCode::TYPE_MISMATCH",
            "DiagnosticCode::TYPE_MISMATCHExtra",
        ):
            with self.subTest(source=source):
                result, errors = self.run_coverage(source)
                self.assertEqual(result, 1)
                self.assertIn(
                    "SIFR-TYPE-0002 (TYPE_MISMATCH) is active but has no non-test "
                    "compiler-source DiagnosticCode::TYPE_MISMATCH use", errors
                )

    def test_registry_list_uses_same_complete_identity(self) -> None:
        for reference in (
            "MysqlDiagnosticCode::TYPE_MISMATCH", "DiagnosticCode::TYPE_MISMATCHExtra",
        ):
            with self.subTest(reference=reference):
                result, errors = self.run_coverage("", active_reference=reference)
                self.assertEqual(result, 1)
                self.assertIn(
                    "SIFR-TYPE-0002 is active in the registry but missing from "
                    "ACTIVE_DIAGNOSTIC_CODES", errors
                )

    def test_cfg_test_use_does_not_supply_required_coverage(self) -> None:
        result, errors = self.run_coverage(
            "#[cfg(test)]\nmod tests {\nlet code = DiagnosticCode::TYPE_MISMATCH;\n}"
        )
        self.assertEqual(result, 1)
        self.assertIn("is active but has no non-test compiler-source", errors)


if __name__ == "__main__":
    unittest.main(verbosity=2)
