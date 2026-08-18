use super::support::parse_suite;
use crate::{collect_project_hir_modules, compile_stdlib};
use sifr_lowering::{HirExpr, HirStmt};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[test]
fn compiled_stdlib_exports_match_public_reference() {
    let compiled = compile_stdlib().expect("stdlib should compile");
    let mut modules =
        BTreeMap::<String, (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>)>::new();
    for (module, functions) in &compiled.defs.functions {
        if module.starts_with("sifr.") {
            modules
                .entry(module.clone())
                .or_default()
                .0
                .extend(functions.keys().cloned());
        }
    }
    for (module, classes) in &compiled.defs.classes {
        if module.starts_with("sifr.") {
            modules
                .entry(module.clone())
                .or_default()
                .1
                .extend(classes.keys().cloned());
        }
    }
    for (module, constants) in &compiled.defs.constants {
        if module.starts_with("sifr.") {
            modules
                .entry(module.clone())
                .or_default()
                .2
                .extend(constants.keys().cloned());
        }
    }

    let mut rendered = String::from(
        "---\ntitle: \"Compiled Standard Library Exports\"\nsidebarTitle: \"Public API\"\ndescription: \"The exact public symbols compiled for every sifr.* standard-library module.\"\n---\n\n# Compiled Standard Library Exports\n\nThis reference is generated from compiler metadata. Imported `_sifr.*` implementation names are excluded unless the compiler's explicit private re-export policy approves the canonical symbol. The three `sifr.heapq` max-heap helpers are retained separately for CPython-compatible max-heap semantics.\n",
    );
    for (module, (functions, classes, constants)) in modules {
        rendered.push_str(&format!("\n## `{module}`\n"));
        if !functions.is_empty() {
            rendered.push_str(&format!(
                "\nFunctions: {}.\n",
                functions
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !classes.is_empty() {
            rendered.push_str(&format!(
                "\nClasses: {}.\n",
                classes
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !constants.is_empty() {
            rendered.push_str(&format!(
                "\nConstants: {}.\n",
                constants
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    let reference_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/stdlib/public-api.mdx");
    if std::env::var_os("SIFR_UPDATE_STDLIB_PUBLIC_API").is_some() {
        std::fs::write(&reference_path, &rendered).expect("public API reference should update");
    }
    assert_eq!(
        rendered,
        std::fs::read_to_string(reference_path).expect("public API reference should be readable")
    );
}

#[test]
fn compiled_stdlib_same_operation_groups_have_one_public_name() {
    let compiled = compile_stdlib().expect("stdlib should compile");
    let groups: &[(&str, &[&str])] = &[
        ("sifr.math", &["fabs", "abs_val"]),
        ("sifr.math", &["pow", "pow_val"]),
        ("sifr.statistics", &["mean", "fmean"]),
        ("sifr.bisect", &["bisect_right", "bisect"]),
        ("sifr.bisect", &["insort_right", "insort"]),
        ("sifr.fnmatch", &["fnmatch", "fnmatchcase"]),
        ("sifr.json", &["loads", "json_loads"]),
        ("sifr.tomllib", &["loads", "toml_loads"]),
        ("sifr.url", &["parse", "parse_url", "url_parse"]),
        ("sifr.url", &["build", "build_url", "url_build"]),
        (
            "sifr.base64",
            &["b64encode", "standard_b64encode", "encodebytes"],
        ),
        (
            "sifr.base64",
            &["b64decode", "standard_b64decode", "decodebytes"],
        ),
    ];
    for (module, names) in groups {
        let exports = compiled
            .defs
            .functions
            .get(*module)
            .unwrap_or_else(|| panic!("{module} functions should be exported"));
        let visible = names
            .iter()
            .filter(|name| exports.contains_key(**name))
            .count();
        assert_eq!(visible, 1, "{module} same-operation group {names:?}");
    }
}

#[test]
fn stdlib_heapq_exports_allowlisted_private_max_heap_helpers() {
    let compiled = compile_stdlib().expect("stdlib should compile");
    let heapq_functions = compiled
        .defs
        .functions
        .get("sifr.heapq")
        .expect("sifr.heapq exports should exist");

    for name in ["_heapify_max", "_heappop_max", "_heapreplace_max"] {
        assert!(
            heapq_functions.contains_key(name),
            "expected approved sifr.heapq max-heap export '{name}' to be visible"
        );
    }
}

#[test]
fn stdlib_integer_constants_fold_in_project_fixed_width_initializers() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from sifr.logging import DEBUG

def main() -> uint8:
    value: uint8 = DEBUG + 1
    return value
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("project lowering should fit imported stdlib integer constants");
    let main_module = result
        .hir_modules
        .get("main")
        .expect("main module should lower");
    let main_fn = main_module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function should lower");
    let HirStmt::Let { ty, value, .. } = &main_fn.body[0] else {
        panic!("expected first statement to be fitted let");
    };
    assert_eq!(ty.display_name(), "uint8");
    assert!(matches!(value, HirExpr::IntLiteral(11)));
}
