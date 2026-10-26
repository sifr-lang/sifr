"""Reject metadata payload loss and accidental persistence of live authorities."""
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import check_dx_metadata_inventory as inventory

from check_dx_metadata_inventory import check_external_fields


class MetadataInventoryTests(unittest.TestCase):
    def test_shared_oracle_is_discovered_and_semantic_drift_changes_inventory(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            relative = "verification/tools/corpus_oracle/src/lib.rs"
            path = root / relative
            path.parent.mkdir(parents=True)
            path.write_text("pub fn parity(external_defs: ExternalDefs) { consume(external_defs); }")
            with patch.object(inventory, "ROOT", root), patch.object(inventory, "OWNERS", ()):
                original = inventory.sites()
                self.assertIn(chr(96) + f"{relative}:1" + chr(96) + " " + chr(96) + "parity" + chr(96), original)
                self.assertIn("| read | test setup/parity |", original)
                path.write_text("pub fn parity(external_defs: ExternalDefs) { external_defs.iter(); }")
                changed = inventory.sites()
                self.assertNotEqual(original, changed)
                self.assertIn("| read, enumerate | test setup/parity |", changed)

    def test_missing_shared_oracle_is_rejected(self):
        with TemporaryDirectory() as temporary:
            with patch.object(inventory, "ROOT", Path(temporary)), patch.object(inventory, "OWNERS", ()):
                with self.assertRaisesRegex(ValueError, "missing maintained shared metadata consumer"):
                    inventory.sites()

    def source(self, extra=""):
        return """pub struct ExternalDefs {
    pub functions: Functions,
    pub provider: Option<std::sync::Arc<dyn ExternalProvider>>,
    pub typed_method_processor: Option<std::sync::Arc<dyn TypedMethodProcessor>>,
""" + extra + "\n}\n"

    def wire(self, extra=""):
        return "pub struct SemanticExports {\n    pub functions: Functions,\n" + extra + "\n}\n"

    def test_payload_and_two_live_authorities_are_explicit(self):
        check_external_fields(self.source(), self.wire())

    def test_whitespace_and_comments_do_not_change_classification(self):
        source = self.source().replace("Option<std::sync::Arc", "Option < std::sync::Arc")
        source = source.replace("pub provider:", "// live callback\n    pub provider:")
        check_external_fields(source, self.wire())

    def test_new_semantic_payload_is_not_silently_excluded(self):
        with self.assertRaisesRegex(ValueError, "field inventory changed"):
            check_external_fields(self.source("    pub new_payload: Payload,"), self.wire())

    def test_missing_wire_payload_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "field inventory changed"):
            check_external_fields(self.source(), self.wire().replace("    pub functions: Functions,\n", ""))

    def test_wire_only_payload_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "field inventory changed"):
            check_external_fields(self.source(), self.wire("    pub new_payload: Payload,"))

    def test_new_semantic_field_must_have_a_matching_wire_field(self):
        field = "    pub new_payload: Payload,"
        check_external_fields(self.source(field), self.wire(field))

    def test_provider_shape_cannot_be_reclassified_as_semantic_data(self):
        for name, trait in (("provider", "ExternalProvider"),
                            ("typed_method_processor", "TypedMethodProcessor")):
            with self.subTest(field=name), self.assertRaisesRegex(ValueError, "live authority shape changed"):
                source = self.source().replace(f"pub {name}: Option<std::sync::Arc<dyn {trait}>>",
                                               f"pub {name}: SemanticPayload")
                check_external_fields(source, self.wire())

    def test_provider_removal_is_an_explicit_inventory_change(self):
        for name, trait in (("provider", "ExternalProvider"),
                            ("typed_method_processor", "TypedMethodProcessor")):
            with self.subTest(field=name), self.assertRaisesRegex(ValueError, "live authority shape changed"):
                source = self.source().replace(f"    pub {name}: Option<std::sync::Arc<dyn {trait}>>,\n", "")
                check_external_fields(source, self.wire())

    def test_wire_cannot_persist_live_authorities(self):
        for name in ("provider", "typed_method_processor"):
            with self.subTest(field=name), self.assertRaisesRegex(ValueError, "field inventory changed"):
                check_external_fields(self.source(), self.wire(f"    pub {name}: Payload,"))

    def test_unknown_provider_is_not_automatically_excluded(self):
        with self.assertRaisesRegex(ValueError, "field inventory changed"):
            check_external_fields(self.source("    pub another_provider: Callback,"), self.wire())

    def test_other_field_name_cannot_spoof_provider_shape(self):
        source = self.source().replace("pub provider: Option<std::sync::Arc<dyn ExternalProvider>>",
                                       "pub provider: SemanticPayload")
        field = "    pub otherpubprovider: Option<std::sync::Arc<dyn ExternalProvider>>,"
        with self.assertRaisesRegex(ValueError, "live authority shape changed"):
            check_external_fields(source.replace("\n}", "\n" + field + "\n}"), self.wire(field))


if __name__ == "__main__":
    unittest.main()
