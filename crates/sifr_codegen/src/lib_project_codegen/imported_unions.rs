use super::{HirModule, StdlibCode};

pub(crate) fn register_imported_union_types(
    emitter: &mut crate::RustEmitter,
    module: &HirModule,
    project_code: &StdlibCode,
) {
    for import in &module.imports {
        if import.module.starts_with("sifr.") || import.module.starts_with("_sifr.") {
            continue;
        }
        if let Some(signatures) = project_code.func_signatures.get(&import.module) {
            for name in &import.names {
                if let Some((params, return_type)) = signatures.get(name) {
                    for (param_type, _) in params {
                        emitter.register_union_type(param_type);
                    }
                    emitter.register_union_type(return_type);
                }
            }
        }
        if let Some(classes) = project_code.module_class_fields.get(&import.module) {
            for name in &import.names {
                if let Some(fields) = classes.get(name) {
                    for (_, field_type) in fields {
                        emitter.register_union_type(field_type);
                    }
                }
            }
        }
    }
}
