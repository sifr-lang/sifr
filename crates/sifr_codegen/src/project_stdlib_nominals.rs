use crate::builtin_errors::BuiltinError;
use crate::stdlib_filter::{
    partition_rust_items_by_name, rust_source_defined_item_names, rust_source_references_item_name,
};
use crate::{HirModule, Renderer, RustFile, StdlibCode, publicize_generated_module_source};
use sifr_ir::HirFunction;
use sifr_type_system::{FunctionType, Type, class_rust_name, is_crate_root_rust_nominal_identity};
use std::collections::{BTreeMap, HashMap, HashSet};

mod module_bindings;
mod project_union_relocation;
mod registry;
mod relocation;
#[cfg(test)]
use relocation::relocate_project_stdlib_nominals_owned_by;
pub(crate) use relocation::{RelocatedStructuralImplementations, relocate_project_stdlib_nominals};

pub(crate) use module_bindings::project_module_binding_names;
pub(crate) use registry::ProjectNominalRegistry;

use project_union_relocation::{relocate_project_unions, shared_nominal_reexport_names};

const SHARED_STDLIB_NOMINAL_MODULE: &str = "__sifr_project_nominals";
const SHARED_MODULE_CRATE_ROOT_NOMINAL_IDENTITIES: &[&str] = &["_sifr.fs.NativeFileHandle"];

pub(crate) struct ProjectStdlibNominalPlan {
    pub(crate) registry: ProjectNominalRegistry,
}

impl ProjectStdlibNominalPlan {
    pub(crate) fn empty() -> Self {
        Self {
            registry: ProjectNominalRegistry::default(),
        }
    }
}

pub(crate) fn project_stdlib_nominal_plan(
    unions: &HashMap<String, Vec<Type>>,
    modules: &[(&str, &HirModule)],
) -> ProjectStdlibNominalPlan {
    let mut declarations = BTreeMap::<String, HashSet<String>>::new();
    let mut builtin_types = BTreeMap::<BuiltinError, Type>::new();
    for members in unions.values() {
        for member in members {
            collect_shared_nominals(member, &mut declarations, &mut builtin_types);
        }
    }
    for (_, module) in modules {
        collect_module_nominals(module, &mut declarations, &mut builtin_types);
        let local_error_names = module
            .classes
            .iter()
            .filter(|class| class.is_error_type)
            .map(|class| class.name.as_str())
            .collect::<HashSet<_>>();
        let intrinsic_functions =
            crate::error_refs::collect_module_intrinsic_function_names(module);
        let referenced = crate::error_refs::collect_referenced_builtin_error_classes(
            module,
            "",
            &intrinsic_functions,
            false,
            crate::BUILTIN_ERROR_CLASSES,
        );
        for builtin in BuiltinError::all() {
            let name = builtin.name();
            if !referenced.contains(name)
                || local_error_names.contains(name)
                || sifr_type_system::io_error_kind(name).is_some()
            {
                continue;
            }
            builtin_types
                .entry(builtin)
                .or_insert_with(|| builtin_error_type(name));
        }
    }
    if builtin_types
        .keys()
        .any(|builtin| builtin.name() == "Error")
    {
        for (_, module) in modules {
            if !crate::python_interop_common::module_uses_async_python_declaration(module) {
                continue;
            }
            for (_, ty) in crate::python_interop_common::python_error_contract_types(module) {
                collect_shared_nominals(&ty, &mut declarations, &mut builtin_types);
            }
        }
    }
    let mut registry = ProjectNominalRegistry::default();
    for (module, names) in declarations {
        let mut names = names.into_iter().collect::<Vec<_>>();
        names.sort();
        for name in &names {
            let identity = format!("{module}.{name}");
            let rust_name = class_rust_name(Some(&identity), name);
            registry.register_shared(identity, rust_name);
        }
    }
    for (builtin, ty) in builtin_types {
        let name = builtin.name();
        let rust_name = class_rust_name(None, name);
        debug_assert!(
            matches!(&ty, Type::Class { identity, .. } if is_compiler_builtin_error(identity.as_deref(), name)),
            "project builtin registry accepted a non-builtin nominal identity"
        );
        if let Type::Class {
            identity: Some(identity),
            ..
        } = &ty
        {
            registry.register_shared(identity.clone(), rust_name.clone());
        }
        registry.register_builtin(builtin, rust_name);
    }
    for identity in SHARED_MODULE_CRATE_ROOT_NOMINAL_IDENTITIES {
        if let Some((_, name)) = identity.rsplit_once('.') {
            registry.register_crate_root(
                (*identity).to_string(),
                class_rust_name(Some(identity), name),
            );
        }
    }

    ProjectStdlibNominalPlan { registry }
}

