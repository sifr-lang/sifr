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
        cls.inventory = builtin.read_inventory(cls.fixture_receipt, cls.fixture_receipt["inputs"])
        environment = os.environ.copy()
        environment.pop("RUSTC_BOOTSTRAP", None)
        environment.update(cls.fixture_receipt["inputs"]["resolver_preparation_environment"])
        common = builtin.run(
            [str(cls.resolver), str(builtin.TOOL / "fixtures"), "builtin_fixture", "fixture_root/src/lib.rs", str(cls.evidence / "fixture/capture.json")],
            env=environment, log=cls.evidence / "fixture-resolver.log",
        )
        cls.common = json.loads(common.stdout)
        (cls.evidence / "fixture-common.json").write_bytes(builtin.encoded(cls.common))
        builtin.verify_capture(cls.fixture, cls.fixture_receipt, cls.fixture_receipt["inputs"], cls.inventory)
        cls.join = builtin.validate_join(cls.fixture, cls.common, cls.inventory)
        (cls.evidence / "fixture-join.json").write_bytes(builtin.encoded(cls.join))
        (cls.evidence / "fixture-invocation-join.json").write_bytes(builtin.encoded(builtin.validate_invocation_multisets(cls.fixture,cls.common)))

    def setUp(self):
        self.assertions = 0

    def tearDown(self):
        print(f"{self.id()}: assertions={self.assertions}; evidence={self.evidence}")

    def require(self, condition, reason):
        self.assertions += 1
        self.assertTrue(condition, reason)

    def reject(self, value, label, *, receipt=None, authority=None):
        self.assertions += 1
        evidence_receipt = copy.deepcopy(receipt or self.fixture_receipt)
        evidence_receipt["capture_digest"] = builtin.digest(builtin.encoded(value))
        with self.assertRaises(builtin.Unsupported):
            builtin.verify_capture(value, evidence_receipt, (receipt or self.fixture_receipt)["inputs"], authority or self.inventory)

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
            builtin.validate_join(corrupted, self.common, self.inventory)
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
            builtin.validate_join(corrupted, self.common, self.inventory)


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
        self.require(builtin.verify_capture(self.fixture, self.fixture_receipt, self.fixture_receipt["inputs"], self.inventory)["invocations"] > 0, "exact known context admitted")
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
                builtin.verify_capture(self.fixture, self.fixture_receipt, current, self.inventory)
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
                    builtin.verify_capture(self.fixture, self.fixture_receipt, inputs, self.inventory)
            finally:
                path.write_bytes(original)
                os.utime(path, ns=(stat.st_atime_ns, stat.st_mtime_ns))
            self.require(builtin.verify_capture(self.fixture, self.fixture_receipt, inputs, self.inventory)["invocations"] > 0, label + " exact restoration")
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
        inventory = builtin.read_inventory(receipt, receipt["inputs"])
        builtin.verify_capture(live, receipt, receipt["inputs"], inventory)
        joined = builtin.validate_join(live, common, inventory)
        self.require(len(joined) == len(common["common_members"]), "full actual common live surface joined")
        counts = builtin.validate_mapping(live, inventory)
        self.require(counts["invocations"] == len(common["invocations"]), "all actual live builtin invocations accounted for")
        receivers = {builtin.encoded(d["receiver_identity"]) for d in live["declarations"]}
        self.require(len(receivers) == len({builtin.encoded(i["receiver"]) for i in common["invocations"]}), "all actual declarations accounted for")
        for name in ("fmt", "clone", "eq"):
            declaration = next(d for d in live["declarations"] if "rust_ir::RustFile as " in d["owner"] and d["owner"].endswith("::" + name))
            self.require(declaration["ast_body"] and declaration["hir_body"] and declaration["typed_sites"], "RustFile." + name + " actual complete typed body")
        (self.evidence / "live-common.json").write_bytes(builtin.encoded(common))
        (self.evidence / "live-join.json").write_bytes(builtin.encoded(joined))
        (self.evidence / "live-invocation-join.json").write_bytes(builtin.encoded(builtin.validate_invocation_multisets(live,common)))
        print("actual live counts:", counts, "receiver_declarations:", len(receivers))
        for owner in ("rust_ir::RustFile as std::fmt::Debug", "rust_ir::Visibility as std::cmp::PartialEq"):
            corrupted = copy.deepcopy(live)
            corrupted["declarations"] = [declaration for declaration in corrupted["declarations"] if not (owner in declaration["owner"] and declaration["owner_kind"] == "AssocFn")]
            self.reject(corrupted, "negative-live-missing-" + owner.split("::")[1].split()[0], receipt=receipt, authority=inventory)
            self.assertions += 1
            with self.assertRaises(builtin.Unsupported):
                builtin.validate_join(corrupted, common, inventory)
        corrupted = copy.deepcopy(live)
        next(d for d in corrupted["declarations"] if d["typed_sites"])["typed_sites"].pop()
        self.reject(corrupted, "negative-live-dropped-call", receipt=receipt, authority=inventory)


