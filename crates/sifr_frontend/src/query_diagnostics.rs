use super::{
    DocumentVersion, FileId, FrontendDiagnosticStyle, FrontendSourceContext, ModuleId, ModuleState,
    SourceHash, SourcePath, SourceText, SymbolKind, SymbolView,
};
use crate::callable_exports::{exported_function_type, RustCallbackExports};
use crate::class_method_exports::{structural_method_map, ClassMethodExports};
pub(crate) use crate::export_type_localization::should_export_callable;
use crate::export_type_localization::{
    copy_class_generic_metadata, copy_function_generic_metadata, declared_generic_metadata,
    exported_parent_chain, imported_class_ancestry, reexport_class_aliases,
};
use crate::module_signatures::ModuleSignature;
use crate::{diagnostic_with_code, diagnostic_with_source_range_args_help};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_lowering::{
    canonicalize_user_export_type, localize_user_import_function_type, localize_user_import_type,
    ExternalDefs, HirClassKind, HirDiagnostic, HirModule, LoweringResult, RustInteropDecoratorKind,
};
use sifr_python_ast::Stmt;
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::{BTreeMap, HashMap};

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
    let mut class_method_exports = ClassMethodExports::default();
    let mut class_type_param_exports = HashMap::new();
    let mut rust_opaque_exports = std::collections::HashSet::new();
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
    let structural_method_exports = structural_method_map(module, &local_classes, lowering_result);
    let imported_ancestry = imported_class_ancestry(module, external_defs);

    for func in &module.functions {
        if should_export_callable(module_name, &func.name) {
            if func
                .decorators
                .iter()
                .any(|decorator| decorator == "const_eval")
            {
                const_fn_exports.insert(func.name.clone(), func.clone());
            }
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
            if class
                .rust_interop
                .iter()
                .any(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)
            {
                rust_opaque_exports.insert(class.name.clone());
            }
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
                        fields: class.fields.clone(),
                        methods,
                        parent_class: exported_parent_chain(
                            class.parent_class.as_deref(),
                            module,
                            &imported_ancestry,
                        ),
                    },
                }
            };
            let class_ty = canonicalize_user_export_type(&exported_type, &local_classes);
            class_exports.insert(class.name.clone(), class_ty);
        }
    }
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
                    if let Some(defaults) = external_defs
                        .function_defaults
                        .get(&import.module)
                        .and_then(|module_defaults| module_defaults.get(name))
                    {
                        default_exports.insert(local_name.clone(), defaults.clone());
                    }
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
                    if external_defs
                        .rust_opaque_classes
                        .get(&import.module)
                        .is_some_and(|classes| classes.contains(name))
                    {
                        rust_opaque_exports.insert(local_name.clone());
                    }
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
    external_defs.replace_structural_methods(module_name, structural_method_exports);
    if error_exports.is_empty() {
        external_defs.error_types.remove(module_name);
    } else {
        external_defs
            .error_types
            .insert(module_name.to_string(), error_exports);
    }
    if !rust_opaque_exports.is_empty() {
        external_defs
            .rust_opaque_classes
            .insert(module_name.to_string(), rust_opaque_exports);
    }
    if !class_field_default_exports.is_empty() {
        external_defs
            .class_field_defaults
            .insert(module_name.to_string(), class_field_default_exports);
    }
    if !lowering_result.declaration_metadata.is_empty() {
        external_defs.declaration_metadata.insert(
            module_name.to_string(),
            lowering_result.declaration_metadata.clone(),
        );
    }
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
    if !class_type_param_exports.is_empty() {
        external_defs
            .class_type_params
            .insert(module_name.to_string(), class_type_param_exports);
    }
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
mod tests {
    use crate::{
        CacheStatus, DiskSourceProvider, DocumentVersion, FrontendContext, FrontendInput,
        FrontendMode, ModuleId, OverlayDocument, OverlaySourceProvider, ProjectRoot,
        SourceDependencyKind, SourcePath, SourceText, TrackingSourceProvider, WorkspaceDirtyReason,
        WorkspaceDirtyScope,
    };

    fn input(source: &str) -> FrontendInput {
        FrontendInput {
            path: SourcePath::new("main.sifr"),
            source: SourceText::new(source),
            mode: FrontendMode::SingleFile,
        }
    }

