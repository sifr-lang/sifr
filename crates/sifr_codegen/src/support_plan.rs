use crate::error_refs::{
    collect_referenced_builtin_error_classes, collect_source_builtin_error_classes,
};
use crate::ir_imports::{
    IrImportNeeds, collect_import_needs_from_items, collect_import_needs_from_source,
};
use crate::ir_optimize::{
    remove_trivial_clones_in_items, remove_unneeded_mutability_in_items,
    remove_unread_pure_bindings_in_items, simplify_control_flow_in_items,
};
use crate::stdlib_demand_plan::plan_demanded_stdlib_sources;
use crate::stdlib_filter::{
    RustItemDeduper, absolutize_external_crate_paths, collect_and_strip_shared_prelude,
    dedup_rust_items, seal_canonical_stdlib_names,
};
use crate::{
    BUILTIN_ERROR_CLASSES, Renderer, RuntimeNeeds, RuntimeSupportDemand, RustEmitter, RustExpr,
    RustFile, RustItem, RustLiteral, StdlibCode, Type, build_async_exit_cause_type_items,
    build_cancellation_error_type_items, build_cpu_offload_items, build_error_into_error_impl,
    build_error_type_items, build_failure_type_items, build_file_handle_infra_items,
    build_file_handle_struct_items, build_generator_runtime_items, build_io_error_items,
    build_join_set_cpu_items, build_join_set_items, build_task_cancellation_items,
    build_task_context_scope_extension_items, build_task_current_context_items,
    build_task_scope_cpu_offload_items, build_task_scope_items, build_task_scope_offload_items,
    build_task_scope_process_items, build_task_supervisor_items, build_timeout_result_type_items,
    build_worker_panic_hook_items, replace_parallel_runtime_items,
    replace_sync_channel_runtime_items, sifr_type_to_rust_type, sync_channel_runtime_needed,
};
use sifr_ir::HirModule;
use sifr_stdlib_manifest::StdlibFeature;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;

#[derive(Clone, Default)]
pub(crate) struct ModuleSupportDemand {
    pub(crate) runtime: RuntimeSupportDemand,
    runtime_needs: RuntimeNeeds,
    intrinsic_functions: HashSet<String>,
    directly_used_stdlib_modules: HashSet<String>,
    imported_stdlib_names: HashMap<String, HashSet<String>>,
    suppressed_union_definitions: HashSet<String>,
    locally_shadowed_error_classes: HashSet<String>,
    referenced_error_classes: HashSet<String>,
    error_conversions: crate::error_refs::ErrorConversionDemand,
    error_conversion_paths: HashMap<String, String>,
    required_features: HashSet<StdlibFeature>,
    needs_file_handle_struct: bool,
    structural_interop_enabled: bool,
}

impl ModuleSupportDemand {
    pub(crate) fn from_emitter(
        module: &HirModule,
        emitter: &RustEmitter,
        module_name: Option<&str>,
    ) -> Self {
        let runtime = RuntimeSupportDemand::for_module(module);
        let needs_file_handles = emitter.runtime_needs.file_handles();
        let user_defined_error_classes = module
            .classes
            .iter()
            .filter(|class| class.is_error_type)
            .map(|class| class.name.as_str())
            .collect::<HashSet<_>>();
        let mut referenced_error_classes = collect_referenced_builtin_error_classes(
            module,
            "",
            &emitter.intrinsic_functions,
            needs_file_handles,
            BUILTIN_ERROR_CLASSES,
        );
        add_runtime_error_classes(&runtime, &mut referenced_error_classes);
        referenced_error_classes.retain(|name| !user_defined_error_classes.contains(name.as_str()));

        Self {
            runtime,
            runtime_needs: emitter.runtime_needs.clone(),
            intrinsic_functions: emitter.intrinsic_functions.clone(),
            directly_used_stdlib_modules: emitter.used_stdlib_modules.clone(),
            imported_stdlib_names: emitter.imported_stdlib_names.clone(),
            suppressed_union_definitions: emitter.union_enums.keys().cloned().collect(),
            locally_shadowed_error_classes: user_defined_error_classes
                .into_iter()
                .map(str::to_string)
                .collect(),
            referenced_error_classes,
            error_conversions: crate::error_refs::collect_error_conversion_demand(
                module,
                module_name,
            ),
            error_conversion_paths: HashMap::new(),
            required_features: emitter.intrinsic_registry_features.clone(),
            needs_file_handle_struct: needs_file_handles
                && !module
                    .classes
                    .iter()
                    .any(|class| class.name == "FileHandle"),
            structural_interop_enabled: emitter.structural_interop_enabled,
        }
    }