class BuiltinInventoryTests(unittest.TestCase):
    setUpClass = classmethod(BuiltinCapabilityTests.setUpClass.__func__)
    setUp = BuiltinCapabilityTests.setUp
    tearDown = BuiltinCapabilityTests.tearDown
    require = BuiltinCapabilityTests.require
    method = BuiltinCapabilityTests.method

    def remove_owners(self, capture, selected):
        value = copy.deepcopy(capture)
        owners = {d["owner"] for d in value["declarations"] if selected(d)}
        value["declarations"] = [d for d in value["declarations"] if d["owner"] not in owners]
        value["expanded_owner_ledger"] = [d for d in value["expanded_owner_ledger"] if d["owner"] not in owners]
        self.require(bool(owners), "actual producer has mutation targets")
        return value

    def semantic_reject(self, value, label, receipt=None, authority=None):
        receipt = copy.deepcopy(receipt or self.fixture_receipt)
        receipt["capture_digest"] = builtin.digest(builtin.encoded(value))
        self.require(receipt["capture_digest"] == builtin.digest(builtin.encoded(value)), "integrity recomputed before semantic admission")
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "independent inventory") as caught:
            builtin.verify_capture(value, receipt, receipt["inputs"], authority or self.inventory)
        (self.evidence / (label + ".json")).write_bytes(builtin.encoded({"capture":value,"receipt":receipt,"semantic_rejection":str(caught.exception),"integrity_passed":True}))

    def common_reject(self, value, common, label, authority=None):
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "invocation|common-member") as caught:
            builtin.validate_join(value, common, authority or self.inventory)
        (self.evidence / (label + ".json")).write_bytes(builtin.encoded({"capture":value,"common":common,"semantic_rejection":str(caught.exception)}))

    def test_hir_ty_owner_inventory_is_independent_of_ast_projection(self):
        inventory = self.inventory["inventory"]
        self.require(any("local_owner" in d["structural_identity"] and d["disposition"] == "nonselected" for d in inventory["universe"]), "body-nested actual impl remains in universe")
        self.require(len(builtin.validate_invocation_multisets(self.fixture,self.common)) == len(self.common["invocations"]), "all primary/marker/auxiliary invocation multisets persisted")
        self.require(inventory["trait_impl_cross_check"], "actual HIR/type universe cross-check")
        self.require(len(inventory["owners"]) == len(self.fixture["declarations"]), "complete compiler owner/member inventory")
        self.require(len(inventory["universe"]) > len([d for d in inventory["owners"] if d["owner_kind"].startswith("Impl")]), "preselection retains source/nonselected impls")
        self.require(any(d["disposition"] == "nonselected" for d in inventory["universe"]), "explicit nonselected disposition")
        for receiver in ("Record", "Tuple", "Unit", "Variants", "Nested", "left::Same", "right::Same", "CopySensitive"):
            self.require(any(receiver + " as " in d["owner"] for d in inventory["owners"]), receiver + " actual compiler impl")
        self.require(any("StructuralPartialEq>" in d["owner"] and d["hir_members"] == [] for d in inventory["owners"]), "actual bodyless marker inventory")
        self.require(any("TrivialClone>" in d["owner"] and d["hir_members"] == [] for d in inventory["owners"]), "actual Copy-sensitive auxiliary inventory")
        owner = next(d["owner"] for d in inventory["owners"] if d["owner"].startswith("<Record as std::cmp::PartialEq>") and d["owner_kind"].startswith("Impl"))
        original = os.environ.get("SIFR_BUILTIN_OMIT_AST_OWNER")
        try:
            os.environ["SIFR_BUILTIN_OMIT_AST_OWNER"] = owner
            self.assertions += 1
            with self.assertRaisesRegex(builtin.Unsupported, "inventory/expanded AST owner mismatch before export"):
                builtin.capture_package(builtin.TOOL / "fixtures", "builtin_fixture", "builtin_fixture", self.evidence / "omitted-projection", self.helper, self.target, self.identity)
            self.require(not (self.evidence / "omitted-projection/raw.json").exists(), "producer publishes no reconciled output after omitted AST impl")
        finally:
            if original is None:
                os.environ.pop("SIFR_BUILTIN_OMIT_AST_OWNER", None)
            else:
                os.environ["SIFR_BUILTIN_OMIT_AST_OWNER"] = original
        mutations = (
            ("empty-owners", lambda i: i["owners"].clear()),
            ("empty-universe", lambda i: i["universe"].clear()),
            ("duplicate-owner", lambda i: i["owners"].append(copy.deepcopy(i["owners"][0]))),
            ("duplicate-member", lambda i: next(d for d in i["owners"] if d["hir_members"])["hir_members"].append(next(d for d in i["owners"] if d["hir_members"])["hir_members"][0])),
            ("swapped-parent", lambda i: next(d for d in i["owners"] if d["owner_kind"] == "AssocFn").update(parent="other::impl")),
            ("swapped-receiver", lambda i: i["owners"][0].update(receiver_identity={"adt":"other::Record","arguments":[]})),
            ("swapped-invocation", lambda i: i["owners"][0]["expansion_chain"][0].update(macro_identity="core::cmp::PartialEq")),
            ("unknown-provenance", lambda i: i["universe"][0].update(disposition="unsupported",reason="unknown origin")),
        )
        for label, mutation in mutations:
            authority = copy.deepcopy(self.inventory)
            mutation(authority["inventory"])
            value = copy.deepcopy(self.fixture)
            value["inventory_digest"] = builtin.digest(builtin.encoded(authority["inventory"]))
            self.assertions += 1
            with self.assertRaisesRegex(builtin.Unsupported, "independent inventory"):
                builtin.validate_mapping(value, authority)
            (self.evidence / (label + ".json")).write_bytes(builtin.encoded(authority))
        authority = copy.deepcopy(self.inventory)
        authority["context"]["target"] = "wrong-context"
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "independent inventory context"):
            builtin.validate_mapping(self.fixture, authority)
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "missing independent inventory"):
            builtin.validate_mapping(self.fixture, None)
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "missing independent inventory expected authority"):
            builtin.verify_capture(self.fixture,self.fixture_receipt,self.fixture_receipt["inputs"],None)
        common = copy.deepcopy(self.common)
        common["invocations"].append(copy.deepcopy(common["invocations"][0]))
        self.common_reject(self.fixture, common, "duplicate-invocation")

    def test_coordinated_fixture_owner_removals_fail_semantic_admission(self):
        self.require(bool(builtin.validate_join(self.fixture,self.common,self.inventory)), "actual complete fixture join")
        primary = {d["owner"] for d in self.fixture["declarations"] if d["owner_kind"].startswith("Impl") and d["trait_identity"] == builtin.COMMON_DERIVES[d["expansion_chain"][0]["macro_identity"]][0]}
        first_invocation = self.fixture["declarations"][0]["expansion_chain"]
        for label, selected in (
            ("Record-PartialEq-and-eq", lambda d: d["owner"].startswith("<Record as std::cmp::PartialEq>")),
            ("Record-StructuralPartialEq", lambda d: d["owner"] == "<Record as std::marker::StructuralPartialEq>"),
            ("CopySensitive-TrivialClone", lambda d: "CopySensitive as std::clone::TrivialClone>" in d["owner"]),
            ("all-primary-impls-and-methods", lambda d: d["owner"] in primary or d["parent"] in primary),
            ("all-markers", lambda d: "StructuralPartialEq>" in d["owner"]),
            ("all-auxiliaries", lambda d: "TrivialClone>" in d["owner"]),
            ("one-method", lambda d: d["owner"] == self.method("Record","eq")["owner"]),
            ("one-invocation-all-owners", lambda d: d["expansion_chain"] == first_invocation),
        ):
            value = self.remove_owners(self.fixture, selected)
            self.semantic_reject(value, "coordinated-fixture-" + label)
        reference = self.fixture_receipt["inputs"]["inventory_authority"]
        path = Path(reference["path"])
        original = path.read_bytes()
        try:
            for changed in (b"{}", builtin.encoded({**self.inventory,"inventory":{**self.inventory["inventory"],"owners":self.inventory["inventory"]["owners"][:-1]}})):
                path.write_bytes(changed)
                self.assertions += 1
                with self.assertRaisesRegex(builtin.Unsupported,"changed/truncated independent inventory"):
                    builtin.verify_capture(self.fixture,self.fixture_receipt,self.fixture_receipt["inputs"],self.inventory)
            forged = copy.deepcopy(self.fixture_receipt)
            forged["inventory_authority"]["digest"] = builtin.digest(path.read_bytes())
            forged["inputs"]["inventory_authority"] = forged["inventory_authority"]
            self.assertions += 1
            with self.assertRaisesRegex(builtin.Unsupported,"source/extern/configuration/context drift"):
                builtin.verify_capture(self.fixture,forged,self.fixture_receipt["inputs"],self.inventory)
            replacement = copy.deepcopy(self.inventory)
            removed = next(d["owner"] for d in replacement["inventory"]["owners"] if "Record as std::marker::StructuralPartialEq>" in d["owner"])
            replacement["inventory"]["owners"] = [d for d in replacement["inventory"]["owners"] if d["owner"] != removed]
            replacement["inventory"]["universe"] = [d for d in replacement["inventory"]["universe"] if d["owner"] != removed]
            path.write_bytes(builtin.encoded(replacement))
            forged = copy.deepcopy(self.fixture_receipt)
            forged["inventory_authority"]["digest"] = builtin.digest(path.read_bytes())
            forged["inputs"]["inventory_authority"] = forged["inventory_authority"]
            value = self.remove_owners(self.fixture, lambda d: d["owner"] == removed)
            value["inventory_digest"] = builtin.digest(builtin.encoded(replacement["inventory"]))
            forged["capture_digest"] = builtin.digest(builtin.encoded(value))
            self.assertions += 1
            with self.assertRaisesRegex(builtin.Unsupported,"replacement expected authority") as caught:
                builtin.verify_capture(value,forged,forged["inputs"],self.inventory)
            (self.evidence / "self-consistent-replacement-receipt.json").write_bytes(builtin.encoded({"capture":value,"receipt":forged,"replacement":replacement,"semantic_rejection":str(caught.exception),"integrity_passed":True}))
            path.unlink()
            self.assertions += 1
            with self.assertRaisesRegex(builtin.Unsupported,"changed/truncated independent inventory"):
                builtin.verify_capture(self.fixture,self.fixture_receipt,self.fixture_receipt["inputs"],self.inventory)
        finally:
            path.write_bytes(original)
        self.require(bool(builtin.verify_capture(self.fixture,self.fixture_receipt,self.fixture_receipt["inputs"],self.inventory)), "authentic authority restored")

    def test_ra_invocation_owned_common_members_fail_without_compiler_impl(self):
        self.require(len(self.join) == len([d for d in self.fixture["declarations"] if d["owner_kind"] == "AssocFn"]), "complete actual common signatures")
        for trait in ("PartialEq", "Clone", "Debug"):
            value = self.remove_owners(self.fixture, lambda d: d["owner"].startswith("<Record as ") and ("::" + trait + ">") in d["owner"])
            self.common_reject(value,self.common,"ra-missing-entire-" + trait)
            self.semantic_reject(value,"ra-integrity-missing-entire-" + trait)
        value = self.remove_owners(self.fixture,lambda d: d["owner"] == self.method("Record","fmt")["owner"])
        self.common_reject(value,self.common,"ra-missing-one-common-method")
        chain = self.method("Record","eq")["expansion_chain"]
        value = self.remove_owners(self.fixture,lambda d: d["expansion_chain"] == chain)
        self.common_reject(value,self.common,"ra-missing-all-invocation-output")
        for label, mutate in (
            ("duplicate-member", lambda c: c["common_members"].append(copy.deepcopy(c["common_members"][0]))),
            ("swapped-member", lambda c: c["common_members"][0].update(invocation=c["common_members"][1]["invocation"])),
            ("swapped-invocation", lambda c: c["invocations"][0].update(receiver=next(i["receiver"] for i in c["invocations"] if i["receiver"] != c["invocations"][0]["receiver"]))),
            ("missing-ra-common", lambda c: c["common_members"].pop()),
        ):
            common = copy.deepcopy(self.common)
            mutate(common)
            self.common_reject(self.fixture,common,"ra-" + label)
        self.require(all(i["macro"] != "builtin_fixture::Debug" for i in self.common["invocations"]), "same-spelled local macro isolated by resolved authority")
        self.require(all(s["trait"] and s["trait"].endswith("::Clone") for s in self.method("ExternalFields","clone")["typed_sites"]), "same-spelled inherent methods isolated")

    def test_coordinated_live_owner_removals_fail_semantic_admission(self):
        live, receipt = builtin.capture_package(builtin.ROOT,"sifr_codegen","sifr_codegen",self.evidence / "live",self.helper,self.target,self.identity)
        environment = os.environ.copy()
        environment.pop("RUSTC_BOOTSTRAP",None)
        environment.update(receipt["inputs"]["resolver_preparation_environment"])
        common = json.loads(builtin.run([str(self.resolver),str(builtin.ROOT),"sifr_codegen","crates/sifr_codegen/src/rust_ir.rs",str(self.evidence / "live/capture.json")],env=environment,log=self.evidence / "live-resolver.log").stdout)
        authority = builtin.read_inventory(receipt,receipt["inputs"])
        self.require(bool(builtin.verify_capture(live,receipt,receipt["inputs"],authority)), "actual original live inventory admission")
        join = builtin.validate_join(live,common,authority)
        self.require(len(join) == len(common["common_members"]), "all actual live common methods joined")
        primary = {d["owner"] for d in live["declarations"] if d["owner_kind"].startswith("Impl") and d["trait_identity"] == builtin.COMMON_DERIVES[d["expansion_chain"][0]["macro_identity"]][0]}
        chain = live["declarations"][0]["expansion_chain"]
        for label, selected in (
            ("Visibility-PartialEq-and-eq",lambda d: d["owner"].startswith("<rust_ir::Visibility as std::cmp::PartialEq>")),
            ("all-StructuralPartialEq",lambda d: "StructuralPartialEq>" in d["owner"]),
            ("all-primary-impls-and-methods",lambda d: d["owner"] in primary or d["parent"] in primary),
            ("one-invocation-all-owners",lambda d: d["expansion_chain"] == chain),
        ):
            value = self.remove_owners(live,selected)
            self.semantic_reject(value,"coordinated-live-" + label,receipt,authority)
            if label != "all-StructuralPartialEq":
                self.common_reject(value,common,"common-live-" + label,authority)
        auxiliaries = [d for d in authority["inventory"]["owners"] if "TrivialClone>" in d["owner"]]
        if auxiliaries:
            value = self.remove_owners(live,lambda d: "TrivialClone>" in d["owner"])
            self.semantic_reject(value,"coordinated-live-all-auxiliaries",receipt,authority)
        else:
            self.require(not any("TrivialClone>" in d["owner"] for d in live["declarations"]), "actual live auxiliary absence agrees with compiler inventory")
        counts = builtin.validate_mapping(live,authority)
        counts["receivers"] = len({builtin.encoded(i["receiver"]) for i in common["invocations"]})
        print("actual live inventory counts:",counts,"auxiliaries:",len(auxiliaries))
        (self.evidence / "live-common.json").write_bytes(builtin.encoded(common))
        (self.evidence / "live-join.json").write_bytes(builtin.encoded(join))
        (self.evidence / "live-invocation-join.json").write_bytes(builtin.encoded(builtin.validate_invocation_multisets(live,common)))


if __name__ == "__main__":
    unittest.main()
