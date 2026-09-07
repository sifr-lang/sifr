use super::{
    DocumentVersion, FileId, FrontendDiagnosticStyle, FrontendSourceContext, ModuleId, ModuleState,
    SourceHash, SourcePath, SourceText, SymbolKind, SymbolView,
};
use crate::callable_exports::{RustCallbackExports, exported_function_type};
use crate::class_method_exports::{ClassMethodExports, structural_method_map};
pub(crate) use crate::export_type_localization::should_export_callable;
use crate::export_type_localization::{
    copy_class_generic_metadata, copy_function_generic_metadata, declared_generic_metadata,
    exported_class_fields, exported_parent_chain, imported_class_ancestry, reexport_class_aliases,
};
use crate::module_export_storage::replace_module_entry;
use crate::module_signatures::ModuleSignature;
use crate::{diagnostic_with_code, diagnostic_with_source_range_args_help};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_lowering::{
    ExternalDefs, HirClassKind, HirDiagnostic, HirModule, LoweringResult,
    canonicalize_user_export_type, localize_user_import_function_type, localize_user_import_type,
};
use sifr_python_ast::Stmt;
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::{BTreeMap, HashMap};

mod const_reexports;
mod rust_class_exports;
pub(super) fn module_state(
    id: ModuleId,
    file: FileId,
    module_name: impl Into<String>,
    path: SourcePath,
    source: SourceText,
    document_version: Option<DocumentVersion>,
) -> ModuleState {
    let source_hash = source_hash(source.as_str());
    ModuleState {
        id,
        file,
        module_name: module_name.into(),
        path,
        source,
        source_hash,
        document_version,
        signature: ModuleSignature::default(),
        source_file_view: None,
        parsed: None,
        lowered: None,
        diagnostics: None,
        analysis: None,
    }
}

pub(super) fn source_hash(source: &str) -> SourceHash {
    SourceHash::from_source_text(source)
}

pub(super) fn local_import_dependencies(
    stmts: &[Stmt],
    module_names: &BTreeMap<String, ModuleId>,
) -> Vec<ModuleId> {
    let mut deps = Vec::new();
    for stmt in stmts {
        let Stmt::ImportFrom(import_from) = stmt else {
            continue;
        };
        if import_from.level > 1 {
            continue;
        }
        let Some(module) = &import_from.module else {
            continue;
        };
        let module_name = module.to_string();
        if module_name == "typing"
            || module_name == "enum"
            || module_name.starts_with("sifr.")
            || module_name.starts_with("_sifr.")
        {
            continue;
        }
        if let Some(module_id) = module_names.get(&module_name) {
            deps.push(*module_id);
        }
    }
    deps.sort();
    deps.dedup();
    deps
}

pub(super) fn symbols_from_hir(module: &HirModule) -> Vec<SymbolView> {
    let mut symbols = Vec::new();
    symbols.extend(module.functions.iter().map(|function| SymbolView {
        name: function.name.clone(),
        kind: SymbolKind::Function,
    }));
    symbols.extend(module.classes.iter().map(|class| SymbolView {
        name: class.name.clone(),
        kind: SymbolKind::Class,
    }));
    symbols.extend(module.constants.iter().map(|(name, _, _)| SymbolView {
        name: name.clone(),
        kind: SymbolKind::Constant,
    }));
    symbols.extend(module.imports.iter().map(|import| SymbolView {
        name: import.module.clone(),
        kind: SymbolKind::Import,
    }));
    symbols.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
    });
    symbols
}

