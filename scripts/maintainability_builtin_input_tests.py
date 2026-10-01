"""Actual-producer acceptance for the bounded H03a1p builtin capability."""
import copy
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest

import maintainability_builtin_input as builtin


class BuiltinCapabilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.target = Path(os.environ["SIFR_BUILTIN_TARGET_DIR"]).resolve()
        cls.helper = cls.target / "debug/sifr_maintainability_builtin_input"
        cls.resolver = cls.target / "debug/ra_common"
        cls.receipt_path = Path(os.environ["SIFR_BUILTIN_COMPONENT_RECEIPT"])
        cls.evidence = Path(tempfile.mkdtemp(prefix="acceptance-", dir=os.environ["SIFR_BUILTIN_EVIDENCE_DIR"]))
        cls.identity = builtin.tool_identity(cls.receipt_path, cls.target)
        cls.fixture, cls.fixture_receipt = builtin.capture_package(
            builtin.TOOL / "fixtures", "builtin_fixture", "builtin_fixture",
            cls.evidence / "fixture", cls.helper, cls.target, cls.identity,
        )
        environment = os.environ.copy()
        environment.pop("RUSTC_BOOTSTRAP", None)
        environment.update(cls.fixture_receipt["inputs"]["resolver_preparation_environment"])
        common = builtin.run(
            [str(cls.resolver), str(builtin.TOOL / "fixtures"), "builtin_fixture", "fixture_root/src/lib.rs", str(cls.evidence / "fixture/capture.json")],
            env=environment, log=cls.evidence / "fixture-resolver.log",
        )
        cls.common = json.loads(common.stdout)
        (cls.evidence / "fixture-common.json").write_bytes(builtin.encoded(cls.common))
        builtin.verify_capture(cls.fixture, cls.fixture_receipt, cls.fixture_receipt["inputs"])
        cls.join = builtin.validate_join(cls.fixture, cls.common)
        (cls.evidence / "fixture-join.json").write_bytes(builtin.encoded(cls.join))

    def setUp(self):
        self.assertions = 0

    def tearDown(self):
        print(f"{self.id()}: assertions={self.assertions}; evidence={self.evidence}")

    def require(self, condition, reason):
        self.assertions += 1
        self.assertTrue(condition, reason)

    def reject(self, value, label, *, receipt=None):
        self.assertions += 1
        evidence_receipt = copy.deepcopy(receipt or self.fixture_receipt)
        evidence_receipt["capture_digest"] = builtin.digest(builtin.encoded(value))
        with self.assertRaises(builtin.Unsupported):
            builtin.verify_capture(value, evidence_receipt, evidence_receipt["inputs"])

        (self.evidence / (label + ".json")).write_bytes(builtin.encoded(value))

    def method(self, receiver, name):
        return next(d for d in self.fixture["declarations"] if d["owner"].startswith(f"<{receiver} as ") and d["owner"].endswith("::" + name))

    def test_builtin_bodies_calls_and_auxiliary_origins(self):
        self.require(len(self.join) == len([d for d in self.fixture["declarations"] if d["owner_kind"] == "AssocFn"]), "all common methods joined")
        for receiver in ("Record", "Tuple", "Unit", "Variants", "CopySensitive", "ExternalFields"):
            for method in ("fmt", "clone", "eq"):
                declaration = self.method(receiver, method)
                self.require(declaration["ast_body"] and declaration["hir_body"], f"{receiver}.{method} actual body")
                self.require(declaration["ast_body_tokens"] and declaration["hir_body_tokens"], f"{receiver}.{method} complete body witness")
        all_sites = [s for d in self.fixture["declarations"] for s in d["typed_sites"]]
        self.require(any(s["target"] and "debug_struct_field" in s["target"] for s in all_sites), "actual specialized Debug helper")
        self.require(any(s["target"] and s["target"].endswith("PartialEq::eq") and s["kind"].startswith("binary:") for s in all_sites), "typed overloaded comparison")
        self.require(any(s["disposition"] == "scalar-operation" for s in all_sites), "explicit scalar operations")
        marker = next(d for d in self.fixture["declarations"] if "StructuralPartialEq>" in d["owner"])
        self.require(marker["ast_impl_member_count"] == marker["hir_impl_member_count"] == 0 and not marker["ast_body"], "proven bodyless marker")
        self.require(any("TrivialClone>" in d["owner"] for d in self.fixture["declarations"]), "Copy-dependent auxiliary impl")
        corrupted = copy.deepcopy(self.fixture)
        next(d for d in corrupted["declarations"] if d["owner_kind"] == "AssocFn")["hir_body"] = False
        self.reject(corrupted, "negative-empty-required-body")
        corrupted = copy.deepcopy(self.fixture)
        corrupted["declarations"] = [d for d in corrupted["declarations"] if not d["owner"].startswith("<Record as std::cmp::PartialEq>")]
        self.reject(corrupted, "negative-missing-entire-PartialEq-impl")
        self.assertions += 1
        with self.assertRaises(builtin.Unsupported):
            builtin.validate_join(corrupted, self.common)
        corrupted = copy.deepcopy(self.fixture)
        corrupted["declarations"] = [d for d in corrupted["declarations"] if d["owner"] != marker["owner"]]
        self.reject(corrupted, "negative-missing-bodyless-marker")
        corrupted = copy.deepcopy(self.fixture)
        next(s for d in corrupted["declarations"] for s in d["typed_sites"] if s["target"])["target"] = "invented::direct_target"
        self.reject(corrupted, "negative-invented-target")
        corrupted = copy.deepcopy(self.fixture)
        corrupted["declarations"].remove(self.method("Record", "fmt"))
        self.reject(corrupted, "negative-missing-Record-fmt")
        self.assertions += 1
        with self.assertRaises(builtin.Unsupported):
            builtin.validate_join(corrupted, self.common)


    def test_hygiene_and_same_spelled_methods_preserve_trait_origin(self):
        cloned = self.method("ExternalFields", "clone")["typed_sites"]
        compared = self.method("ExternalFields", "eq")["typed_sites"]
        self.require(all(s["trait"] and s["trait"].endswith("::Clone") for s in cloned), "trait Clone authority rather than inherent clone")
        self.require(all(s["trait"] and s["trait"].endswith("::PartialEq") for s in compared if s["kind"] == "binary:Eq"), "trait equality authority")
        self.require(all(s["implementation"] and "field_origin::External" in s["implementation"] for s in cloned), "actual locked external impl")
        self.require(all(s["implementation_owner"] == "checkout:/field_origin#0.1.0" for s in cloned), "actual external Cargo package/version preserved")
        local = next(d for d in self.fixture["other_macro_declarations"] if d["owner"].endswith("LocallyGenerated"))
        self.require(local["expansion_chain"][0]["macro_identity"] == "builtin_fixture::Debug" and not local["expansion_chain"][0]["builtin"], "same-spelled macro resolved as local")
        debug = self.method("Record", "fmt")
        self.require(debug["expansion_chain"][0]["macro_identity"] == "core::fmt::macros::Debug" and debug["expansion_chain"][0]["builtin"], "builtin resolved macro identity")
        corrupted = copy.deepcopy(self.fixture)
        site = next(s for d in corrupted["declarations"] for s in d["typed_sites"] if s["target"] and s["target"].endswith("Clone::clone"))
        site["target"] = "field_origin::External::clone"
        site["trait"] = None
        self.reject(corrupted, "negative-ra-shadow-inherent-clone")
        corrupted = copy.deepcopy(self.fixture)
        corrupted["declarations"][0]["expansion_chain"][0]["macro_identity"] = "builtin_fixture::Debug"
        self.reject(corrupted, "negative-name-only-builtin")

    def test_expansion_ast_hir_mapping_is_owned_and_complete(self):
        for declaration in self.fixture["declarations"]:
            for ast, typed in zip(declaration["ast_sites"], declaration["typed_sites"]):
                self.require(ast["token_sequence"] == typed["token_sequence"], "verified syntax/call structural witness")
                self.require(ast["mapping"] == "owned-structural-one-to-one", "explicit mapping relation")
                self.require(ast["span"]["quality"] == "coarse-generated-anchor", "coarse generated provenance retained")
        nested = self.method("Nested", "clone")
        self.require(any(c["macro_identity"] == "builtin_fixture::nest" for c in nested["expansion_chain"]), "real parent expansion chain")
        self.require(self.method("left::Same", "clone")["receiver_identity"] != self.method("right::Same", "clone")["receiver_identity"], "same-name owners stay distinct")
        root = self.evidence / "other-checkout"
        shutil.copytree(builtin.TOOL / "fixtures", root)
        capture, receipt = builtin.capture_package(root, "builtin_fixture", "builtin_fixture", self.evidence / "other-capture", self.helper, self.target, self.identity)
        self.require(capture == self.fixture, "unchanged producer capture normalizes across checkouts")
        for mutation, label in (
            (lambda c: c["declarations"][0].update(owner=c["declarations"][1]["owner"]), "swapped-owners"),
            (lambda c: c["declarations"][1]["ast_sites"][0]["span"].update(quality="exact-source"), "forged-exact"),
            (lambda c: c["declarations"][1]["typed_sites"].clear(), "dropped-calls"),
            (lambda c: c["declarations"][1]["typed_sites"].append(copy.deepcopy(c["declarations"][1]["typed_sites"][0])), "duplicate-calls"),
            (lambda c: c["declarations"][1]["ast_sites"][0].update(ancestor=0), "ambiguous-structure"),
        ):
            corrupted = copy.deepcopy(self.fixture)
            mutation(corrupted)
            self.reject(corrupted, "negative-" + label)

    def test_component_context_and_input_drift_fail_closed(self):
        self.require(builtin.verify_capture(self.fixture, self.fixture_receipt, self.fixture_receipt["inputs"])["invocations"] > 0, "exact known context admitted")
        component = json.loads(self.receipt_path.read_text())
        for field, changed in (("rustc", "wrong commit"), ("inventory", {})):
            corrupted = copy.deepcopy(component)
            corrupted[field] = changed
            path = self.receipt_path.parent / (self.evidence.name + "-negative-component-" + field + ".json")
            path.write_bytes(builtin.encoded(corrupted))
            self.assertions += 1
            with self.assertRaises(builtin.Unsupported):
                builtin.tool_identity(path)
        for field, changed in (("target", "x86_64-pc-windows-msvc"), ("test", True), ("files", {}), ("tool", {}), ("invocation", {})):
            current = copy.deepcopy(self.fixture_receipt["inputs"])
            current[field] = changed
            self.assertions += 1
            with self.assertRaises(builtin.Unsupported):
                builtin.verify_capture(self.fixture, self.fixture_receipt, current)
        inputs = self.fixture_receipt["inputs"]
        for label, path in (
            ("source", builtin.TOOL / "fixtures/fixture_root/src/lib.rs"),
            ("external-source", builtin.TOOL / "fixtures/field_origin/src/lib.rs"),
            ("configuration", builtin.TOOL / "fixtures/Cargo.toml"),
            ("extern", next(Path(name) for name in inputs["files"] if name.endswith(".rmeta") and Path(name).is_relative_to(self.target))),
        ):
            original = path.read_bytes()
            stat = path.stat()
            try:
                path.write_bytes(original + b"\n")
                self.assertions += 1
                with self.assertRaises(builtin.Unsupported):
                    builtin.verify_capture(self.fixture, self.fixture_receipt, inputs)
            finally:
                path.write_bytes(original)
                os.utime(path, ns=(stat.st_atime_ns, stat.st_mtime_ns))
            self.require(builtin.verify_capture(self.fixture, self.fixture_receipt, inputs)["invocations"] > 0, label + " exact restoration")
        environment = os.environ.copy()
        environment.pop("RUSTC_BOOTSTRAP", None)
        destination = self.evidence / "failed-typecheck.json"
        environment["SIFR_BUILTIN_CAPTURE"] = str(destination)
        broken = self.evidence / "broken.rs"
        broken.write_text("#[derive(Debug, Clone, PartialEq)] pub struct Broken { value: UndefinedType }\n")
        result = builtin.run([str(self.helper), str(broken), "--crate-type", "lib", "--edition", "2024"], env=environment, log=self.evidence / "failed-typecheck.log", allowed=(101,))
        self.require(result.returncode != 0 and not destination.exists(), "failed typechecking publishes no partial capture")
        self.require("RUSTC_BOOTSTRAP" not in environment, "bootstrap excluded from analysis")
        corrupted = copy.deepcopy(self.fixture)
        corrupted["declarations"][0]["expansion_chain"] = []
        self.reject(corrupted, "negative-missing-expansion")

    def test_live_rust_ir_builtin_surface_has_complete_dispositions(self):
        live, receipt = builtin.capture_package(builtin.ROOT, "sifr_codegen", "sifr_codegen", self.evidence / "live", self.helper, self.target, self.identity)
        environment = os.environ.copy()
        environment.pop("RUSTC_BOOTSTRAP", None)
        environment.update(receipt["inputs"]["resolver_preparation_environment"])
        common = json.loads(builtin.run([str(self.resolver), str(builtin.ROOT), "sifr_codegen", "crates/sifr_codegen/src/rust_ir.rs", str(self.evidence / "live/capture.json")], env=environment, log=self.evidence / "live-resolver.log").stdout)
        builtin.verify_capture(live, receipt, receipt["inputs"])
        joined = builtin.validate_join(live, common)
        self.require(len(joined) == len(common["common_members"]), "full actual common live surface joined")
        counts = builtin.validate_mapping(live)
        self.require(counts["invocations"] == len(common["invocations"]), "all actual live builtin invocations accounted for")
        receivers = {builtin.encoded(d["receiver_identity"]) for d in live["declarations"]}
        self.require(len(receivers) == len({builtin.encoded(i["receiver"]) for i in common["invocations"]}), "all actual declarations accounted for")
        for name in ("fmt", "clone", "eq"):
            declaration = next(d for d in live["declarations"] if "rust_ir::RustFile as " in d["owner"] and d["owner"].endswith("::" + name))
            self.require(declaration["ast_body"] and declaration["hir_body"] and declaration["typed_sites"], "RustFile." + name + " actual complete typed body")
        (self.evidence / "live-common.json").write_bytes(builtin.encoded(common))
        (self.evidence / "live-join.json").write_bytes(builtin.encoded(joined))
        print("actual live counts:", counts, "receiver_declarations:", len(receivers))
        for owner in ("rust_ir::RustFile as std::fmt::Debug", "rust_ir::Visibility as std::cmp::PartialEq"):
            corrupted = copy.deepcopy(live)
            corrupted["declarations"] = [declaration for declaration in corrupted["declarations"] if not (owner in declaration["owner"] and declaration["owner_kind"] == "AssocFn")]
            self.reject(corrupted, "negative-live-missing-" + owner.split("::")[1].split()[0], receipt=receipt)
            self.assertions += 1
            with self.assertRaises(builtin.Unsupported):
                builtin.validate_join(corrupted, common)
        corrupted = copy.deepcopy(live)
        next(d for d in corrupted["declarations"] if d["typed_sites"])["typed_sites"].pop()
        self.reject(corrupted, "negative-live-dropped-call", receipt=receipt)


if __name__ == "__main__":
    unittest.main()
