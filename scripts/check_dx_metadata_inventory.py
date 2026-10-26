#!/usr/bin/env python3
"""Check the pinned DX.5 consumer inventory and explicit payload-field coverage."""
from __future__ import annotations
import argparse
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "internal_docs/compiler_dx_metadata_consumers.md"
BEGIN = "<!-- dx5-sites:start -->"
END = "<!-- dx5-sites:end -->"
OWNERS = ("sifr_lowering", "sifr_frontend", "sifr_compiler_services", "sifr_semantic_services", "sifr_driver", "sifr_codegen", "sifr_analysis", "sifr_lsp", "sifr")
# Shared test consumers may live outside production crate roots.
SHARED_TEST_SOURCES = ("verification/tools/corpus_oracle/src/lib.rs",)
TERMS = re.compile(r"\b(?:ExternalDefs|StdlibCode|StdlibEmissionCode|StdlibEmissionView|external_defs|externals|stdlib_defs|stdlib_code|hir_modules|module_rust_code|module_constants|func_signatures|transitive_deps|generator_functions|generic_classes|generic_class_params|generic_class_templates|module_class_fields|module_class_templates)\b|\b\w*stdlib\w*\b")


def scrub(text: str) -> str:
    # Keep byte/line positions for exact source anchors; exclude string examples/comments.
    return re.sub(r'//[^\n]*|"(?:\\.|[^"\\])*"', lambda m: re.sub(r"[^\n]", " ", m[0]), text)


def functions(text: str):
    clean = scrub(text)
    for match in re.finditer(r"\bfn\s+(\w+)", clean):
        brace = clean.find("{", match.end())
        semi = clean.find(";", match.end())
        if brace < 0 or 0 <= semi < brace:
            continue
        depth = 1
        end = brace + 1
        while end < len(clean) and depth:
            depth += (clean[end] == "{") - (clean[end] == "}")
            end += 1
        yield match[1], match.start(), end, clean[match.start():end]


def consumer_sources():
    for owner in OWNERS:
        for path in sorted((ROOT / "crates" / owner / "src").rglob("*.rs")):
            yield owner, path, False
    for relative in SHARED_TEST_SOURCES:
        path = ROOT / relative
        if not path.is_file():
            raise ValueError(f"missing maintained shared metadata consumer: {relative}")
        yield "shared-test", path, True


def sites() -> str:
    rows = []
    for owner, path, shared_test in consumer_sources():
        text = path.read_text()
        for symbol, start, end, body in functions(text):
            indirect = owner == "sifr_lsp" and re.search(r"AnalysisHost|analysis|workspace", body)
            matches = list(TERMS.finditer(body))
            if not matches and not indirect:
                continue
            line = text.count("\n", 0, start) + 1
            operations = ["read"]
            if re.search(r"\.(?:iter|values|keys|into_values)\(", body):
                operations.append("enumerate")
            if re.search(r"\bmut\s+(?:external_defs|externals|stdlib_defs)\b|&mut\s+(?:\w+::)*ExternalDefs|(?:external_defs|externals|stdlib_defs)[\s\S]{0,160}\.(?:insert|remove|extend|entry|to_mut)\(", body):
                operations.append("mutate overlay/producer")
            if re.search(r"(?:external_defs|externals|stdlib_defs|stdlib_code|compiled\.defs)[\s\S]{0,160}\.clone\(", body):
                operations.append("clone projection")
            if re.search(r"Arc|OnceLock|Cow|WorkspaceSession|AnalysisHost|&(?:'\w+\s+)?ExternalDefs", body):
                operations.append("borrow/retain")
            test = shared_test or "test" in path.name or "tests" in path.parts or "#[test]" in text[max(0,start-100):start]
            target = "test setup/parity" if test else ("snapshot query (indirect)" if indirect and not matches else "layered view + demanded record handles")
            if ("metadata_producer" in path.parts or symbol == "compile_stdlib_sources_with_sysroot") and not test:
                target = "canonical source-only producer (retain)"
            anchors = sorted({text.count("\n", 0, start+m.start())+1 for m in matches})
            anchor_text = ", ".join(map(str, anchors)) if anchors else "indirect"
            rows.append(f"| `{path.relative_to(ROOT)}:{line}` `{symbol}` | {', '.join(operations)} | {target} | {anchor_text} |")
    return "\n".join([BEGIN, "", "| Exact source symbol | Current access (conservative union within symbol) | DX.7 migration | Matching source lines |", "| --- | --- | --- | --- |", *rows, "", END])


