"""Actual nested include source authority and semantic mutations."""
import maintainability_builtin_input as b


def invocations(test):
    return [i for i in test.extended_common["invocations"] if i["source_name"] == "Included"]


def positive(test):
    original = invocations(test)
    test.require(len(original) == 3, "actual included Debug/Clone/PartialEq invocations")
    for invocation in original:
        mapping = invocation["include_source_mapping"]
        test.require(mapping is not None and mapping["kind"] == "ra-public-include-token-descent-and-to-def", "public span-token descent and semantic identity")
        test.require(mapping["receiver"] == invocation["receiver"] and mapping["source_range"] == invocation["source_range"], "semantic included receiver and physical source range")
        test.require(mapping["expanded_source_tokens"] == invocation["declaration_facts"]["source_tokens"], "exact expanded/physical source token correspondence")
        test.require(mapping["file"].endswith("/included_shapes.rs") and invocation["invocation_site"]["file"] == mapping["file"], "actual innermost included physical source")
        owners = [d for d in test.extended["declarations"] if d["receiver_identity"] == invocation["receiver"] and d["expansion_chain"][0]["macro_identity"] == invocation["macro"]]
        test.require(bool(owners) and all(sum(c["macro_identity"] == "core::macros::builtin::include" for c in d["expansion_chain"]) == 2 for d in owners), "original compiler nested include chain retained")
        test.require(all(d["expansion_chain"][0]["call_site"]["quality"] == "exact-source" for d in owners), "exact compiler derive source authority")
    context = test.extended_common["context"]
    test.require(context["kind"] == "ra-semantic-selected-cargo-target-root" and context["crate"] == "builtin_fixture", "selected semantic crate bound to actual Cargo target root")
    joined = b.validate_join(test.extended, test.extended_common, test.extended_authority, receipt=test.extended_receipt, input_identity=test.extended_receipt["inputs"])
    test.require(any("included_contracts::Included" in d["owner"] for d in joined), "actual included compiler/RA methods joined")


def negatives(test):
    def selected(common):
        return next(i for i in common["invocations"] if i["source_name"] == "Included")
    for name, change in (
        ("omission", lambda i: i.pop("include_source_mapping")),
        ("empty", lambda i: i.update(include_source_mapping=None)),
        ("identity", lambda i: i["include_source_mapping"].update(receiver={"adt": "forged::Included", "arguments": []})),
        ("file", lambda i: i["include_source_mapping"].update(file="/forged/included_shapes.rs")),
        ("range", lambda i: i["include_source_mapping"].update(source_range="0..1")),
        ("expanded-tokens", lambda i: i["include_source_mapping"]["expanded_source_tokens"].pop()),
        ("kind", lambda i: i["include_source_mapping"].update(kind="spelling-only")),
        ("extra", lambda i: i["include_source_mapping"].update(fallback=True)),
        ("physical-range", lambda i: i.update(source_range="0..1")),
        ("derive-ordinal", lambda i: i.update(derive_ordinal=99)),
        ("attribute-ordinal", lambda i: i.update(attribute_ordinal=99)),
    ):
        test.mutation(lambda c, m, change=change: change(selected(m)), "included-source-" + name)
    for name, change in (
        ("omitted", lambda m: m.pop("context")),
        ("empty", lambda m: m.update(context=None)),
        ("package", lambda m: m["context"].update(package="wrong-package")),
        ("crate", lambda m: m["context"].update(crate="wrong-crate")),
        ("root", lambda m: m["context"].update(root_file="/forged/lib.rs")),
        ("kind", lambda m: m["context"].update(kind="first-crate-fallback")),
    ):
        test.mutation(lambda c,m,change=change:change(m), "selected-semantic-context-" + name)