    /// Merge one named module into a crate-level support owner.
    ///
    /// Error-class shadows are deliberately module-local. The shared owner
    /// must retain builtins required by a sibling module or by compiler-owned
    /// runtime and stdlib support.
    pub(crate) fn merge_project_module(&mut self, other: &Self) {
        assert!(
            self.locally_shadowed_error_classes.is_empty(),
            "project support demand must start from the unshadowed aggregate identity"
        );
        self.runtime.merge(&other.runtime);
        self.runtime_needs.merge(&other.runtime_needs);
        self.intrinsic_functions
            .extend(other.intrinsic_functions.iter().cloned());
        self.directly_used_stdlib_modules
            .extend(other.directly_used_stdlib_modules.iter().cloned());
        for (module, names) in &other.imported_stdlib_names {
            self.imported_stdlib_names
                .entry(module.clone())
                .or_default()
                .extend(names.iter().cloned());
        }
        self.suppressed_union_definitions
            .extend(other.suppressed_union_definitions.iter().cloned());
        self.referenced_error_classes
            .extend(other.referenced_error_classes.iter().cloned());
        self.error_conversions.merge(&other.error_conversions);
        self.required_features
            .extend(other.required_features.iter().copied());
        self.needs_file_handle_struct |= other.needs_file_handle_struct;
        self.structural_interop_enabled |= other.structural_interop_enabled;
    }

    pub(crate) fn set_error_conversion_paths(&mut self, paths: &HashMap<String, String>) {
        self.error_conversion_paths.clone_from(paths);
    }

    pub(crate) fn needs_support(&self) -> bool {
        !self.runtime.is_empty()
            || self.runtime_needs.file_handles()
            || !self.directly_used_stdlib_modules.is_empty()
            || !self.referenced_error_classes.is_empty()
            || !self.error_conversions.is_empty()
    }

    pub(crate) fn directly_used_stdlib_modules(&self) -> HashSet<String> {
        self.directly_used_stdlib_modules.clone()
    }

    pub(crate) fn base_required_features(&self) -> HashSet<StdlibFeature> {
        let mut features = self.required_features.clone();
        if self.structural_interop_enabled {
            features.insert(StdlibFeature::StructuralRuntime);
        }
        features
    }
}

pub(crate) struct RenderedSupport {
    pub(crate) source: String,
    pub(crate) import_needs: IrImportNeeds,
    pub(crate) required_features: HashSet<StdlibFeature>,
    pub(crate) used_stdlib_modules: HashSet<String>,
}