pub(super) fn empty_hir_module() -> HirModule {
    HirModule {
        functions: Vec::new(),
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}

pub(super) fn hir_diagnostic_to_rendered(
    module_name: &str,
    diagnostic_style: FrontendDiagnosticStyle,
    source_context: Option<FrontendSourceContext<'_>>,
    error: HirDiagnostic,
) -> RenderedDiagnostic {
    let code = error
        .code
        .unwrap_or(DiagnosticCode::INTERNAL_COMPILER_PANIC);
    let uncoded = error.code.is_none();
    let primary_range = error.primary_range;
    let structured_args = error.args;
    let help = error.help;
    let message = match diagnostic_style {
        FrontendDiagnosticStyle::Bare => error.message,
        FrontendDiagnosticStyle::ModulePrefixed => {
            format!("[{}] {}", module_name, error.message)
        }
    };
    let message = if uncoded {
        format!(
            "internal compiler error: HIR lowering emitted a diagnostic without canonical code: {message}"
        )
    } else {
        message
    };
    if let (Some(context), Some(range)) = (source_context, primary_range) {
        return diagnostic_with_source_range_args_help(
            code,
            context,
            range,
            "{message}",
            BTreeMap::from([(
                "message".to_string(),
                DiagnosticArg::String(message.clone()),
            )]),
            structured_args,
            help,
        );
    }
    let mut rendered = diagnostic_with_code(message, code);
    rendered.args.extend(structured_args);
    rendered.help = help;
    rendered
}

pub fn collect_module_exports(
    module_name: &str,
    lowering_result: &LoweringResult,
    external_defs: &mut ExternalDefs,
) {
    let module = &lowering_result.module;
    let mut fn_exports = HashMap::new();
    let mut const_fn_exports = HashMap::new();
    let mut class_exports = HashMap::new();
    let mut generic_type_alias_exports = HashMap::new();
    let mut class_method_exports = ClassMethodExports::default();
    let mut class_type_param_exports = HashMap::new();
    let mut rust_opaque_exports = std::collections::HashSet::new();
    let mut rust_structural_exports = std::collections::HashSet::new();
    let mut class_field_default_exports = HashMap::new();
    let mut const_exports = HashMap::new();
    let mut const_integer_value_exports = HashMap::new();
    let mut default_exports = HashMap::new();
    let mut vararg_exports = HashMap::new();
    let mut python_shape_exports = HashMap::new();
    let mut workload_exports = HashMap::new();
    let mut error_exports = std::collections::HashSet::new();
    let mut rust_callback_exports = RustCallbackExports::default();
    let (mut generic_exports, mut type_param_bound_exports, local_classes) =
        declared_generic_metadata(module_name, module);
    let structural_method_exports = structural_method_map(
        module_name,
        module,
        &local_classes,
        lowering_result,
        external_defs,
    );
    let imported_ancestry = imported_class_ancestry(module, external_defs);

    for (name, (type_params, alias)) in &lowering_result.generic_type_aliases {
        if !name.starts_with('_') {
            generic_type_alias_exports.insert(
                name.clone(),
                (
                    type_params.clone(),
                    canonicalize_user_export_type(alias, &local_classes),
                ),
            );
        }
    }

    for func in &module.functions {
        if should_export_callable(module_name, &func.name) {
            const_reexports::record_local_const_function(func, &mut const_fn_exports);
            fn_exports.insert(
                func.name.clone(),
                exported_function_type(func, &local_classes),
            );
            if let Some(vararg_index) = lowering_result.function_varargs.get(&func.name) {
                vararg_exports.insert(func.name.clone(), *vararg_index);
            }
            if let Some(shapes) = lowering_result.function_python_call_shapes.get(&func.name) {
                python_shape_exports.insert(func.name.clone(), shapes.clone());
            }
            if let Some(label) = lowering_result.function_workloads.get(&func.name) {
                workload_exports.insert(func.name.clone(), label.clone());
            }
            rust_callback_exports.record_function(func);
        }
    }

    for (callable_name, defaults) in &lowering_result.function_defaults {
        if should_export_callable(module_name, callable_name) {
            default_exports.insert(callable_name.clone(), defaults.clone());
        }
    }

    for class in &module.classes {
        if let Some(defaults) = lowering_result.class_field_defaults.get(&class.name) {
            class_field_default_exports.insert(class.name.clone(), defaults.clone());
        }
        if !class.type_params.is_empty() {
            class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
        }
        if !class.name.starts_with('_') {
            if class.is_error_type {
                error_exports.insert(class.name.clone());
            }
            rust_class_exports::record_local(
                class,
                &mut rust_opaque_exports,
                &mut rust_structural_exports,
            );
            class_method_exports.record_local(class);
            rust_callback_exports.record_class(class);
            let mut methods: Vec<(String, FunctionType)> = class
                .methods
                .iter()
                .map(|m| {
                    let params: Vec<(String, Type, ParamConvention)> = m
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                        .collect();
                    (
                        m.name.clone(),
                        FunctionType {
                            receiver: m.receiver,
                            params,
                            return_type: Box::new(m.return_type.clone()),
                        },
                    )
                })
                .collect();
            for (dunder_name, op_func) in &class.operator_impls {
                let params: Vec<(String, Type, ParamConvention)> = op_func
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                    .collect();
                methods.push((
                    dunder_name.clone(),
                    FunctionType {
                        receiver: op_func.receiver,
                        params,
                        return_type: Box::new(op_func.return_type.clone()),
                    },
                ));
            }
            let exported_type = if let Some(inner) = &class.newtype_inner {
                Type::Newtype {
                    identity: None,
                    name: class.name.clone(),
                    inner: Box::new(inner.clone()),
                }
            } else {
                match &class.kind {
                    HirClassKind::Protocol => Type::Protocol {
                        identity: None,
                        name: class.name.clone(),
                        methods,
                    },
                    HirClassKind::Enum => Type::Enum {
                        identity: None,
                        name: class.name.clone(),
                        variants: class.enum_variants.clone(),
                    },
                    HirClassKind::Regular | HirClassKind::PythonOpaque(_) => Type::Class {
                        identity: None,
                        type_args: class
                            .type_params
                            .iter()
                            .cloned()
                            .map(Type::TypeVar)
                            .collect(),
                        name: class.name.clone(),
                        fields: exported_class_fields(class),
                        methods,
                        parent_class: if class.is_error_type {
                            class.semantic_parent_chain()
                        } else {
                            exported_parent_chain(
                                class.parent_class.as_deref(),
                                module,
                                &imported_ancestry,
                            )
                        },
                    },
                }
            };
            let class_ty = canonicalize_user_export_type(&exported_type, &local_classes);
            class_exports.insert(class.name.clone(), class_ty);
        }
    }
    crate::descriptor_exports::add_aliases(lowering_result, &local_classes, &mut class_exports);
    for (name, ty, _) in &module.constants {
        if !name.starts_with('_') {
            const_exports.insert(
                name.clone(),
                canonicalize_user_export_type(ty, &local_classes),
            );
            if let Some(value) = lowering_result.constant_integer_values.get(name) {
                const_integer_value_exports.insert(name.clone(), value.clone());
            }
        }
    }

    let reexport_aliases = reexport_class_aliases(module, external_defs);

    for import in &module.imports {
        let class_aliases = reexport_aliases
            .get(&import.module)
            .cloned()
            .unwrap_or_default();
        for name in &import.names {
            let local_name = import
                .aliases
                .iter()
                .find(|(original, _)| original == name)
                .map_or_else(|| name.clone(), |(_, alias)| alias.clone());
            if local_name.starts_with('_') {
                continue;
            }
            rust_callback_exports.copy_imported(external_defs, &import.module, name, &local_name);
            if let Some(module_fns) = external_defs.functions.get(&import.module) {
                if let Some(function_type) = module_fns.get(name) {
                    fn_exports.insert(
                        local_name.clone(),
                        localize_user_import_function_type(
                            function_type,
                            &import.module,
                            &class_aliases,
                        ),
                    );
                    const_reexports::copy_const_function_and_defaults(
                        external_defs,
                        &import.module,
                        name,
                        &local_name,
                        &mut const_fn_exports,
                        &mut default_exports,
                    );
                    if let Some(vararg_index) = external_defs
                        .function_varargs
                        .get(&import.module)
                        .and_then(|module_varargs| module_varargs.get(name))
                    {
                        vararg_exports.insert(local_name.clone(), *vararg_index);
                    }
                    if let Some(shapes) = external_defs
                        .function_python_call_shapes
                        .get(&import.module)
                        .and_then(|module_shapes| module_shapes.get(name))
                    {
                        python_shape_exports.insert(local_name.clone(), shapes.clone());
                    }
                    if let Some(label) = external_defs
                        .function_workloads
                        .get(&import.module)
                        .and_then(|module_workloads| module_workloads.get(name))
                    {
                        workload_exports.insert(local_name.clone(), label.clone());
                    }
                    copy_function_generic_metadata(
                        external_defs,
                        &import.module,
                        name,
                        &local_name,
                        &mut generic_exports,
                        &mut type_param_bound_exports,
                    );
                    continue;
                }
            }
            if let Some(module_classes) = external_defs.classes.get(&import.module) {
                if let Some(class_type) = module_classes.get(name) {
                    if external_defs.is_error_type(&import.module, name) {
                        error_exports.insert(local_name.clone());
                    }
                    rust_class_exports::record_imported(
                        external_defs,
                        &import.module,
                        name,
                        &local_name,
                        &mut rust_opaque_exports,
                        &mut rust_structural_exports,
                    );
                    class_exports.insert(
                        local_name.clone(),
                        localize_user_import_type(class_type, &import.module, &class_aliases),
                    );
                    class_method_exports.record_imported(
                        external_defs,
                        &import.module,
                        name,
                        &local_name,
                    );
                    copy_class_generic_metadata(
                        external_defs,
                        &import.module,
                        name,
                        &local_name,
                        &mut class_type_param_exports,
                        &mut generic_exports,
                        &mut type_param_bound_exports,
                    );
                    continue;
                }
            }
            if let Some(module_aliases) = external_defs.generic_type_aliases.get(&import.module) {
                if let Some((type_params, alias)) = module_aliases.get(name) {
                    generic_type_alias_exports.insert(
                        local_name.clone(),
                        (
                            type_params.clone(),
                            localize_user_import_type(alias, &import.module, &class_aliases),
                        ),
                    );
                    continue;
                }
            }
            if let Some(module_consts) = external_defs.constants.get(&import.module) {
                if let Some(const_type) = module_consts.get(name) {
                    const_exports.insert(
                        local_name.clone(),
                        localize_user_import_type(const_type, &import.module, &class_aliases),
                    );
                    if let Some(value) = external_defs
                        .constant_integer_values
                        .get(&import.module)
                        .and_then(|module_values| module_values.get(name))
                    {
                        const_integer_value_exports.insert(local_name, value.clone());
                    }
                }
            }
        }
    }

    external_defs
        .functions
        .insert(module_name.to_string(), fn_exports);
    if !const_fn_exports.is_empty() {
        external_defs
            .const_functions
            .insert(module_name.to_string(), const_fn_exports);
    }
    external_defs
        .classes
        .insert(module_name.to_string(), class_exports);
    external_defs
        .generic_type_aliases
        .insert(module_name.to_string(), generic_type_alias_exports);
    external_defs.replace_structural_methods(module_name, structural_method_exports);
    if error_exports.is_empty() {
        external_defs.error_types.remove(module_name);
    } else {
        external_defs
            .error_types
            .insert(module_name.to_string(), error_exports);
    }
    rust_class_exports::replace_module(
        external_defs,
        module_name,
        rust_opaque_exports,
        rust_structural_exports,
    );
    replace_module_entry(
        &mut external_defs.class_field_defaults,
        module_name,
        class_field_default_exports,
        HashMap::is_empty,
    );
    replace_module_entry(
        &mut external_defs.declaration_metadata,
        module_name,
        lowering_result.declaration_metadata.clone(),
        Vec::is_empty,
    );
    crate::descriptor_exports::store(module_name, module, lowering_result, external_defs);
    if !lowering_result.specialization_requests.is_empty() {
        external_defs.specialization_requests.insert(
            module_name.to_string(),
            lowering_result.specialization_requests.clone(),
        );
    }
    if !lowering_result.specialization_outputs.is_empty() {
        external_defs.specialization_outputs.insert(
            module_name.to_string(),
            lowering_result.specialization_outputs.clone(),
        );
    }
    if !lowering_result.json_integer_boundary_requests.is_empty() {
        external_defs.json_integer_boundary_requests.insert(
            module_name.to_string(),
            lowering_result.json_integer_boundary_requests.clone(),
        );
    }
    class_method_exports.store(external_defs, module_name);
    replace_module_entry(
        &mut external_defs.class_type_params,
        module_name,
        class_type_param_exports,
        HashMap::is_empty,
    );
    if !generic_exports.is_empty() {
        external_defs
            .generic_functions
            .insert(module_name.to_string(), generic_exports);
    }
    if !type_param_bound_exports.is_empty() {
        external_defs
            .type_param_bounds
            .insert(module_name.to_string(), type_param_bound_exports);
    }
    if !default_exports.is_empty() {
        external_defs
            .function_defaults
            .insert(module_name.to_string(), default_exports);
    }
    if !vararg_exports.is_empty() {
        external_defs
            .function_varargs
            .insert(module_name.to_string(), vararg_exports);
    }
    if !python_shape_exports.is_empty() {
        external_defs
            .function_python_call_shapes
            .insert(module_name.to_string(), python_shape_exports);
    }
    if !workload_exports.is_empty() {
        external_defs
            .function_workloads
            .insert(module_name.to_string(), workload_exports);
    }
    rust_callback_exports.store(external_defs, module_name);
    external_defs
        .constants
        .insert(module_name.to_string(), const_exports);
    if !const_integer_value_exports.is_empty() {
        external_defs
            .constant_integer_values
            .insert(module_name.to_string(), const_integer_value_exports);
    }
}

#[cfg(test)]
#[path = "query_diagnostics_tests.rs"]
mod tests;
