use super::{HashMap, HirModule, Renderer, RustFile, StdlibCode};
use crate::lib_project_codegen::register_imported_union_types;

pub(crate) fn render_project_structural_record_prelude(
    modules: &[(&str, &HirModule)],
    project_code: &StdlibCode,
) -> String {
    let mut records = HashMap::new();
    for (_, module) in modules {
        let mut emitter = super::RustEmitter::new();
        emitter.collect_union_types(module);
        register_imported_union_types(&mut emitter, module, project_code);
        records.extend(emitter.structural_record_types);
    }
    if records.is_empty() {
        return String::new();
    }
    let mut emitter = super::RustEmitter::new();
    emitter.structural_record_types = records;
    emitter.generate_structural_record_definitions();
    Renderer::new().render_file(&RustFile {
        items: emitter.body_items,
    })
}

#[cfg(test)]
#[path = "../tests/unit/project_structural_record_codegen/tests.rs"]
mod tests;
