use super::{
    DocumentVersion, FileId, FrontendDiagnosticStyle, FrontendSourceContext, ModuleId, ModuleState,
    SourceHash, SourcePath, SourceText, SymbolKind, SymbolView,
};
pub(crate) use crate::export_type_localization::should_export_callable;
use crate::export_type_localization::{
    copy_class_generic_metadata, copy_function_generic_metadata, declared_generic_metadata,
    exported_parent_chain, imported_class_ancestry, reexport_class_aliases,
};
use crate::module_signatures::ModuleSignature;
use crate::{
    diagnostic_with_code, diagnostic_with_source_range, diagnostic_with_source_range_args_help,
};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_lowering::{
    canonicalize_user_export_function_type, canonicalize_user_export_type,
    localize_user_import_function_type, localize_user_import_type, ExternalDefs, HirDiagnostic,
    HirModule, LoweringResult, LoweringWarningDiagnostic, RevealTypeDiagnostic,
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

pub fn reveal_type_diagnostics(
    source_context: Option<FrontendSourceContext<'_>>,
    reveal_types: &[RevealTypeDiagnostic],
) -> Vec<RenderedDiagnostic> {
    reveal_types
        .iter()
        .map(|diagnostic| reveal_type_diagnostic(source_context, diagnostic))
        .collect()
}

pub fn warning_diagnostics(
    source_context: Option<FrontendSourceContext<'_>>,
    warnings: &[LoweringWarningDiagnostic],
) -> Vec<RenderedDiagnostic> {
    warnings
        .iter()
        .map(|diagnostic| warning_diagnostic(source_context, diagnostic))
        .collect()
}

pub(super) fn warning_diagnostic(
    source_context: Option<FrontendSourceContext<'_>>,
    diagnostic: &LoweringWarningDiagnostic,
) -> RenderedDiagnostic {
    let (code, message, message_template, args, primary_range) = match diagnostic {
        LoweringWarningDiagnostic::ArithmeticOverflowRisk {
            operation,
            primary_range,
        } => (
            DiagnosticCode::TYPE_ARITHMETIC_OVERFLOW_RISK,
            format!("integer {operation} may overflow at runtime"),
            "integer {operation} may overflow at runtime",
            vec![("operation", DiagnosticArg::String(operation.clone()))],
            *primary_range,
        ),
        LoweringWarningDiagnostic::UnreachableStatement { primary_range } => (
            DiagnosticCode::FLOW_UNREACHABLE_STATEMENT,
            "unreachable statement ignored".to_string(),
            "unreachable statement ignored",
            Vec::new(),
            *primary_range,
        ),
        LoweringWarningDiagnostic::BigIntTransitionAlias { primary_range } => (
            DiagnosticCode::INT_BIGINT_TRANSITION_ALIAS,
            "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values".to_string(),
            "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values",
            Vec::new(),
            *primary_range,
        ),
    };
    if let (Some(context), Some(range)) = (source_context, primary_range) {
        return diagnostic_with_source_range(code, context, range, message_template, &args);
    }
    rendered_spanless_diagnostic(code, message, message_template, &args)
}

pub(super) fn reveal_type_diagnostic(
    source_context: Option<FrontendSourceContext<'_>>,
    diagnostic: &RevealTypeDiagnostic,
) -> RenderedDiagnostic {
    let code = DiagnosticCode::TYPE_REVEAL_TYPE;
    let message = format!("revealed type is {}", diagnostic.revealed_type);
    let args = [(
        "revealed_type",
        DiagnosticArg::String(diagnostic.revealed_type.clone()),
    )];
    if let (Some(context), Some(range)) = (source_context, diagnostic.primary_range) {
        return diagnostic_with_source_range(
            code,
            context,
            range,
            "revealed type is {revealed_type}",
            &args,
        );
    }
    rendered_spanless_diagnostic(code, message, "revealed type is {revealed_type}", &args)
}

pub(super) fn rendered_spanless_diagnostic(
    code: DiagnosticCode,
    message: String,
    message_template: &'static str,
    args: &[(&'static str, DiagnosticArg)],
) -> RenderedDiagnostic {
    let mut rendered_args = BTreeMap::new();
    for (name, value) in args {
        rendered_args.insert((*name).to_string(), value.clone());
    }
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: message_template.to_string(),
        args: rendered_args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

pub fn collect_module_exports(
    module_name: &str,
    lowering_result: &LoweringResult,
    external_defs: &mut ExternalDefs,
) {
    let module = &lowering_result.module;
    let mut fn_exports = HashMap::new();
    let mut class_exports = HashMap::new();
    let mut class_type_param_exports = HashMap::new();
    let mut const_exports = HashMap::new();
    let mut const_integer_value_exports = HashMap::new();
    let mut default_exports = HashMap::new();
    let mut vararg_exports = HashMap::new();
    let mut python_shape_exports = HashMap::new();
    let mut workload_exports = HashMap::new();
    let (mut generic_exports, mut type_param_bound_exports, local_classes) =
        declared_generic_metadata(module_name, module);
    let imported_ancestry = imported_class_ancestry(module, external_defs);

    for func in &module.functions {
        if should_export_callable(module_name, &func.name) {
            let params: Vec<(String, Type, ParamConvention)> = func
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                .collect();
            let function_type = FunctionType {
                params,
                return_type: Box::new(func.return_type.clone()),
            };
            fn_exports.insert(
                func.name.clone(),
                canonicalize_user_export_function_type(&function_type, module_name, &local_classes),
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
        }
    }

    for (callable_name, defaults) in &lowering_result.function_defaults {
        if should_export_callable(module_name, callable_name) {
            default_exports.insert(callable_name.clone(), defaults.clone());
        }
    }

    for class in &module.classes {
        if !class.name.starts_with('_') {
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
                        params,
                        return_type: Box::new(op_func.return_type.clone()),
                    },
                ));
            }
            let class_ty = canonicalize_user_export_type(
                &Type::Class {
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
                module_name,
                &local_classes,
            );
            class_exports.insert(class.name.clone(), class_ty);
            if !class.type_params.is_empty() {
                class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
            }
        }
    }

    for (name, ty, _) in &module.constants {
        if !name.starts_with('_') {
            const_exports.insert(
                name.clone(),
                canonicalize_user_export_type(ty, module_name, &local_classes),
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
                    class_exports.insert(
                        local_name.clone(),
                        localize_user_import_type(class_type, &import.module, &class_aliases),
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
    external_defs
        .classes
        .insert(module_name.to_string(), class_exports);
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