pub(crate) fn render_support(
    demand: &ModuleSupportDemand,
    stdlib_code: &StdlibCode,
) -> RenderedSupport {
    let stdlib = render_stdlib_support(demand, stdlib_code);
    let needs_file_handles = demand.runtime_needs.file_handles() || stdlib.needs_file_handles;
    let mut referenced_error_classes = demand.referenced_error_classes.clone();
    referenced_error_classes.extend(collect_source_builtin_error_classes(
        &stdlib.source,
        BUILTIN_ERROR_CLASSES,
    ));
    if needs_file_handles {
        referenced_error_classes.insert("IOError".to_string());
    }
    add_runtime_error_classes(&demand.runtime, &mut referenced_error_classes);
    referenced_error_classes.retain(|name| !demand.locally_shadowed_error_classes.contains(name));

    let stdlib_emits_task_context = stdlib.source.contains("__sifr_task_current_context")
        || stdlib.source.contains("__SIFR_TASK_CONTEXT_LABEL");
    let uses_task_current_context = demand.intrinsic_functions.contains("task_current_context")
        && !stdlib.source.contains("__sifr_task_current_context");
    let uses_worker_panic_hook = demand.runtime.needs_worker_panic_hook()
        || stdlib
            .source
            .contains("__sifr_with_silent_worker_panic_hook");

    let mut items = build_error_items(&referenced_error_classes);
    if referenced_error_classes.contains("Error") {
        for &error_name in BUILTIN_ERROR_CLASSES {
            if error_name == "Error" || sifr_type_system::io_error_kind(error_name).is_some() {
                continue;
            }
            if referenced_error_classes.contains(error_name) {
                items.push(build_error_into_error_impl(error_name));
            }
        }
    }
    if referenced_error_classes.contains("Error") {
        items.extend(
            demand
                .error_conversions
                .render(&demand.error_conversion_paths),
        );
    }

    let uses_task_runtime = demand.runtime.task_scope || demand.runtime.join_set;
    if uses_task_runtime || demand.runtime.failure_type {
        items.extend(build_failure_type_items());
    }
    if uses_task_runtime || demand.runtime.cancellation_error_type {
        items.extend(build_cancellation_error_type_items());
    }
    if demand.runtime.async_exit_cause_type {
        items.extend(build_async_exit_cause_type_items());
    }
    if demand.runtime.timeout_result_type && !demand.runtime.task_scope {
        items.extend(build_timeout_result_type_items());
    }
    let needs_generator_common = demand.runtime.sync_generator || demand.runtime.async_generator;
    items.extend(build_generator_runtime_items(
        needs_generator_common && !stdlib.source.contains("struct __SifrYielder<"),
        demand.runtime.sync_generator && !stdlib.source.contains("struct __SifrGenerator<"),
        demand.runtime.async_generator && !stdlib.source.contains("struct AsyncGenerator<"),
    ));
    if demand.runtime.template {
        items.extend(crate::build_template_runtime_items());
    }
    if needs_file_handles {
        items.extend(build_file_handle_infra_items());
        if demand.needs_file_handle_struct && !stdlib.provides_file_handle_struct {
            items.extend(build_file_handle_struct_items());
        }
    }
    if uses_task_runtime || demand.runtime.async_python || demand.runtime.native_async_cleanup {
        items.extend(build_task_cancellation_items(
            demand.runtime.async_python || demand.runtime.native_async_cleanup,
            uses_task_runtime || demand.runtime.native_async_cleanup,
        ));
    }
    if uses_task_runtime {
        items.extend(build_task_scope_items());
        items.extend(build_task_supervisor_items());
        items.extend(build_task_context_scope_extension_items(
            !stdlib_emits_task_context,
        ));
    }
    if uses_task_current_context {
        items.extend(build_task_current_context_items(!uses_task_runtime));
    }
    if demand.runtime.task_scope_offload {
        items.extend(build_task_scope_offload_items());
    }
    if demand.runtime.task_scope_process {
        items.extend(build_task_scope_process_items());
    }
    if demand.runtime.task_scope_spawn_cpu {
        items.extend(build_task_scope_cpu_offload_items());
    }
    if demand.runtime.join_set {
        items.extend(build_join_set_items());
    }
    if uses_worker_panic_hook {
        items.extend(build_worker_panic_hook_items());
    }
    if demand.runtime.join_set_spawn_cpu {
        items.extend(build_join_set_cpu_items());
    }
    if demand.runtime.spawn_cpu {
        items.extend(build_cpu_offload_items());
    }

    remove_trivial_clones_in_items(&mut items);
    simplify_control_flow_in_items(&mut items);
    remove_unread_pure_bindings_in_items(&mut items);
    remove_unneeded_mutability_in_items(&mut items, &HashSet::new());

    let item_source = Renderer::new().render_file(&RustFile {
        items: items.clone(),
    });
    let source = [stdlib.source.trim(), item_source.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut import_needs = collect_import_needs_from_source(&stdlib.source);
    import_needs.merge(&collect_import_needs_from_items(&items));
    if needs_file_handles {
        import_needs.runtime.needs_mutex = true;
    }

    let mut required_features = demand.required_features.clone();
    add_import_features(&import_needs, &mut required_features);
    if demand.structural_interop_enabled {
        required_features.insert(StdlibFeature::StructuralRuntime);
    }
    if demand.runtime.task_sleep
        || demand.runtime.task_scope
        || demand.runtime.join_set
        || demand.runtime.async_python
        || demand.runtime.native_async_cleanup
        || source.contains("tokio::")
    {
        required_features.insert(StdlibFeature::Tokio);
    }
    if demand.runtime.needs_worker_panic_hook() || source.contains("rayon::") {
        required_features.insert(StdlibFeature::Rayon);
    }
    if crate::python_interop_common::rust_source_uses_python_runtime(&source) {
        required_features.insert(StdlibFeature::PythonRuntime);
    }
    if source.contains("::sifr_stdlib::fs::") {
        required_features.insert(StdlibFeature::Fs);
    }

    RenderedSupport {
        source,
        import_needs,
        required_features,
        used_stdlib_modules: stdlib.used_modules,
    }
}

pub(crate) fn render_import_items(needs: &IrImportNeeds) -> Vec<RustItem> {
    let mut items = Vec::new();
    if needs.collections.needs_hashmap {
        items.push(use_item(&["std", "collections", "HashMap"]));
    }
    if needs.collections.needs_hashset {
        items.push(use_item(&["std", "collections", "HashSet"]));
    }
    if needs.collections.needs_vecdeque {
        items.push(use_item(&["std", "collections", "VecDeque"]));
    }
    if needs.runtime.numeric.needs_bigint {
        items.push(use_item(&["num_bigint", "BigInt"]));
    }
    if needs.runtime.numeric.needs_decimal {
        items.push(use_item(&["rust_decimal", "Decimal"]));
    }
    if needs.runtime.numeric.needs_bigdecimal {
        items.push(use_item(&["bigdecimal", "BigDecimal"]));
    }
    if needs.runtime.needs_sifr_int {
        items.push(RustItem::Use(vec![
            String::new(),
            "sifr_runtime".to_string(),
            "SifrInt".to_string(),
        ]));
    }
    if needs.runtime.needs_sifr_range {
        items.push(RustItem::Use(vec![
            String::new(),
            "sifr_runtime".to_string(),
            "SifrRange".to_string(),
        ]));
    }
    if needs.runtime.needs_mutex {
        items.push(use_item(&["std", "sync", "Mutex"]));
    }
    items
}

pub(crate) fn add_import_features(needs: &IrImportNeeds, features: &mut HashSet<StdlibFeature>) {
    if needs.runtime.numeric.needs_bigint {
        features.insert(StdlibFeature::NumBigint);
        features.insert(StdlibFeature::NumTraits);
    }
    if needs.runtime.numeric.needs_decimal {
        features.insert(StdlibFeature::RustDecimal);
    }
    if needs.runtime.numeric.needs_bigdecimal {
        features.insert(StdlibFeature::BigDecimal);
    }
    if needs.runtime.needs_sifr_runtime {
        features.insert(StdlibFeature::SifrRuntime);
    }
}

struct StdlibSupport {
    source: String,
    used_modules: HashSet<String>,
    needs_file_handles: bool,
    provides_file_handle_struct: bool,
}

fn render_stdlib_support(demand: &ModuleSupportDemand, stdlib_code: &StdlibCode) -> StdlibSupport {
    let mut module_order = Vec::new();
    let mut seen_modules = HashSet::new();
    for module_name in demand
        .directly_used_stdlib_modules
        .iter()
        .collect::<BTreeSet<_>>()
    {
        append_stdlib_dependencies(
            module_name,
            stdlib_code,
            &mut seen_modules,
            &mut module_order,
        );
    }
    let planned = plan_demanded_stdlib_sources(
        stdlib_code,
        &module_order,
        &demand.directly_used_stdlib_modules,
        &demand.imported_stdlib_names,
        &demand.suppressed_union_definitions,
    );
    let mut skip_types = BUILTIN_ERROR_CLASSES
        .iter()
        .map(|name| (*name).to_string())
        .collect::<HashSet<_>>();
    skip_types.extend(
        sifr_type_system::IO_ERROR_KIND_CASES
            .iter()
            .map(|(name, _)| (*name).to_string()),
    );
    skip_types.insert("__io_err".to_string());
    skip_types.extend(demand.suppressed_union_definitions.iter().cloned());

    let mut emitted_items = RustItemDeduper::default();
    let mut source = String::new();
    let mut needs_file_handles = false;
    let mut provides_file_handle_struct = false;
    for module_name in &module_order {
        let Some(module_source) = stdlib_code.module_rust_code.get(module_name) else {
            continue;
        };
        let mut filtered = planned.get(module_name).cloned().unwrap_or_default();
        if module_name == "sifr.sync" && sync_channel_runtime_needed(&filtered) {
            filtered = replace_sync_channel_runtime_items(&filtered);
        }
        if module_name == "sifr.parallel" {
            filtered = replace_parallel_runtime_items(
                &filtered,
                &demand.runtime.parallel.function_names(),
            );
        }
        if filtered.trim().is_empty() {
            continue;
        }
        let prepared = collect_and_strip_shared_prelude(&filtered);
        needs_file_handles |= prepared.shared_needs.file_handles.needs_file_handles;
        provides_file_handle_struct |= prepared
            .shared_needs
            .file_handles
            .provides_file_handle_struct;
        let sealed = seal_canonical_stdlib_names(
            &prepared.stripped_code,
            &module_source.module,
            &module_source.nominal_types,
        );
        let absolute = absolutize_external_crate_paths(&sealed);
        let deduped = dedup_rust_items(&absolute, &mut emitted_items, &skip_types);
        if deduped.trim().is_empty() {
            continue;
        }
        let _ = writeln!(source, "// --- stdlib: {module_name} ---");
        source.push_str(&deduped);
        source.push('\n');
    }

    StdlibSupport {
        source,
        used_modules: seen_modules,
        needs_file_handles,
        provides_file_handle_struct,
    }
}

fn append_stdlib_dependencies(
    module_name: &str,
    stdlib_code: &StdlibCode,
    seen_modules: &mut HashSet<String>,
    module_order: &mut Vec<String>,
) {
    if !seen_modules.insert(module_name.to_string()) {
        return;
    }
    if let Some(dependencies) = stdlib_code.transitive_deps.get(module_name) {
        for dependency in dependencies.iter().collect::<BTreeSet<_>>() {
            if dependency.starts_with("sifr.") || dependency.starts_with("_sifr.") {
                append_stdlib_dependencies(dependency, stdlib_code, seen_modules, module_order);
            }
        }
    }
    module_order.push(module_name.to_string());
}

fn build_error_items(referenced: &HashSet<String>) -> Vec<RustItem> {
    let mut items = Vec::new();
    let io_error_referenced = referenced.contains("IOError")
        || sifr_type_system::IO_ERROR_KIND_CASES
            .iter()
            .any(|(subclass, _)| referenced.contains(*subclass));
    if io_error_referenced {
        items.extend(build_io_error_items());
    }
    for &error_name in BUILTIN_ERROR_CLASSES {
        if error_name == "IOError" || sifr_type_system::io_error_kind(error_name).is_some() {
            continue;
        }
        if !referenced.contains(error_name) {
            continue;
        }
        let exact_zero = || RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "SifrInt".to_string(),
                "from_i64".to_string(),
            ])),
            args: vec![RustExpr::Literal(RustLiteral::Int(0))],
        };
        let (extra_fields, defaults) = match error_name {
            "JSONDecodeError" | "TOMLDecodeError" => (
                vec![
                    ("line".to_string(), sifr_type_to_rust_type(&Type::Int)),
                    ("column".to_string(), sifr_type_to_rust_type(&Type::Int)),
                ],
                vec![
                    ("line".to_string(), exact_zero()),
                    ("column".to_string(), exact_zero()),
                ],
            ),
            "JsonIntegerRangeError" => (
                vec![
                    ("path".to_string(), sifr_type_to_rust_type(&Type::Str)),
                    ("profile".to_string(), sifr_type_to_rust_type(&Type::Str)),
                ],
                vec![
                    (
                        "path".to_string(),
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "String".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        },
                    ),
                    (
                        "profile".to_string(),
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "String".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        },
                    ),
                ],
            ),
            "JsonLimitError" | "ArithmeticLimitError" => (
                vec![("limit".to_string(), sifr_type_to_rust_type(&Type::Int))],
                vec![("limit".to_string(), exact_zero())],
            ),
            "RegexError" => (
                vec![("detail".to_string(), sifr_type_to_rust_type(&Type::Str))],
                vec![(
                    "detail".to_string(),
                    RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "String".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    },
                )],
            ),
            _ => (vec![], vec![]),
        };
        items.extend(build_error_type_items(error_name, &extra_fields, &defaults));
    }
    items
}

