"""Actual-producer acceptance for the bounded H03a1p builtin capability."""
import copy
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest

import maintainability_builtin_input as builtin
import dynamic_acceptance
import include_acceptance


def prepared(test, root, package, label, *, whole, test_mode=False, repeat=False):
    key = builtin.digest(builtin.encoded([str(root), package, whole, test_mode, repeat, test.identity, {k:builtin.digest(v.encode()) for k,v in os.environ.items() if not k.startswith("SIFR_BUILTIN_")}]))
    path = Path(os.environ["SIFR_BUILTIN_EVIDENCE_DIR"]) / "prepared" / key
    if (path / "successful.json").is_file():
        binding=json.loads((path / "successful.json").read_text())
        for name,sha in binding.items():
            if builtin.digest((path/name).read_bytes())!=sha:
                raise builtin.Unsupported("changed cached successful producer authority")
        c=json.loads((path/"capture.json").read_text());r=json.loads((path/"receipt.json").read_text());a=builtin.read_inventory(r,r["inputs"])
        builtin.verify_capture(c,r,r["inputs"],a)
        common=json.loads((path/"common.json").read_text())
        builtin.validate_join(c,common,a,receipt=r,input_identity=r["inputs"])
        print("prepared cache HIT",label,r["timing_seconds"])
        return c,r,common
    c,r=builtin.capture_package(root,package,package,path,test.helper,test.target,test.identity,whole=whole,test=test_mode)
    env=os.environ.copy();env.pop("RUSTC_BOOTSTRAP",None);env.update(r["inputs"]["resolver_preparation_environment"])
    suffix=("fixture_root/src/" if package=="builtin_fixture" else f"crates/{package}/src/") if whole else "fixture_root/src/lib.rs" if package=="builtin_fixture" else "crates/sifr_codegen/src/rust_ir.rs"
    command=[str(test.resolver),str(root),package,suffix,str(path/"capture.json")]
    if test_mode:command.append("test")
    result=builtin.run(command,env=env,log=path/"resolver.log")
    common=json.loads(result.stdout);(path/"common.json").write_bytes(builtin.encoded(common))
    a=builtin.read_inventory(r,r["inputs"]);join=builtin.validate_join(c,common,a,receipt=r,input_identity=r["inputs"])
    (path/"join.json").write_bytes(builtin.encoded(join))
    (path/"successful.json").write_bytes(builtin.encoded({name:builtin.digest((path/name).read_bytes()) for name in ("capture.json","receipt.json","inventory-authority.json","common.json","join.json")}))
    print("prepared cache MISS",label,r["timing_seconds"],"resolver",result.elapsed_seconds)
    return c,r,common


class BuiltinCapabilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.target = Path(os.environ["SIFR_BUILTIN_TARGET_DIR"]).resolve()
        cls.helper = cls.target / "debug/sifr_maintainability_builtin_input"
        cls.resolver = cls.target / "debug/ra_common"
        cls.receipt_path = Path(os.environ["SIFR_BUILTIN_COMPONENT_RECEIPT"])
        cls.evidence = Path(tempfile.mkdtemp(prefix="acceptance-", dir=os.environ["SIFR_BUILTIN_EVIDENCE_DIR"]))
        cls.identity = builtin.tool_identity(cls.receipt_path, cls.target)
        cls.fixture, cls.fixture_receipt, cls.common = prepared(cls, builtin.TOOL / "fixtures", "builtin_fixture", "fixture", whole=False)
        cls.inventory = builtin.read_inventory(cls.fixture_receipt, cls.fixture_receipt["inputs"])
        builtin.verify_capture(cls.fixture, cls.fixture_receipt, cls.fixture_receipt["inputs"], cls.inventory)
        cls.join = builtin.validate_join(cls.fixture, cls.common, cls.inventory, receipt=cls.fixture_receipt, input_identity=cls.fixture_receipt["inputs"])
        (cls.evidence / "fixture-join.json").write_bytes(builtin.encoded(cls.join))
        (cls.evidence / "fixture-invocation-join.json").write_bytes(builtin.encoded(builtin.validate_invocation_multisets(cls.fixture,cls.common, receipt=cls.fixture_receipt, input_identity=cls.fixture_receipt["inputs"], authority=cls.inventory)))

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
            builtin.validate_join(corrupted, self.common, self.inventory, receipt=self.fixture_receipt, input_identity=self.fixture_receipt["inputs"])
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
            builtin.validate_join(corrupted, self.common, self.inventory, receipt=self.fixture_receipt, input_identity=self.fixture_receipt["inputs"])


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
        live, receipt, common = prepared(self,builtin.ROOT,"sifr_codegen","live",whole=False)
        inventory = builtin.read_inventory(receipt, receipt["inputs"])
        builtin.verify_capture(live, receipt, receipt["inputs"], inventory)
        joined = builtin.validate_join(live, common, inventory, receipt=receipt, input_identity=receipt["inputs"])
        self.require(len(joined) == len(common["common_members"]), "full actual common live surface joined")
        counts = builtin.validate_mapping(live, inventory, receipt=receipt, input_identity=receipt["inputs"])
        self.require(counts["invocations"] == len(common["invocations"]), "all actual live builtin invocations accounted for")
        receivers = {builtin.encoded(d["receiver_identity"]) for d in live["declarations"]}
        self.require(len(receivers) == len({builtin.encoded(i["receiver"]) for i in common["invocations"]}), "all actual declarations accounted for")
        for name in ("fmt", "clone", "eq"):
            declaration = next(d for d in live["declarations"] if "rust_ir::RustFile as " in d["owner"] and d["owner"].endswith("::" + name))
            self.require(declaration["ast_body"] and declaration["hir_body"] and declaration["typed_sites"], "RustFile." + name + " actual complete typed body")
        (self.evidence / "live-common.json").write_bytes(builtin.encoded(common))
        (self.evidence / "live-join.json").write_bytes(builtin.encoded(joined))
        (self.evidence / "live-invocation-join.json").write_bytes(builtin.encoded(builtin.validate_invocation_multisets(live,common, receipt=receipt, input_identity=receipt["inputs"], authority=inventory)))
        print("actual live counts:", counts, "receiver_declarations:", len(receivers))
        for owner in ("rust_ir::RustFile as std::fmt::Debug", "rust_ir::Visibility as std::cmp::PartialEq"):
            corrupted = copy.deepcopy(live)
            corrupted["declarations"] = [declaration for declaration in corrupted["declarations"] if not (owner in declaration["owner"] and declaration["owner_kind"] == "AssocFn")]
            self.reject(corrupted, "negative-live-missing-" + owner.split("::")[1].split()[0], receipt=receipt, authority=inventory)
            self.assertions += 1
            with self.assertRaises(builtin.Unsupported):
                builtin.validate_join(corrupted, common, inventory, receipt=receipt, input_identity=receipt["inputs"])
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
        receipt = self._live_receipt if authority is not None else self.fixture_receipt
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "invocation|common-member|independent inventory") as caught:
            builtin.validate_join(value, common, authority or self.inventory, receipt=receipt, input_identity=receipt["inputs"])
        (self.evidence / (label + ".json")).write_bytes(builtin.encoded({"capture":value,"common":common,"semantic_rejection":str(caught.exception)}))

    def test_hir_ty_owner_inventory_is_independent_of_ast_projection(self):
        inventory = self.inventory["inventory"]
        self.require(any("local_owner" in d["structural_identity"] and d["disposition"] == "nonselected" for d in inventory["universe"]), "body-nested actual impl remains in universe")
        self.require(len(builtin.validate_invocation_multisets(self.fixture,self.common, receipt=self.fixture_receipt, input_identity=self.fixture_receipt["inputs"], authority=self.inventory)) == len(self.common["invocations"]), "all primary/marker/auxiliary invocation multisets persisted")
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
                builtin.validate_mapping(value, authority, receipt=self.fixture_receipt, input_identity=self.fixture_receipt["inputs"])
            (self.evidence / (label + ".json")).write_bytes(builtin.encoded(authority))
        authority = copy.deepcopy(self.inventory)
        authority["context"]["target"] = "wrong-context"
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "independent inventory context"):
            builtin.validate_mapping(self.fixture, authority, receipt=self.fixture_receipt, input_identity=self.fixture_receipt["inputs"])
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "missing independent inventory"):
            builtin.validate_mapping(self.fixture, None, receipt=self.fixture_receipt, input_identity=self.fixture_receipt["inputs"])
        self.assertions += 1
        with self.assertRaisesRegex(builtin.Unsupported, "missing independent inventory expected authority"):
            builtin.verify_capture(self.fixture,self.fixture_receipt,self.fixture_receipt["inputs"],None)
        common = copy.deepcopy(self.common)
        common["invocations"].append(copy.deepcopy(common["invocations"][0]))
        self.common_reject(self.fixture, common, "duplicate-invocation")

    def test_coordinated_fixture_owner_removals_fail_semantic_admission(self):
        self.require(bool(builtin.validate_join(self.fixture,self.common,self.inventory, receipt=self.fixture_receipt, input_identity=self.fixture_receipt["inputs"])), "actual complete fixture join")
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
        live, receipt, common = prepared(self,builtin.ROOT,"sifr_codegen","live",whole=False)
        authority = builtin.read_inventory(receipt,receipt["inputs"])
        self._live_receipt = receipt
        self.require(bool(builtin.verify_capture(live,receipt,receipt["inputs"],authority)), "actual original live inventory admission")
        join = builtin.validate_join(live,common,authority, receipt=receipt, input_identity=receipt["inputs"])
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
        counts = builtin.validate_mapping(live,authority, receipt=receipt, input_identity=receipt["inputs"])
        counts["receivers"] = len({builtin.encoded(i["receiver"]) for i in common["invocations"]})
        print("actual live inventory counts:",counts,"auxiliaries:",len(auxiliaries))
        (self.evidence / "live-common.json").write_bytes(builtin.encoded(common))
        (self.evidence / "live-join.json").write_bytes(builtin.encoded(join))
        (self.evidence / "live-invocation-join.json").write_bytes(builtin.encoded(builtin.validate_invocation_multisets(live,common, receipt=receipt, input_identity=receipt["inputs"], authority=authority)))


