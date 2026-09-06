"""SQL spelling versus removed Sifr scalar regressions for the compatibility guard."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import check_no_pre_v1_compatibility as guard


BIGINT = guard.joined("big", "int")
PG_PATH = "crates/sifr_sql_postgresql/src/types.rs"
MYSQL_PATH = "crates/sifr_sql_mysql/src/types.rs"
PG_TEST_PATH = "crates/sifr_sql_postgresql/tests/postgresql_regressions.rs"
SQL_EXPRESSIONS = {
    PG_PATH: '''(
        &["int8", "BIGINT", "pg_catalog.int8"],
        DatabaseType::Integer {
            sign: IntegerSign::Signed,
            width: IntegerWidth::Bits64,
        },
    )'''.replace("BIGINT", BIGINT),
    MYSQL_PATH: '''"BIGINT" => DatabaseType::Integer {
        sign,
        width: IntegerWidth::Bits64,
    },'''.replace("BIGINT", BIGINT),
    PG_TEST_PATH: '''DatabaseType::Integer {
        width: sifr_sql_contract::IntegerWidth::Bits64,
        ..
    } => "BIGINT",'''.replace("BIGINT", BIGINT),
}
REMOVED_SURFACES = (
    f'let public_name = "{BIGINT}";',
    f"let public_name = '{BIGINT}';",
    guard.joined("Type::Big", "Int"),
    guard.joined("KnownType::Big", "Int"),
    guard.joined("SIFR-INT-", "0011"),
    guard.joined("SIFR-TYPE-", "0006"),
)


class SqlIntegerSpellingTests(unittest.TestCase):
    def scan_source(self, relative: str, source: str) -> list[guard.Failure]:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / relative
            path.parent.mkdir(parents=True)
            path.write_text(source, encoding="utf-8")
            return guard.scan(root)

    def assert_public_failure(self, relative: str, source: str) -> None:
        failures = self.scan_source(relative, source)
        self.assertTrue(failures, (relative, source))
        self.assertEqual({failure.rule_id for failure in failures}, {"public-bigint"})
        self.assertEqual({failure.path for failure in failures}, {Path(relative)})

    def test_three_external_sql_mappings_are_accepted(self) -> None:
        for relative, source in SQL_EXPRESSIONS.items():
            with self.subTest(path=relative):
                self.assertEqual(self.scan_source(relative, source), [])

    def test_real_repository_sites_match_exactly_one_literal_each(self) -> None:
        for relative in SQL_EXPRESSIONS:
            with self.subTest(path=relative):
                source = (guard.REPO_ROOT / relative).read_text(encoding="utf-8")
                spans = guard.sql_integer_spelling_spans(source, relative)
                self.assertEqual(len(spans), 1)
                self.assertEqual({source[start:end] for start, end in spans}, {f'"{BIGINT}"'})
                self.assertEqual(self.scan_source(relative, source), [])

    def test_whitespace_line_endings_and_unicode_offsets(self) -> None:
        for relative, original in SQL_EXPRESSIONS.items():
            for separator in (" ", "\n", "\r\n", "\t"):
                source = "// π database spelling\n\n" + separator.join(original.split())
                with self.subTest(path=relative, separator=repr(separator)):
                    self.assertEqual(self.scan_source(relative, source), [])

    def test_database_shape_and_width_are_required(self) -> None:
        for relative, source in SQL_EXPRESSIONS.items():
            for old, new in (
                ("Bits64", "Bits32"),
                ("DatabaseType::Integer", "Type::Integer"),
                ("DatabaseType::Integer", "OtherDatabaseType::Integer"),
            ):
                with self.subTest(path=relative, mutation=(old, new)):
                    self.assert_public_failure(relative, source.replace(old, new))

    def test_postgres_aliases_and_signedness_are_required(self) -> None:
        source = SQL_EXPRESSIONS[PG_PATH]
        for old, new in (
            ('"int8"', '"int4"'),
            ('"pg_catalog.int8"', '"pg_catalog.int4"'),
            ("IntegerSign::Signed", "IntegerSign::Unsigned"),
        ):
            with self.subTest(mutation=(old, new)):
                self.assert_public_failure(PG_PATH, source.replace(old, new))

    def test_mysql_sign_binding_is_preserved(self) -> None:
        source = SQL_EXPRESSIONS[MYSQL_PATH]
        for sign in ("IntegerSign::Signed", "IntegerSign::Unsigned", "Type::Integer"):
            with self.subTest(sign=sign):
                self.assert_public_failure(MYSQL_PATH, source.replace("sign,", f"sign: {sign},"))

    def test_sql_paths_are_not_blanket_exemptions(self) -> None:
        for relative in SQL_EXPRESSIONS:
            for surface in REMOVED_SURFACES:
                with self.subTest(path=relative, surface=surface):
                    self.assert_public_failure(relative, surface)

    def test_removed_language_support_still_fails_across_scan_roots(self) -> None:
        for relative in (
            "crates/compiler/src/types.rs", "stdlib/sifr/numeric.sifr",
            "demos/scalar/main.sifr", "verification/scalar_check.py",
        ):
            for surface in REMOVED_SURFACES:
                with self.subTest(path=relative, surface=surface):
                    self.assert_public_failure(relative, surface)

    def test_sql_shapes_in_other_paths_are_not_exempt(self) -> None:
        for owner, source in SQL_EXPRESSIONS.items():
            for relative in (
                "crates/compiler/src/types.rs", owner + ".rs",
                "stdlib/sifr/sql.sifr", "verification/sql.py",
                *[path for path in SQL_EXPRESSIONS if path != owner],
            ):
                with self.subTest(owner=owner, path=relative):
                    self.assert_public_failure(relative, source)

    def test_multiple_matches_on_same_line_remain_rejected(self) -> None:
        for relative, expression in SQL_EXPRESSIONS.items():
            compact = " ".join(expression.split())
            for surface in REMOVED_SURFACES:
                for source in (surface + " " + compact, compact + " " + surface):
                    with self.subTest(path=relative, source=source):
                        failures = self.scan_source(relative, source)
                        self.assertEqual([(f.rule_id, f.line) for f in failures], [("public-bigint", 1)])

    def test_nearby_lines_remain_rejected_with_exact_locations(self) -> None:
        for relative, expression in SQL_EXPRESSIONS.items():
            source = REMOVED_SURFACES[0] + "\n" + expression + "\n" + REMOVED_SURFACES[1]
            with self.subTest(path=relative):
                failures = self.scan_source(relative, source)
                self.assertEqual(
                    [(f.rule_id, f.line) for f in failures],
                    [("public-bigint", 1), ("public-bigint", len(source.splitlines()))],
                )

    def test_extra_literal_inside_mapping_is_not_exempt(self) -> None:
        for relative, expression in SQL_EXPRESSIONS.items():
            with self.subTest(path=relative):
                self.assert_public_failure(
                    relative, expression.replace("Bits64", f'Bits64 /* "{BIGINT}" */')
                )

    def test_other_rules_apply_beside_sql_mapping(self) -> None:
        hidden = guard.joined("__compat_", "sifr_scalar")
        for relative, expression in SQL_EXPRESSIONS.items():
            with self.subTest(path=relative):
                source = " ".join(expression.split()) + " " + hidden
                failures = self.scan_source(relative, source)
                self.assertEqual([(f.rule_id, f.line) for f in failures], [("hidden-compat-names", 1)])

    def test_retained_contract_is_mandatory(self) -> None:
        payload = json.loads(guard.CONTRACTS_PATH.read_text(encoding="utf-8"))
        payload["contracts"] = [
            row for row in payload["contracts"] if row["id"] != "retained-sql-integer-spellings"
        ]
        with self.assertRaises(guard.ContractError):
            guard.validate_contracts(payload, guard.REPO_ROOT)


if __name__ == "__main__":
    unittest.main()