def struct_body(text: str, name: str) -> str:
    text = scrub(text)
    match = re.search(r"pub struct " + name + r"\s*\{", text)
    if not match:
        raise ValueError(f"missing schema struct {name}")
    start = match.end()
    end = text.find("\n}", start)
    if end < 0:
        raise ValueError(f"unterminated schema struct {name}")
    return text[start:end]


def fields(text: str, name: str) -> set[str]:
    return set(re.findall(r"(?:pub\s+)?(\w+)\s*:(?!:)", struct_body(text, name))) - {"std", "collections", "sifr_ir", "num_bigint"}


# These are live compiler authorities, not semantic payload. Pin their shapes so
# a future field/type change cannot silently inherit a transport exemption.
LIVE_EXTERNAL_AUTHORITIES = {
    "provider": "Option<std::sync::Arc<dyn ExternalProvider>>",
    "typed_method_processor": "Option<std::sync::Arc<dyn TypedMethodProcessor>>",
}


def check_external_fields(source: str, wire: str) -> None:
    compact = re.sub(r"\s+", "", struct_body(source, "ExternalDefs"))
    for name, shape in LIVE_EXTERNAL_AUTHORITIES.items():
        pattern = r"(?:^|,)pub" + re.escape(name + ":" + re.sub(r"\s+", "", shape)) + r"(?:,|$)"
        if re.search(pattern, compact) is None:
            raise ValueError(f"ExternalDefs live authority shape changed: {name}")
    if fields(source, "ExternalDefs") - LIVE_EXTERNAL_AUTHORITIES.keys() != fields(wire, "SemanticExports"):
        raise ValueError("ExternalDefs field inventory changed; explicitly classify and preserve the payload")


def check_fields() -> None:
    source = (ROOT / "crates/sifr_lowering/src/lower/external_defs.rs").read_text()
    wire = (ROOT / "crates/sifr_metadata/src/metadata/semantic_records.rs").read_text()
    check_external_fields(source, wire)
    groups = ("hir_nodes", "specialization_metadata", "rust_interop", "python_interop", "template_strings", "sql_queries", "sql_migrations")
    for group in groups:
        source = (ROOT / f"crates/sifr_ir/src/{group}.rs").read_text()
        if group == "hir_nodes":
            source += "\n" + (ROOT / "crates/sifr_ir/src/method_call_metadata.rs").read_text()
        wire = (ROOT / f"crates/sifr_metadata/src/metadata/{group}.rs").read_text()
        for name in re.findall(r"pub struct (\w+)\s*\{", source):
            if name == "SourceOriginId":
                continue  # Explicitly remapped to SourceLocation + local ordinal.
            if fields(source, name) != fields(wire, name):
                raise ValueError(f"payload fields differ for {group}::{name}")
        original = set(re.findall(r"pub (?:struct|enum) (\w+)", source))
        transported = set(re.findall(r"pub (?:struct|enum) (\w+)", wire))
        if original != transported:
            raise ValueError(f"record family inventory differs for {group}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group()
    action.add_argument("--refresh-sites", action="store_true")
    action.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        import unittest
        from test_dx_metadata_inventory import MetadataInventoryTests
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(MetadataInventoryTests)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1
    check_fields()
    current = INVENTORY.read_text()
    generated = sites()
    start, end = current.index(BEGIN), current.index(END) + len(END)
    if args.refresh_sites:
        INVENTORY.write_text(current[:start] + generated + current[end:])
    elif current[start:end] != generated:
        raise ValueError("DX.5 consumer sites changed; review migration classification then refresh sites")
    print("DX.5 payload fields and consumer migration inventory: PASS")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except ValueError as error:
        print(error, file=sys.stderr)
        sys.exit(1)
