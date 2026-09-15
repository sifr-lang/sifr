"""Regression coverage for taxonomy audit ownership boundaries."""

from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import verification_taxonomy as taxonomy


class AuditPathTests(unittest.TestCase):
    def setUp(self) -> None:
        # Own a neutral absolute tree independently of the invoking checkout.
        self.temporary = tempfile.TemporaryDirectory(
            prefix="sifr-taxonomy-", dir=os.environ.get("TAXONOMY_TEST_TMPDIR")
        )
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)

    def audit_roots(self):
        callers = (
            Path("neutral"),
            Path("sifr-" + "item" + "12k-cont"),
            Path(taxonomy.DELIVERY_STAGE + "_99") / "caller",
            *(Path(name) / "caller" for name in sorted(taxonomy.SKIP_DIR_NAMES)),
        )
        for caller in callers:
            root = self.base / caller / "owned"
            root.mkdir(parents=True)
            yield root

    def write(self, root: Path, relative: str, text: str = "// compiler test\n") -> Path:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def test_valid_names_ignore_caller_ancestors(self) -> None:
        for root in self.audit_roots():
            with self.subTest(root=root):
                source = self.write(root, "crates/example/semantic_terms.rs")
                self.assertFalse(taxonomy.should_skip(source, audit_root=root))
                self.assertEqual(taxonomy.validate_filename(source, audit_root=root), [])
                self.assertEqual(taxonomy.collect_failures((root,), audit_root=root), [])

    def test_invalid_filenames_and_descendant_directories_remain_governed(self) -> None:
        labels = (
            "item" + "12k", "part" + "6", taxonomy.DELIVERY_STAGE + "40",
            taxonomy.DELIVERY_BATCH + "_2", "sprint" + "3", "workstream" + "4",
            "m" + "1", "p" + "1", "ms" + "2", "pt" + "3", "ph" + "4",
            "w" + "5", "M" + "8", "P" + "2", "m" + "1_2",
            taxonomy.DELIVERY_STEP + "_99", taxonomy.DELIVERY_WORK_ITEM + "-helper",
        )
        for root in self.audit_roots():
            expected: set[Path] = set()
            for label in labels:
                for relative in (
                    f"crates/example_{label}.rs",
                    f"crates/example_{label}/nested/semantic_terms.rs",
                ):
                    with self.subTest(root=root, relative=relative):
                        source = self.write(root, relative)
                        expected.add(source)
                        failures = taxonomy.validate_filename(source, audit_root=root)
                        self.assertTrue(failures)
                        self.assertTrue(all(failure.path == source for failure in failures))
                        self.assertTrue(all(relative in failure.text for failure in failures))
            self.assertEqual(set(taxonomy.walk_text_candidates(root, audit_root=root)), expected)
            for selection in ((root,), (root / "crates",), tuple(sorted(expected))):
                with self.subTest(root=root, selection=selection):
                    failures = taxonomy.collect_failures(selection, audit_root=root)
                    self.assertEqual({failure.path for failure in failures}, expected)

    def test_content_is_checked_beneath_every_caller(self) -> None:
        for root in self.audit_roots():
            with self.subTest(root=root):
                source = self.write(root, "crates/semantic_terms.rs", "// Item " + "8\n")
                for selection in ((root,), (source,)):
                    failures = taxonomy.collect_failures(selection, audit_root=root)
                    self.assertEqual(len(failures), 1)
                    self.assertEqual((failures[0].path, failures[0].line), (source, 1))

    def test_demo_content_uses_the_same_audit_root(self) -> None:
        for root in self.audit_roots():
            with self.subTest(root=root):
                demo = self.write(root, "demos/example/main.sifr", "m" + "12: bool = False\n")
                self.write(root, "crates/example/main.rs", "m" + "12: bool = false;\n")
                failures = taxonomy.collect_failures((root,), audit_root=root)
                self.assertEqual(len(failures), 1)
                self.assertEqual((failures[0].path, failures[0].line), (demo, 1))
                self.assertIn("descriptive name", failures[0].text)

    def test_only_governed_skip_components_and_extensions_are_excluded(self) -> None:
        for root in self.audit_roots():
            with self.subTest(root=root):
                for name in taxonomy.SKIP_DIR_NAMES:
                    source = self.write(root, f"{name}/nested/main.rs", "// Item " + "8\n")
                    for selection in ((root,), (root / name,), (source,)):
                        self.assertEqual(taxonomy.collect_failures(selection, audit_root=root), [])
                self.write(root, "opaque.txt", "Item " + "8\n")
                self.assertEqual(taxonomy.collect_failures((root,), audit_root=root), [])
                visible = self.write(root, "visible.rs", "// Item " + "8\n")
                failures = taxonomy.collect_failures((root,), audit_root=root)
                self.assertEqual([failure.path for failure in failures], [visible])

    def test_default_repository_boundary_retains_descendant_names(self) -> None:
        valid = taxonomy.REPO_ROOT / "crates" / "semantic_terms.rs"
        invalid = taxonomy.REPO_ROOT / "crates" / ("item" + "12k") / "semantic_terms.rs"
        self.assertEqual(taxonomy.validate_filename(valid), [])
        self.assertFalse(taxonomy.should_skip(valid))
        self.assertTrue(taxonomy.validate_filename(invalid))
        skipped = taxonomy.REPO_ROOT / "target" / "semantic_terms.rs"
        self.assertTrue(taxonomy.should_skip(skipped))

    def test_paths_outside_explicit_boundary_are_rejected(self) -> None:
        root = self.base / "owned"
        root.mkdir()
        source = self.write(self.base, "sibling/main.rs")
        for operation in (
            taxonomy.validate_filename, taxonomy.should_skip, taxonomy.validate_text,
            taxonomy.walk_text_candidates,
        ):
            with self.subTest(operation=operation.__name__):
                with self.assertRaises(ValueError):
                    operation(source, audit_root=root)
        with self.assertRaises(ValueError):
            taxonomy.collect_failures((source,), audit_root=root)

    def test_original_self_tests_under_neutral_matching_and_skip_ancestors(self) -> None:
        for root in self.audit_roots():
            with self.subTest(root=root), patch.object(tempfile, "tempdir", str(root)):
                self.assertEqual(taxonomy.run_self_test(quiet=True), 0)


if __name__ == "__main__":
    unittest.main()