    #[test]
    fn single_file_queries_are_cached_and_deterministic() {
        let mut context = FrontendContext::load_single_file(input(
            "def main():\n    value: int = 1\n    reveal_type(value)\n",
        ))
        .expect("context should load");

        let main_module = ModuleId(0);
        let first = context.diagnostics_for_module(main_module);
        let second = context.diagnostics_for_module(main_module);

        assert_eq!(first.value().module, main_module);
        assert_eq!(second.metadata().cache_status, CacheStatus::Hit);
        assert_eq!(first.value().diagnostics, second.value().diagnostics);
    }

    #[test]
    fn source_update_invalidates_cached_queries() {
        let mut context = FrontendContext::load_single_file(input("def main():\n    return 1\n"))
            .expect("context should load");
        let main_module = ModuleId(0);
        let _ = context.diagnostics_for_module(main_module);

        let report = context
            .update_module_source(
                main_module,
                SourceText::new("def main():\n    return 2\n"),
                Some(DocumentVersion::new(2)),
            )
            .expect("update should succeed");

        assert!(report.invalidated_modules.contains(&main_module));
        assert_eq!(
            context
                .diagnostics_for_module(main_module)
                .metadata()
                .cache_status,
            CacheStatus::Miss
        );
    }

    #[test]
    fn private_module_body_update_stays_local_when_signatures_match() {
        let dir = temp_project_dir("private_body_invalidation");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main() -> int:\n    return value()\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def value() -> int:\n    return 1\n",
        )
        .expect("helper should be written");
        let mut context = load_temp_project(&dir);
        let helper = ModuleId(1);
        let main = ModuleId(0);
        assert!(context
            .diagnostics_for_project()
            .into_value()
            .diagnostics
            .is_empty());

        let report = context
            .update_module_source(
                helper,
                SourceText::new("def value() -> int:\n    return 2\n"),
                Some(DocumentVersion::new(2)),
            )
            .expect("helper update should succeed");

