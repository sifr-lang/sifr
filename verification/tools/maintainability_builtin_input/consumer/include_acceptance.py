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
        test.require(mapping["expanded_source_tokens"] == invocation["declaration_facts"]["source_tokens"], "complete original expanded source token stream")
        indices={c["index"] for c in mapping["ordinary_comments"]}
        test.require(len(indices)==2 and [t for index,t in enumerate(mapping["physical_source_tokens"]) if index not in indices]==mapping["expanded_source_tokens"], "complete physical token stream retains leading and interior ordinary comments")
        test.require(any("'a ::" in c["text"] for c in mapping["ordinary_comments"]), "comment contents retained without literal/token rewriting")
        test.require(mapping["file"].endswith("/included_shapes.rs") and invocation["invocation_site"]["file"] == mapping["file"], "actual innermost included physical source")
        owners = [d for d in test.extended["declarations"] if d["receiver_identity"] == invocation["receiver"] and d["expansion_chain"][0]["macro_identity"] == invocation["macro"]]
        test.require(bool(owners) and all(sum(c["macro_identity"] == "core::macros::builtin::include" for c in d["expansion_chain"]) == 2 for d in owners), "original compiler nested include chain retained")
        test.require(all(d["expansion_chain"][0]["call_site"]["quality"] == "exact-source" for d in owners), "exact compiler derive source authority")
    context = test.extended_common["context"]
    test.require(context["kind"] == "ra-semantic-selected-cargo-target-root" and context["crate"] == "builtin_fixture", "selected semantic crate bound to actual Cargo target root")
    locals=[i for i in test.extended_common["invocations"] if i["source_name"]=="LocalShape"]
    test.require(len(locals)==3 and {i["receiver"]["adt"].rsplit("::",2)[-2] for i in locals}=={"local_left","local_right","construct"}, "three real same-spelled local ADTs retain resolved function ownership")
    test.require(len({b.encoded(i["receiver"]) for i in locals})==3, "distinct function-local semantic receiver identities")
    test.require(all(i["declaration_facts"]["identity"]==i["receiver"]["adt"] for i in locals), "actual local declaration source and canonical receiver identity")
    test.require(sum(i["include_source_mapping"] is not None for i in locals)==1, "real method-local declaration through two include layers")
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
    def swap_local(c,common):
        originals=[i for i in common["invocations"] if i["source_name"]=="LocalShape"]
        originals[0]["receiver"]=originals[1]["receiver"]
    test.mutation(swap_local,"swapped-local-containing-function-receiver")
    def swap_identity(c,common):
        originals=[i for i in common["invocations"] if i["source_name"]=="LocalShape"]
        originals[0]["declaration_facts"]["identity"]=originals[1]["declaration_facts"]["identity"]
    test.mutation(swap_identity,"swapped-local-containing-function-source-owner")
    def selected_include(common):
        return next(i for i in common["invocations"] if i["source_name"]=="Included")["include_source_mapping"]
    for label,change in (
        ("omission",lambda m:m.pop("ordinary_comments")),
        ("remove",lambda m:m["ordinary_comments"].pop()),
        ("text",lambda m:m["ordinary_comments"][0].update(text="// forged")),
        ("index",lambda m:m["ordinary_comments"][0].update(index=999)),
        ("range",lambda m:m["ordinary_comments"][0].update(range="0..1")),
        ("order",lambda m:m["ordinary_comments"].reverse()),
        ("physical",lambda m:m["physical_source_tokens"].pop()),
    ):
        test.mutation(lambda c,m,change=change:change(selected_include(m)),"included-comment-"+label)
