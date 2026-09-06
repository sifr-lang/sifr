use crate::builtin_errors::BuiltinError;
use crate::stdlib_filter::{
    partition_rust_items_by_name, rust_source_defined_item_names, rust_source_references_item_name,
    strip_relocated_rust_items_by_name,
};
use crate::{HirModule, Renderer, RustFile, StdlibCode, publicize_generated_module_source};
use sifr_ir::HirFunction;
use sifr_type_system::{FunctionType, Type, class_rust_name, is_crate_root_rust_nominal_identity};
use std::collections::{BTreeMap, HashMap, HashSet};

mod module_bindings;
mod project_union_relocation;
mod registry;

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

pub(crate) fn relocate_project_stdlib_nominals(
    source: &str,
    module_name: &str,
    plan: &ProjectStdlibNominalPlan,
    crate_root_modules: &HashSet<&str>,
    local_class_rust_names: &HashSet<String>,
) -> String {
    let names = plan
        .registry
        .shared_rust_names
        .iter()
        .chain(&plan.registry.crate_root_rust_names)
        .cloned()
        .collect::<HashSet<_>>();
    relocate_project_stdlib_nominals_owned_by(
        source,
        module_name,
        crate_root_modules,
        local_class_rust_names,
        &names,
    )
}

pub(crate) fn relocate_project_stdlib_nominals_owned_by(
    source: &str,
    module_name: &str,
    crate_root_modules: &HashSet<&str>,
    local_class_rust_names: &HashSet<String>,
    owned_names: &HashSet<String>,
) -> String {
    if owned_names.is_empty() {
        return source.to_string();
    }
    let relocatable_names = owned_names
        .iter()
        .filter(|name| !local_class_rust_names.contains(*name))
        .collect::<HashSet<_>>();
    let names = relocatable_names.iter().map(|name| name.as_str()).collect();
    let stripped = strip_relocated_rust_items_by_name(source, &names, local_class_rust_names);
    if crate_root_modules.contains(module_name) {
        return stripped;
    }
    let mut ordered_names = owned_names.iter().collect::<Vec<_>>();
    ordered_names.sort();
    let mut imports = String::new();
    for name in ordered_names {
        if local_class_rust_names.contains(name) {
            continue;
        }
        if !rust_source_references_item_name(&stripped, name) {
            continue;
        }
        imports.push_str("use crate::");
        imports.push_str(name);
        imports.push_str(";\n");
    }
    format!("{imports}\n{stripped}")
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
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
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
mod tests {
    use super::*;
    use sifr_ir::{HirStmt, MethodKind};

    fn stdlib_error(identity: &str) -> Type {
        Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: "Error".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        }
    }

    #[test]
    fn same_basename_stdlib_errors_keep_distinct_nominal_declarations() {
        let mut declarations = BTreeMap::new();
        let mut builtin_types = BTreeMap::new();

        collect_shared_nominals(
            &Type::Union(vec![
                stdlib_error("sifr.csv.Error"),
                stdlib_error("sifr.configparser.Error"),
            ]),
            &mut declarations,
            &mut builtin_types,
        );

        assert_eq!(
            declarations.get("sifr.csv"),
            Some(&HashSet::from(["Error".to_string()]))
        );
        assert_eq!(
            declarations.get("sifr.configparser"),
            Some(&HashSet::from(["Error".to_string()]))
        );
        assert!(builtin_types.is_empty());
    }

    #[test]
    fn same_basename_stdlib_errors_get_distinct_project_paths() {
        let csv = stdlib_error("sifr.csv.Error");
        let config = stdlib_error("sifr.configparser.Error");
        let unions = HashMap::from([("ImportedErrors".to_string(), vec![csv, config])]);

        let plan = project_stdlib_nominal_plan(&unions, &[]);
        let csv_path = plan
            .registry
            .rust_paths
            .get("sifr.csv.Error")
            .expect("csv error should have a canonical project path");
        let config_path = plan
            .registry
            .rust_paths
            .get("sifr.configparser.Error")
            .expect("configparser error should have a canonical project path");

        assert_ne!(csv_path, config_path);
        assert!(csv_path.contains(&class_rust_name(Some("sifr.csv.Error"), "Error")));
        assert!(config_path.contains(&class_rust_name(Some("sifr.configparser.Error"), "Error")));
    }

    #[test]
    fn direct_crate_root_nominal_gets_a_project_path_without_other_nominals() {
        let native_file = Type::Class {
            identity: Some("_sifr.fs.NativeFileHandle".to_string()),
            type_args: Vec::new(),
            name: "NativeFileHandle".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let unions = HashMap::from([("NativeFile".to_string(), vec![native_file])]);

        let plan = project_stdlib_nominal_plan(&unions, &[]);

        assert_eq!(
            plan.registry.rust_paths.get("_sifr.fs.NativeFileHandle"),
            Some(&format!(
                "crate::{}",
                class_rust_name(Some("_sifr.fs.NativeFileHandle"), "NativeFileHandle")
            ))
        );
    }

    #[test]
    fn builtin_error_union_keeps_its_shared_module_definition() {
        let timeout_error = Type::Class {
            identity: Some("sifr.builtin.TimeoutError".to_string()),
            type_args: Vec::new(),
            name: "TimeoutError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let unions = HashMap::from([(
            "TimeoutOrValueError".to_string(),
            vec![
                timeout_error,
                Type::Class {
                    identity: None,
                    type_args: Vec::new(),
                    name: "ValueError".to_string(),
                    fields: vec![("message".to_string(), Type::Str)],
                    methods: Vec::new(),
                    parent_class: Some("Error".to_string()),
                },
            ],
        )]);

        let plan = project_stdlib_nominal_plan(&unions, &[]);

        assert!(
            plan.registry
                .rust_paths
                .contains_key("sifr.builtin.TimeoutError")
        );
        assert!(!plan.registry.crate_root_rust_names.contains("TimeoutError"));
    }

    #[test]
    fn task_scope_error_plan_includes_scope_failure_conversion() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: Vec::new(),
                return_type: Type::Result(
                    Box::new(Type::None),
                    Box::new(builtin_error_type("Error")),
                ),
                body: vec![HirStmt::AsyncWith {
                    kind: sifr_ir::HirAsyncWithKind::TaskScope,
                    target: Some("scope".to_string()),
                    body: Vec::new(),
                }],
                is_async: true,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: Vec::new(),
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: Vec::new(),
            }],
            classes: Vec::new(),
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };

        let generated =
            crate::generate_rust_multi_with_metadata(&[("main", &module)], &StdlibCode::default());
        assert_eq!(
            generated
                .project_union_prelude
                .matches("struct ScopeFailure")
                .count(),
            1,
            "{}",
            generated.project_union_prelude
        );
        assert!(!generated.rust_files["main"].contains("struct ScopeFailure"));
    }

    #[test]
    fn emitted_transitive_nominals_join_the_project_registry() {
        let mut stdlib = StdlibCode::default();
        stdlib.module_rust_code.insert(
            "sifr.io".to_string(),
            crate::StdlibRustSource {
                module: "sifr.io".to_string(),
                source_path: "stdlib/sifr/io.sifr".to_string(),
                source_sha256: "digest".to_string(),
                nominal_types: HashSet::from([
                    "BinaryFileHandle".to_string(),
                    "TextFileHandle".to_string(),
                ]),
                rust: String::new(),
            },
        );
        let shared_source =
            "pub struct __SifrIoBinaryFileHandle;\npub struct __SifrIoTextFileHandle;\n";
        let mut registry = ProjectNominalRegistry::default();
        registry.register_shared(
            "sifr.io.TextFileHandle".to_string(),
            "__SifrIoTextFileHandle".to_string(),
        );

        register_transitive_stdlib_nominals(shared_source, &stdlib, &mut registry);

        assert!(
            registry
                .shared_rust_names
                .contains("__SifrIoBinaryFileHandle")
        );
        assert_eq!(
            registry.rust_paths.get("sifr.io.BinaryFileHandle"),
            Some(&"crate::__sifr_project_nominals::__SifrIoBinaryFileHandle".to_string())
        );
    }

    #[test]
    fn emitted_builtin_nominals_join_the_project_registry() {
        let shared_source = "pub struct ParseError;\npub struct FileNotFoundError;\n";
        let mut registry = ProjectNominalRegistry::default();

        register_emitted_builtin_nominals(shared_source, &mut registry);

        assert!(registry.shared_rust_names.contains("ParseError"));
        assert_eq!(
            registry.rust_paths.get("sifr.builtin.ParseError"),
            Some(&"crate::__sifr_project_nominals::ParseError".to_string())
        );
        assert!(!registry.rust_paths.contains_key("ParseError"));
        assert!(!registry.shared_rust_names.contains("FileNotFoundError"));
        assert!(!registry.rust_paths.contains_key("FileNotFoundError"));
    }

    #[test]
    fn builtin_registry_identity_does_not_replace_a_module_qualified_shadow() {
        let mut paths = HashMap::from([
            (
                "shadow.ValueError".to_string(),
                "crate::shadow::ValueError".to_string(),
            ),
            (
                "ValueError".to_string(),
                "crate::shadow::ValueError".to_string(),
            ),
        ]);
        let mut registry = ProjectNominalRegistry::default();
        let builtin = BuiltinError::from_name("ValueError").expect("known builtin fixture");
        registry.register_builtin(builtin, "ValueError".to_string());

        paths.extend(registry.rust_paths);

        assert_eq!(
            paths.get("shadow.ValueError"),
            Some(&"crate::shadow::ValueError".to_string())
        );
        assert_eq!(
            paths.get("ValueError"),
            Some(&"crate::shadow::ValueError".to_string())
        );
        assert_eq!(
            paths.get("sifr.builtin.ValueError"),
            Some(&"crate::__sifr_project_nominals::ValueError".to_string())
        );

        let relocated = relocate_project_stdlib_nominals_owned_by(
            "struct ValueError { message: String, detail: String }\nfn detail(error: ValueError) -> String { error.detail }\n",
            "shadow",
            &HashSet::from(["main"]),
            &HashSet::from(["ValueError".to_string()]),
            &HashSet::from(["ValueError".to_string()]),
        );
        assert!(relocated.contains("struct ValueError"), "{relocated}");
        assert!(relocated.contains("error.detail"), "{relocated}");
        assert!(!relocated.contains("use crate::ValueError"), "{relocated}");
    }

    #[test]
    fn direct_io_error_kind_type_does_not_create_a_project_nominal() {
        let direct_kind = Type::Class {
            identity: Some("sifr.builtin.FileNotFoundError".to_string()),
            type_args: Vec::new(),
            name: "FileNotFoundError".to_string(),
            fields: vec![
                ("message".to_string(), Type::Str),
                ("kind".to_string(), Type::Str),
            ],
            methods: Vec::new(),
            parent_class: Some("IOError".to_string()),
        };
        let unions = HashMap::from([("DirectKind".to_string(), vec![direct_kind])]);

        let plan = project_stdlib_nominal_plan(&unions, &[]);

        assert!(!plan.registry.rust_paths.contains_key("FileNotFoundError"));
        assert!(
            !plan
                .registry
                .rust_paths
                .contains_key("sifr.builtin.FileNotFoundError")
        );
    }

    #[test]
    fn io_error_kind_handlers_do_not_create_dangling_project_nominals() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: Vec::new(),
                return_type: Type::None,
                body: vec![HirStmt::TryExcept {
                    body: vec![HirStmt::Pass],
                    handlers: vec![sifr_ir::HirExceptHandler {
                        error_type: Some("FileNotFoundError".to_string()),
                        error_resolved_type: None,
                        name: None,
                        body: vec![HirStmt::Pass],
                    }],
                    body_error_types: Vec::new(),
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: Vec::new(),
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: Vec::new(),
            }],
            classes: Vec::new(),
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };

        let generated =
            crate::generate_rust_multi_with_metadata(&[("main", &module)], &StdlibCode::default());

        assert!(
            !generated
                .project_union_prelude
                .contains("FileNotFoundError")
        );
        syn::parse_file(&generated.project_union_prelude)
            .expect("project nominal prelude should not contain a dangling re-export");
    }

    #[test]
    fn relocation_retains_local_child_conversion_into_shared_parent() {
        let mut plan = ProjectStdlibNominalPlan::empty();
        plan.registry.register_shared(
            "sifr.logging.Formatter".to_string(),
            "Formatter".to_string(),
        );
        let source = r#"
struct Formatter { template: String }
struct ChildFormatter { formatter: Formatter }

impl From<ChildFormatter> for Formatter {
    fn from(value: ChildFormatter) -> Self { value.formatter }
}
"#;

        let relocated = relocate_project_stdlib_nominals(
            source,
            "main",
            &plan,
            &HashSet::from(["main"]),
            &HashSet::from(["ChildFormatter".to_string()]),
        );

        assert!(!relocated.contains("struct Formatter"));
        assert!(relocated.contains("struct ChildFormatter"));
        assert!(relocated.contains("From<ChildFormatter> for Formatter"));
        syn::parse_file(&relocated).expect("retained local-child conversion should parse");
    }
}