/// Establish ownership before per-module structural implementations relocate.
/// Some imported static APIs do not occur in an ordinary HIR value signature.
pub(crate) fn register_imported_structural_nominals(
    modules: &[(&str, &HirModule)],
    stdlib: &crate::StdlibEmissionCode,
    plan: &mut ProjectStdlibNominalPlan,
) {
    let mut declarations = BTreeMap::new();
    for (_, module) in modules {
        for class in crate::structural_impl_codegen::imported_stdlib_classes(module, stdlib) {
            // Opaque declarations belong to their runtime provider, not to a
            // generated shared module. They emit no local structural contract.
            if class.python_opaque_declaration().is_some()
                || class.rust_interop.iter().any(|declaration| {
                    declaration.kind == sifr_ir::RustInteropDecoratorKind::Opaque
                })
            {
                continue;
            }
            collect_nominal_identity(class.identity.as_deref(), &mut declarations);
        }
    }
    for (module, names) in declarations {
        for name in names {
            let identity = format!("{module}.{name}");
            let rust_name = class_rust_name(Some(&identity), &name);
            plan.registry.register_shared(identity, rust_name);
        }
    }
}

pub(crate) fn extract_project_stdlib_nominal_prelude(
    support_source: &str,
    unions: &HashMap<String, Vec<Type>>,
    stdlib_code: &StdlibCode,
    plan: &mut ProjectStdlibNominalPlan,
) -> (String, String) {
    register_emitted_builtin_nominals(support_source, &mut plan.registry);
    register_transitive_stdlib_nominals(support_source, stdlib_code, &mut plan.registry);
    let (shared_source, relocated_project_unions) =
        relocate_project_unions(support_source, unions, &mut plan.registry);
    let nominal_name_refs = plan
        .registry
        .shared_rust_names
        .iter()
        .chain(&plan.registry.crate_root_rust_names)
        .chain(&relocated_project_unions)
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let (shared_source, remaining_support) =
        partition_rust_items_by_name(&shared_source, &nominal_name_refs);
    let crate_root_candidates = SHARED_MODULE_CRATE_ROOT_NOMINAL_IDENTITIES
        .iter()
        .filter_map(|identity| {
            identity
                .rsplit_once('.')
                .map(|(_, name)| class_rust_name(Some(identity), name))
        })
        .collect::<HashSet<_>>();
    let crate_root_name_refs = crate_root_candidates
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let (crate_root_source, shared_source) =
        partition_rust_items_by_name(&shared_source, &crate_root_name_refs);
    let crate_root_defined_names = rust_source_defined_item_names(&crate_root_source);
    for identity in SHARED_MODULE_CRATE_ROOT_NOMINAL_IDENTITIES {
        let Some((_, name)) = identity.rsplit_once('.') else {
            continue;
        };
        let rust_name = class_rust_name(Some(identity), name);
        if crate_root_defined_names.contains(&rust_name) {
            plan.registry
                .register_crate_root((*identity).to_string(), rust_name);
        }
    }
    let crate_root_source = publicize_generated_module_source(&crate_root_source);
    let shared_source = publicize_generated_module_source(&shared_source);
    let nominal_imports = Renderer::new().render_file(&RustFile {
        items: crate::render_import_items(&crate::ir_imports::collect_import_needs_from_source(
            &shared_source,
        )),
    });
    let mut prelude = crate_root_source;
    if !prelude.is_empty() {
        prelude.push('\n');
    }
    prelude.push_str("mod ");
    prelude.push_str(SHARED_STDLIB_NOMINAL_MODULE);
    prelude.push_str(" {\n");
    let mut crate_root_imports = plan
        .registry
        .crate_root_rust_names
        .iter()
        .chain(&relocated_project_unions)
        .filter(|name| rust_source_references_item_name(&shared_source, name))
        .collect::<Vec<_>>();
    crate_root_imports.sort();
    crate_root_imports.dedup();
    for name in crate_root_imports {
        prelude.push_str("    use crate::");
        prelude.push_str(name);
        prelude.push_str(";\n");
    }
    for line in nominal_imports.lines() {
        prelude.push_str("    ");
        prelude.push_str(line);
        prelude.push('\n');
    }
    for line in shared_source.lines() {
        prelude.push_str("    ");
        prelude.push_str(line);
        prelude.push('\n');
    }
    prelude.push_str("}\n");
    let ordered_names = shared_nominal_reexport_names(&plan.registry, &relocated_project_unions);
    for name in ordered_names {
        prelude.push_str("pub use ");
        prelude.push_str(SHARED_STDLIB_NOMINAL_MODULE);
        prelude.push_str("::");
        prelude.push_str(&name);
        prelude.push_str(";\n");
    }

    (prelude, remaining_support)
}