fn add_runtime_error_classes(runtime: &RuntimeSupportDemand, referenced: &mut HashSet<String>) {
    if runtime.task_scope
        || runtime.join_set
        || runtime.failure_type
        || runtime.native_async_cleanup
    {
        referenced.insert("SecondaryError".to_string());
    }
    if runtime.async_generator {
        referenced.insert("GeneratorCloseError".to_string());
    }
    if runtime.needs_worker_panic_hook() {
        referenced.insert("WorkerRuntimeError".to_string());
        referenced.insert("WorkerError".to_string());
    }
}

fn use_item(path: &[&str]) -> RustItem {
    RustItem::Use(path.iter().map(|segment| (*segment).to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::{ModuleSupportDemand, render_support};
    use crate::{StdlibCode, StdlibRustSource};
    use std::collections::HashSet;

    #[test]
    fn single_file_user_error_suppresses_late_runtime_demand() {
        let mut demand = ModuleSupportDemand::default();
        demand.runtime.task_scope = true;
        demand
            .locally_shadowed_error_classes
            .insert("SecondaryError".to_string());

        let rendered = render_support(&demand, &StdlibCode::default());

        assert!(!rendered.source.contains("struct SecondaryError"));
    }

    #[test]
    fn project_merge_does_not_promote_a_module_shadow_to_a_crate_veto() {
        let mut module = ModuleSupportDemand::default();
        module
            .locally_shadowed_error_classes
            .insert("ValueError".to_string());
        module
            .directly_used_stdlib_modules
            .insert("sifr.fixture".to_string());

        let mut project = ModuleSupportDemand::default();
        project.merge_project_module(&module);

        let mut stdlib = StdlibCode::default();
        stdlib.module_rust_code.insert(
            "sifr.fixture".to_string(),
            StdlibRustSource {
                module: "sifr.fixture".to_string(),
                source_path: "stdlib/sifr/fixture.sifr".to_string(),
                source_sha256: "error-identity-fixture".to_string(),
                nominal_types: HashSet::new(),
                rust: "fn operation() -> Result<(), ValueError> { Ok(()) }\n".to_string(),
            },
        );

        let rendered = render_support(&project, &stdlib);

        assert!(rendered.source.contains("struct ValueError"));
    }
}