class BuiltinExtensionTests(BuiltinInventoryTests):
    # Inherit assertion helpers, but keep this exact four-case acceptance class.
    test_hir_ty_owner_inventory_is_independent_of_ast_projection = None
    test_coordinated_fixture_owner_removals_fail_semantic_admission = None
    test_ra_invocation_owned_common_members_fail_without_compiler_impl = None
    test_coordinated_live_owner_removals_fail_semantic_admission = None

    @classmethod
    def setUpClass(cls):
        BuiltinCapabilityTests.setUpClass.__func__(cls)
        cls.extended,cls.extended_receipt,cls.extended_common=prepared(cls,builtin.TOOL/"fixtures","builtin_fixture","extension",whole=True)
        cls.extended_authority=builtin.read_inventory(cls.extended_receipt,cls.extended_receipt["inputs"])

    def mutation(self, change, label, *, capture=None, receipt=None, authority=None, common=None):
        original=capture or self.extended;receipt=copy.deepcopy(receipt or self.extended_receipt);authority=authority or self.extended_authority
        value=copy.deepcopy(original);projection=copy.deepcopy(common or self.extended_common)
        change(value,projection)
        receipt["capture_digest"]=builtin.digest(builtin.encoded(value))
        self.require(receipt["capture_digest"]==builtin.digest(builtin.encoded(value)),"recomputed projection integrity")
        self.assertions+=1
        with self.assertRaisesRegex(builtin.Unsupported,"independent inventory|declaration|invocation|consumer|publication|correspondence|substitution|Dynamic|existential|trait-object|schema") as caught:
            builtin.validate_join(value,projection,authority,receipt=receipt,input_identity=receipt["inputs"])
        (self.evidence/(label+".json")).write_bytes(builtin.encoded({"capture":value,"common":projection,"receipt":receipt,"intact_authority_digest":builtin.digest(builtin.encoded(authority)),"semantic_rejection":str(caught.exception)}))

    def test_default_eq_hash_ordering_bodies_and_members(self):
        d=self.extended["declarations"]
        for shape in ("ExtendedRecord","ExtendedTuple","ExtendedUnit","ExtendedEnum"):
            for trait,method in (("Eq","assert_fields_are_eq"),("Ord","cmp"),("PartialOrd","partial_cmp"),("Default","default"),("Hash","hash")):
                member=next(x for x in d if shape+" as " in x["owner"] and x["owner"].endswith("::"+method))
                self.require(member["ast_body"] and member["hir_body"],shape+" "+trait+" real body")
                self.require(member["declaration_facts"]["trait_bridge"]["trait_method"].endswith("::"+method),"actual original trait method relation")
        eq=next(x for x in d if "ExtendedRecord as " in x["owner"] and x["owner"].endswith("::assert_fields_are_eq"))
        self.require("AssertParamIsEq" in eq["tokens"] and eq["body_type_dependencies"],"Eq actual field type dependencies")
        joined=builtin.validate_join(self.extended,self.extended_common,self.extended_authority,receipt=self.extended_receipt,input_identity=self.extended_receipt["inputs"])
        self.require(next(j for j in joined if j["owner"]==eq["owner"])["member_disposition"]=="compiler-only-owned-typed-member","Eq compiler-only generated signature has explicit correspondence")
        self.mutation(lambda c,m:next(x for x in c["declarations"] if x["owner"]==eq["owner"])["body_type_dependencies"].clear(),"missing-Eq-type-dependencies")
        default=next(x for x in d if "ExtendedEnum as " in x["owner"] and x["owner"].endswith("::default"))
        self.require("Empty" in default["tokens"] and "#[default]" in next(i["declaration_source"] for i in self.extended_common["invocations"] if i["source_name"]=="ExtendedEnum"),"real default unit variant attribute")
        sites=[s for x in d for s in x["typed_sites"]]
        self.require(any(s["target"] and s["target"].endswith("Hash::hash") for s in sites),"resolved generic Hash field/discriminant calls")
        self.require(len(builtin.validate_invocation_multisets(self.extended,self.extended_common,receipt=self.extended_receipt,input_identity=self.extended_receipt["inputs"],authority=self.extended_authority))==len(self.extended_common["invocations"]),"complete original owned multisets")
        observation=builtin.consume(self.extended,self.extended_receipt,self.extended_receipt["inputs"],self.extended_authority,("call-count","resolved-module-fanout"))
        self.require(bool(observation["body_erasure_observations"]),"authentic published body erasure labeled with call owner and stage")
        for label,mutate in (("erased-static",lambda n:n.update(kind="ReStatic")),("erased-named",lambda n:n.update(kind="ReEarlyParam",index=0,name="'a"))):
            def change(c,common):
                node=next(n for x in c["declarations"] for s in x["typed_sites"] for n in builtin.declaration_consumer.nodes(s) if n.get("kind")=="ReErased");mutate(node)
            self.mutation(change,label)
        def changed_arguments(c,common):
            site=next(s for x in c["declarations"] for s in x["typed_sites"] if s["published_arguments"])
            site["published_arguments"].append({"const":{"kind":"Value(9)"}})
        self.mutation(changed_arguments,"changed-type-const-arguments")
        def changed_type_argument(c,common):
            argument=next(n for d in c["declarations"] for s in d["typed_sites"] for n in s["published_arguments"] or [] if "type" in n)
            argument["type"]={"builtin":"invented-type"}
        self.mutation(changed_type_argument,"changed-actual-type-argument")
        self.mutation(lambda c,m:next(x for x in c["declarations"] if x["owner"]==eq["owner"]).update(hir_body=False),"falsely-bodyless-Eq")

        dynamic_acceptance.positive(self)
        include_acceptance.positive(self)

    def test_lifetime_receiver_and_method_generics_preserve_constraints(self):
        declarations=self.extended["declarations"]
        owned=next(d for d in declarations if "TwoLifetimes" in d["owner"] and d["owner"].endswith("::hash"))
        facts=owned["declaration_facts"]
        self.require([p["kind"] for p in facts["generics"]["parent"]["own"]]==["lifetime","lifetime","type"],"ordered inherited lifetime/type parameters")
        self.require(facts["generics"]["own"][0]["name"]=="__H","Hash method-own parameter")
        predicates=list(builtin.declaration_consumer.nodes(facts["predicates"]))
        self.require(any("region_outlives" in p for p in predicates) and any("type_outlives" in p for p in predicates),"region/type outlives facts")
        self.require({p["trait"] for p in predicates if "trait" in p}.issuperset({"core::hash::Hasher","core::marker::Sized"}),"actual Hasher/Sized constraints")
        self.require(any(n.get("kind")=="ReStatic" for n in builtin.declaration_consumer.nodes(facts["original"]["field_types"])),"static field distinguished")
        self.require(any(n.get("kind")=="ReBound" for n in builtin.declaration_consumer.nodes(facts["signature"])),"bound method reference regions retained")
        self.require(facts["signature"]["binders"],"method binder variables retained")
        join=builtin.validate_join(self.extended,self.extended_common,self.extended_authority,receipt=self.extended_receipt,input_identity=self.extended_receipt["inputs"])
        bridge=next(j["declaration_correspondence"] for j in join if j["owner"]==owned["owner"])
        self.require(bridge["compiler_trait_substitution"]["alpha_parameters"][0]["trait_name"]=="H","verified H to __H alpha relation")
        self.require(bridge["original_adt"]["kind"]=="ra-semantic-declaration-and-source-correspondence","RA authority labeled accurately")
        for label,mutate in (("missing-ra-lifetime",lambda facts:facts["lifetimes"].pop()),("wrong-ra-lifetime-owner",lambda facts:next(n for n in facts["lifetimes"] if n["disposition"]["kind"]=="parameter")["disposition"].update(index=999))):
            self.mutation(lambda c,m:mutate(next(i for i in m["invocations"] if i["source_name"]=="TwoLifetimes")["declaration_facts"]),label)
        negative_root=self.evidence/"const-generic-fixture"
        shutil.copytree(builtin.TOOL/"fixtures",negative_root)
        source=negative_root/"fixture_root/src/lib.rs"
        source.write_text(source.read_text()+"\n#[derive(Debug, Clone)] pub struct ConstGeneric<const N: usize> { pub items: [u8; N] }\n")
        destination=self.evidence/"const-generic-capture"
        self.assertions+=1
        with self.assertRaisesRegex(builtin.Unsupported,"unsupported"):
            builtin.capture_package(negative_root,"builtin_fixture","builtin_fixture",destination,self.helper,self.target,self.identity,whole=True)
        self.require(not (destination/"capture.json").exists(),"unsupported const generic impl shape publishes no accepted export")
        self.assertions+=1
        with self.assertRaisesRegex(builtin.Unsupported,"capability unresolved"):
            builtin.consume(self.extended,self.extended_receipt,self.extended_receipt["inputs"],self.extended_authority,("lifetime-sensitive",))
        mutations=(
            ("declaration-erasure",lambda f:next(n for n in builtin.declaration_consumer.nodes(f) if n.get("kind")=="ReEarlyParam").update(kind="ReErased")),
            ("bound-depth",lambda f:next(n for n in builtin.declaration_consumer.nodes(f["signature"]) if n.get("kind")=="ReBound").update(depth="INNERMOST+1")),
            ("static-param",lambda f:next(n for n in builtin.declaration_consumer.nodes(f["original"]) if n.get("kind")=="ReStatic").update(kind="ReEarlyParam",index=0,name="'a")),
            ("missing-own-parameter",lambda f:f["generics"]["own"].clear()),
            ("missing-own-Hasher",lambda f:f["predicates"]["own"].pop()),
            ("missing-outlives",lambda f:f["predicates"]["parent"]["own"].clear()),
            ("wrong-method-owner",lambda f:f["generics"].update(owner="wrong::owner")),
            ("swapped-receiver-arguments",lambda f:f["receiver"]["arguments"].reverse()),
            ("missing-binder",lambda f:f["signature"]["binders"].clear()),
        )
        for label,mutate in mutations:
            def change(c,common):
                mutate(next(d for d in c["declarations"] if d["owner"]==owned["owner"])["declaration_facts"])
                for member in common["common_members"]:
                    if member["method"]=="hash" and "TwoLifetimes" in member["receiver_identity"]["adt"]:
                        member["trait_declaration_facts"]["parameters"].clear()
                        member["generic_bounds"].clear()
            self.mutation(change,label)

        dynamic_acceptance.method_binders(self)
        dynamic_acceptance.source_binder_negative(self)

    def test_extended_inventory_owner_and_constraint_removals_fail_closed(self):
        for trait in ("Eq","Ord","PartialOrd","Default","Hash","Clone","Copy","TrivialClone"):
            owners={d["owner"] for d in self.extended["declarations"] if "Lifetime" in d["owner"] and ("::"+trait+">") in d["owner"]}
            self.require(bool(owners),"actual extended mutation owner "+trait)
            def change(c,common):
                c["declarations"]=[d for d in c["declarations"] if d["owner"] not in owners]
                c["expanded_owner_ledger"]=[d for d in c["expanded_owner_ledger"] if d["owner"] not in owners]
                common["common_members"]=[m for m in common["common_members"] if not (m["trait_identity"].endswith("::"+trait) and "Lifetime" in m["receiver_identity"]["adt"])]
            self.mutation(change,"coordinated-extended-"+trait)
        self.mutation(lambda c,m:next(d for d in c["declarations"] if d["typed_sites"])["typed_sites"].pop(),"missing-owned-call")
        for hook in ("<absent as hook>",next(d["owner"] for d in self.extended["declarations"] if d["owner_kind"]=="AssocFn")):
            old=os.environ.get("SIFR_BUILTIN_OMIT_AST_OWNER")
            try:
                os.environ["SIFR_BUILTIN_OMIT_AST_OWNER"]=hook
                self.assertions+=1
                with self.assertRaisesRegex(builtin.Unsupported,"omitted-AST"):
                    builtin.validate_join(self.extended,self.extended_common,self.extended_authority,receipt=self.extended_receipt,input_identity=self.extended_receipt["inputs"])
            finally:
                if old is None:os.environ.pop("SIFR_BUILTIN_OMIT_AST_OWNER",None)
                else:os.environ["SIFR_BUILTIN_OMIT_AST_OWNER"]=old
        for operation in (lambda:builtin._validate_join(self.extended,self.extended_common,self.extended_authority),lambda:builtin.validate_join(self.extended,self.extended_common,self.extended_authority),lambda:builtin.validate_mapping(self.extended,self.extended_authority),lambda:builtin.validate_invocation_multisets(self.extended,self.extended_common)):
            self.assertions+=1
            with self.assertRaisesRegex(builtin.Unsupported,"unauthenticated"):operation()
        for field in ("derive_ordinal","attribute_ordinal"):
            self.mutation(lambda c,m:m["invocations"][0].update({field:999}),"swapped-"+field)
        self.mutation(lambda c,m:next(d for d in c["declarations"] if d["owner_kind"]=="AssocFn")["expansion_chain"][0]["call_site"].update(start=[999,0]),"swapped-callsite")
        replacement=copy.deepcopy(self.extended_authority);replacement["inventory"]["owners"].pop()
        self.assertions+=1
        with self.assertRaisesRegex(builtin.Unsupported,"replacement expected authority"):
            builtin.verify_capture(self.extended,self.extended_receipt,self.extended_receipt["inputs"],replacement)

        dynamic_acceptance.negatives(self)
        include_acceptance.negatives(self)

    def test_live_owned_derive_inventory_has_complete_dispositions(self):
        contexts=[];all_kinds=set();live_erased=[]
        for package in ("sifr_codegen","sifr_lowering"):
            for mode in (False,True):
                c,r,m=prepared(self,builtin.ROOT,package,package+("-test" if mode else "-production"),whole=True,test_mode=mode)
                a=builtin.read_inventory(r,r["inputs"])
                self.require(r["preparation_cargo_artifact_success"] and not r["capture_cargo_artifact_success"],"actual locked selected original Cargo preparation")
                self.require(c["context"]["test"]==mode and c["context"]["target"]=="x86_64-unknown-linux-gnu","exact original context")
                all_kinds.update(d["expansion_chain"][0]["macro_identity"] for d in c["declarations"])
                observation=builtin.consume(c,r,r["inputs"],a,("call-count","resolved-module-fanout"));live_erased.extend(observation["body_erasure_observations"])
                self.require(len(builtin.validate_invocation_multisets(c,m,receipt=r,input_identity=r["inputs"],authority=a))==len(m["invocations"]),"all live invocation multisets")
                self.require(any(n.get("kind")=="ReEarlyParam" for d in c["declarations"] for n in builtin.declaration_consumer.nodes(d["declaration_facts"])),"actual live declaration lifetime shapes")
                for label,change in (("call",lambda v,p:next(d for d in v["declarations"] if d["typed_sites"])["typed_sites"].pop()),("owner",lambda v,p:v["declarations"].pop()),("generic",lambda v,p:next(d for d in v["declarations"] if d["declaration_facts"]["generics"]["own"])["declaration_facts"]["generics"]["own"].clear())):
                    self.mutation(change,package+str(mode)+label,capture=c,receipt=r,authority=a,common=m)
                self.require(not any(cfg["key"]=="rust_analyzer" for cfg in c["cfg"]),"rust_analyzer disabled in original selected cfg")
                repeated,rr,rm=prepared(self,builtin.ROOT,package,package+str(mode)+"-repeat",whole=True,test_mode=mode,repeat=True)
                ra=builtin.read_inventory(rr,rr["inputs"])
                self.require(bool(builtin.verify_capture(repeated,rr,rr["inputs"],ra)),"second unchanged-path authentic capture")
                self.require(dynamic_acceptance.semantic_capture(repeated)==dynamic_acceptance.semantic_capture(c),"twice unchanged-path complete semantic capture")
                self.require(rm==m,"twice unchanged-path complete RA/source correspondence")
                contexts.append((c,r,r["inputs"],a))
                (self.evidence/(package+str(mode)+"-counts.json")).write_bytes(builtin.encoded({"counts":r["counts"],"erasure":observation,"context":c["context"]}))
        self.require(all_kinds==set(builtin.COMMON_DERIVES),"fresh whole contexts discover all nine resolved kinds")
        self.require(any("SimpleStmtLoweringCtx" in e["owner"] for e in live_erased) and sum("SelectedDeclarations" in e["owner"] for e in live_erased)>=3,"authentic four Default erased observations retained")
        self.require(bool(builtin.validate_contexts(contexts)),"four whole contexts complete")
        self.assertions+=1
        with self.assertRaisesRegex(builtin.Unsupported,"context completeness"):builtin.validate_contexts(contexts[:-1])
        c,r,m=prepared(self,builtin.TOOL/"fixtures","builtin_fixture","extension-test",whole=True,test_mode=True)
        self.require(any("TestOnly" in d["owner"] for d in c["declarations"]),"actual test-only fixture selected")
        other=self.evidence/"relocated-fixtures"
        shutil.copytree(builtin.TOOL/"fixtures",other)
        repeated,repeat_receipt,repeat_common=prepared(self,other,"builtin_fixture","relocated-extension",whole=True)
        self.require(dynamic_acceptance.semantic_capture(repeated)==dynamic_acceptance.semantic_capture(self.extended),"unchanged authentic capture normalizes across checkout paths")
        self.require(builtin.normalize(repeat_common,other)==builtin.normalize(self.extended_common,builtin.TOOL/"fixtures"),"relocated complete original RA declaration correspondence")


if __name__ == "__main__":
    unittest.main()