fn builtin_error_type(name: &str) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: (name != "Error").then(|| "Error".to_string()),
    }
}

fn register_transitive_stdlib_nominals(
    shared_source: &str,
    stdlib_code: &StdlibCode,
    registry: &mut ProjectNominalRegistry,
) {
    let defined_names = rust_source_defined_item_names(shared_source);
    for (module, source) in &stdlib_code.module_rust_code {
        for name in &source.nominal_types {
            let identity = format!("{module}.{name}");
            if is_crate_root_rust_nominal_identity(&identity) {
                continue;
            }
            let rust_name = class_rust_name(Some(&identity), name);
            if defined_names.contains(&rust_name) {
                registry.register_shared(identity, rust_name);
            }
        }
    }
}

fn register_emitted_builtin_nominals(shared_source: &str, registry: &mut ProjectNominalRegistry) {
    let defined_names = rust_source_defined_item_names(shared_source);
    for builtin in BuiltinError::all() {
        let name = builtin.name();
        if sifr_type_system::io_error_kind(name).is_some() || !defined_names.contains(name) {
            continue;
        }
        registry.register_builtin(builtin, class_rust_name(None, name));
    }
}

fn collect_function_nominals(
    function: &FunctionType,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<BuiltinError, Type>,
) {
    for (_, parameter, _) in &function.params {
        collect_shared_nominals(parameter, declarations, builtin_types);
    }
    collect_shared_nominals(&function.return_type, declarations, builtin_types);
}

fn collect_hir_function_nominals(
    function: &HirFunction,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<BuiltinError, Type>,
) {
    for parameter in &function.params {
        collect_shared_nominals(&parameter.ty, declarations, builtin_types);
    }
    collect_shared_nominals(&function.return_type, declarations, builtin_types);
    for ty in crate::hir_analysis::queries::collect_let_declared_types(&function.body) {
        collect_shared_nominals(&ty, declarations, builtin_types);
    }
    // Constructor-only references still need the same canonical project owner
    // as annotations and bindings (notably consuming error upcasts).
    crate::hir_analysis::traversal::walk_stmts(
        &function.body,
        crate::hir_analysis::traversal::TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
        &mut |_| {},
        &mut |expr| collect_shared_nominals(expr.ty(), declarations, builtin_types),
    );
}