        assert_eq!(report.invalidated_modules, vec![helper]);
        assert!(!report.invalidated_modules.contains(&main));
        assert_eq!(
            report.dirty_scope_report.scope,
            WorkspaceDirtyScope::OneModule {
                path: SourcePath::new(dir.join("helper.sifr"))
            }
        );
        assert_eq!(
            report.dirty_scope_report.reasons,
            vec![WorkspaceDirtyReason::SourceTextChanged]
        );
    }

    #[test]
    fn private_body_update_with_unchanged_imports_stays_local() {
        let dir = temp_project_dir("private_body_import_signature");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main() -> int:\n    return value()\n",
        )
        .expect("main should be written");
        std::fs::write(dir.join("dep.sifr"), "other: int = 1\n").expect("dep should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "from dep import other\n\ndef value() -> int:\n    return other\n",
        )
        .expect("helper should be written");
        let mut context = load_temp_project(&dir);
        let helper = ModuleId(2);
        let main = ModuleId(0);
        assert!(context
            .diagnostics_for_project()
            .into_value()
            .diagnostics
            .is_empty());

        let report = context
            .update_module_source(
                helper,
                SourceText::new(
                    "from dep import other\n\ndef value() -> int:\n    return other + 1\n",
                ),
                Some(DocumentVersion::new(2)),
            )
            .expect("helper update should succeed");

        assert_eq!(report.invalidated_modules, vec![helper]);
        assert!(!report.invalidated_modules.contains(&main));
        assert_eq!(
            report.dirty_scope_report.scope,
            WorkspaceDirtyScope::OneModule {
                path: SourcePath::new(dir.join("helper.sifr"))
            }
        );
        assert_eq!(
            report.dirty_scope_report.reasons,
            vec![WorkspaceDirtyReason::SourceTextChanged]
        );
    }

    #[test]
    fn public_export_update_invalidates_reverse_dependents() {
        let dir = temp_project_dir("export_signature_invalidation");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main() -> int:\n    return value()\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def value() -> int:\n    return 1\n",
        )
        .expect("helper should be written");
        let mut context = load_temp_project(&dir);
        let helper = ModuleId(1);
        let main = ModuleId(0);
        let _ = context.diagnostics_for_project();

        let report = context
            .update_module_source(
                helper,
                SourceText::new("def value() -> str:\n    return \"changed\"\n"),
                Some(DocumentVersion::new(2)),
            )
            .expect("helper update should succeed");

        assert_eq!(report.invalidated_modules, vec![main, helper]);
        assert_eq!(
            report.dirty_scope_report.scope,
            WorkspaceDirtyScope::ReverseDependencies {
                path: SourcePath::new(dir.join("helper.sifr"))
            }
        );
        assert_eq!(
            report.dirty_scope_report.reasons,
            vec![
                WorkspaceDirtyReason::SourceTextChanged,
                WorkspaceDirtyReason::ExportSignatureChanged
            ]
        );
    }

    #[test]
    fn import_signature_update_selects_graph_scope() {
        let dir = temp_project_dir("import_signature_invalidation");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main() -> int:\n    return value()\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def value() -> int:\n    return 1\n",
        )
        .expect("helper should be written");
        std::fs::write(
            dir.join("other.sifr"),
            "def other() -> int:\n    return 2\n",
        )
        .expect("other should be written");
        let mut context = load_temp_project(&dir);
        let main = ModuleId(0);
        let _ = context.diagnostics_for_project();

        let report = context
            .update_module_source(
                main,
                SourceText::new(
                    "from other import other\n\ndef main() -> int:\n    return other()\n",
                ),
                Some(DocumentVersion::new(2)),
            )
            .expect("main update should succeed");

        assert_eq!(
            report.dirty_scope_report.scope,
            WorkspaceDirtyScope::GraphStructure
        );
        assert_eq!(
            report.dirty_scope_report.reasons,
            vec![
                WorkspaceDirtyReason::SourceTextChanged,
                WorkspaceDirtyReason::ImportSignatureChanged
            ]
        );
    }

    #[test]
    fn project_graph_records_local_import_edges() {
        let dir = std::env::temp_dir().join(format!(
            "sifr_frontend_project_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("temp project should be created");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main():\n    print(value)\n",
        )
        .expect("main should be written");
        std::fs::write(dir.join("helper.sifr"), "value: int = 1\n")
            .expect("helper should be written");

        let mut context = FrontendContext::load_project(&ProjectRoot {
            root: SourcePath::new(&dir),
            entrypoint: SourcePath::new(dir.join("main.sifr")),
        })
        .expect("project should load");

        let graph = context.module_graph();
        assert_eq!(graph.entrypoint, ModuleId(0));
        assert_eq!(graph.edges.len(), 1);

        let diagnostics = context.diagnostics_for_project().into_value().diagnostics;
        assert!(
            diagnostics.is_empty(),
            "project diagnostics should consume dependency exports from the canonical frontend: {diagnostics:?}"
        );
    }

    #[test]
    fn project_loading_uses_overlay_and_tracking_provider() {
        let dir = std::env::temp_dir().join(format!(
            "sifr_frontend_project_overlay_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("temp project should be created");
        let main_path = dir.join("main.sifr");
        let helper_path = dir.join("helper.sifr");
        std::fs::write(
            &main_path,
            "from helper import value\n\ndef main():\n    print(value)\n",
        )
        .expect("main should be written");
        std::fs::write(&helper_path, "value: int = 1\n").expect("helper should be written");

        let mut overlay = OverlaySourceProvider::new(DiskSourceProvider::new());
        overlay.insert_overlay(OverlayDocument::new(
            SourcePath::new(&helper_path),
            None,
            DocumentVersion::new(5),
            SourceText::new("value: int = 2\n"),
            Some("value: int = 1\n"),
        ));
        let mut provider = TrackingSourceProvider::new(overlay);

        let context = FrontendContext::load_project_with_provider(
            &ProjectRoot {
                root: SourcePath::new(&dir),
                entrypoint: SourcePath::new(&main_path),
            },
            &mut provider,
        )
        .expect("project should load through provider");

        assert!(provider
            .dependencies()
            .iter()
            .any(|dependency| dependency.kind == SourceDependencyKind::DirectoryRead));
        assert!(context.source_map().files.iter().any(|file| {
            file.canonical_path.as_path() == helper_path
                && file.source.as_str() == "value: int = 2\n"
        }));
    }

    fn temp_project_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sifr_frontend_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("temp project should be created");
        dir
    }

    fn load_temp_project(dir: &std::path::Path) -> FrontendContext {
        FrontendContext::load_project(&ProjectRoot {
            root: SourcePath::new(dir),
            entrypoint: SourcePath::new(dir.join("main.sifr")),
        })
        .expect("project should load")
    }
}
