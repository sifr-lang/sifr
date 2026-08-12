use crate::ir_imports::collect_import_needs_from_items;
use crate::lib_project_codegen::ProjectUnionUsage;
use crate::{publicize_generated_module_source, HashMap, Renderer, RustFile, RustItem};
use std::fmt::Write;

const PROJECT_UNION_MODULE: &str = "__sifr_project_unions";

pub(crate) fn render_project_union_prelude(
    usage: &ProjectUnionUsage,
    nominal_type_paths: &HashMap<String, String>,
) -> String {
    if usage.unions.is_empty() {
        return String::new();
    }
    let mut emitter = crate::RustEmitter::new();
    emitter.union_enums.clone_from(&usage.unions);
    emitter
        .ordinary_union_enums
        .clone_from(&usage.ordinary_unions);
    emitter
        .try_error_carrier_enums
        .clone_from(&usage.try_error_unions);
    emitter
        .project_nominal_type_paths
        .clone_from(nominal_type_paths);
    emitter.generate_enum_definitions();

    let import_needs = collect_import_needs_from_items(&emitter.enum_items);
    let mut imports = Vec::new();
    let mut add_import = |needed: bool, path: &[&str]| {
        if needed {
            imports.push(RustItem::Use(
                path.iter().map(|part| (*part).to_string()).collect(),
            ));
        }
    };
    add_import(
        import_needs.collections.needs_hashmap,
        &["std", "collections", "HashMap"],
    );
    add_import(
        import_needs.collections.needs_hashset,
        &["std", "collections", "HashSet"],
    );
    add_import(
        import_needs.collections.needs_vecdeque,
        &["std", "collections", "VecDeque"],
    );
    add_import(
        import_needs.runtime.numeric.needs_bigint,
        &["num_bigint", "BigInt"],
    );
    add_import(
        import_needs.runtime.numeric.needs_decimal,
        &["rust_decimal", "Decimal"],
    );
    add_import(
        import_needs.runtime.numeric.needs_bigdecimal,
        &["bigdecimal", "BigDecimal"],
    );
    add_import(
        import_needs.runtime.needs_sifr_int,
        &["", "sifr_runtime", "SifrInt"],
    );
    add_import(import_needs.runtime.needs_mutex, &["std", "sync", "Mutex"]);

    let import_source = Renderer::new().render_file(&RustFile { items: imports });
    let enum_source = publicize_generated_module_source(&Renderer::new().render_file(&RustFile {
        items: emitter.enum_items,
    }));
    let mut prelude = format!("mod {PROJECT_UNION_MODULE} {{\n");
    for line in import_source.lines().chain(enum_source.lines()) {
        prelude.push_str("    ");
        prelude.push_str(line);
        prelude.push('\n');
    }
    prelude.push_str("}\n");
    let mut names = usage.unions.keys().collect::<Vec<_>>();
    names.sort();
    for name in names {
        let _ = writeln!(prelude, "pub use {PROJECT_UNION_MODULE}::{name};");
    }
    prelude
}