fn collect_module_nominals(
    module: &HirModule,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<BuiltinError, Type>,
) {
    for function in &module.functions {
        collect_hir_function_nominals(function, declarations, builtin_types);
    }
    for class in &module.classes {
        for (_, field) in &class.fields {
            collect_shared_nominals(field, declarations, builtin_types);
        }
        if let Some(parent) = &class.parent_type {
            collect_shared_nominals(parent, declarations, builtin_types);
        }
        for method in &class.methods {
            collect_hir_function_nominals(method, declarations, builtin_types);
        }
        for (_, operator) in &class.operator_impls {
            collect_hir_function_nominals(operator, declarations, builtin_types);
        }
    }
    for (_, ty, _) in &module.constants {
        collect_shared_nominals(ty, declarations, builtin_types);
    }
}

fn collect_shared_nominals(
    ty: &Type,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<BuiltinError, Type>,
) {
    match ty.resolve_alias() {
        class @ Type::Class {
            identity,
            type_args,
            name,
            fields,
            methods,
            ..
        } => {
            if let Some(builtin) = compiler_builtin_error(identity.as_deref(), name)
                && sifr_type_system::io_error_kind(name).is_none()
            {
                builtin_types
                    .entry(builtin)
                    .or_insert_with(|| class.clone());
            }
            if !class.is_python_object_contract() && !class.is_python_resource_identity_contract() {
                collect_nominal_identity(identity.as_deref(), declarations);
            }
            for type_arg in type_args {
                collect_shared_nominals(type_arg, declarations, builtin_types);
            }
            if identity.as_deref().is_some_and(|identity| {
                identity.starts_with("sifr.") || identity.starts_with("_sifr.")
            }) {
                for (_, field) in fields {
                    collect_shared_nominals(field, declarations, builtin_types);
                }
                for (_, method) in methods {
                    collect_function_nominals(method, declarations, builtin_types);
                }
            }
        }
        Type::Protocol {
            identity, methods, ..
        } => {
            collect_nominal_identity(identity.as_deref(), declarations);
            for (_, method) in methods {
                collect_function_nominals(method, declarations, builtin_types);
            }
        }
        Type::Newtype {
            identity, inner, ..
        } => {
            collect_nominal_identity(identity.as_deref(), declarations);
            collect_shared_nominals(inner, declarations, builtin_types);
        }
        Type::Enum { identity, .. } => collect_nominal_identity(identity.as_deref(), declarations),
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner) => {
            collect_shared_nominals(inner, declarations, builtin_types);
        }
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Coroutine(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            collect_shared_nominals(left, declarations, builtin_types);
            collect_shared_nominals(right, declarations, builtin_types);
        }
        Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => {
            for item in items {
                collect_shared_nominals(item, declarations, builtin_types);
            }
        }
        Type::Function(function) | Type::AsyncFunction(function) => {
            collect_function_nominals(function, declarations, builtin_types);
        }
        Type::Callable(parameters, _, result) | Type::AsyncCallable(parameters, _, result) => {
            for parameter in parameters {
                collect_shared_nominals(parameter, declarations, builtin_types);
            }
            collect_shared_nominals(result, declarations, builtin_types);
        }
        _ => {}
    }
}

fn collect_nominal_identity(
    identity: Option<&str>,
    declarations: &mut BTreeMap<String, HashSet<String>>,
) {
    let Some(identity) = identity else {
        return;
    };
    if is_crate_root_rust_nominal_identity(identity)
        || (!identity.starts_with("sifr.") && !identity.starts_with("_sifr."))
    {
        return;
    }
    let Some((module, name)) = identity.rsplit_once('.') else {
        return;
    };
    if is_compiler_builtin_error(Some(identity), name) {
        return;
    }
    declarations
        .entry(module.to_string())
        .or_default()
        .insert(name.to_string());
}

fn is_compiler_builtin_error(identity: Option<&str>, name: &str) -> bool {
    compiler_builtin_error(identity, name).is_some()
}

fn compiler_builtin_error(identity: Option<&str>, name: &str) -> Option<BuiltinError> {
    BuiltinError::from_name(name).filter(|_| {
        identity.is_none_or(|identity| {
            identity.starts_with("sifr.builtin.")
                || sifr_type_system::is_global_rust_nominal_identity(identity)
        })
    })
}

#[cfg(test)]
#[path = "../tests/unit/project_stdlib_nominals/tests.rs"]
mod tests;
