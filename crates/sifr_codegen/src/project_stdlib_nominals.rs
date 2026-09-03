use crate::stdlib_filter::{
    partition_rust_items_by_name, rust_source_defined_item_names, rust_source_references_item_name,
    strip_relocated_rust_items_by_name, strip_rust_items_by_name,
};
use crate::{
    HirModule, Renderer, RustFile, StdlibCode, build_error_into_error_impl,
    generate_rust_with_stdlib_for_module_with_structural_policy, publicize_generated_module_source,
};
use sifr_ir::{HirExpr, HirFunction, HirImport, HirStmt, MethodKind};
use sifr_stdlib_manifest::StdlibFeature;
use sifr_type_system::{FunctionType, Type, class_rust_name, is_crate_root_rust_nominal_identity};
use std::collections::{BTreeMap, HashMap, HashSet};

mod project_union_relocation;

use project_union_relocation::{relocate_project_unions, shared_nominal_reexport_names};

const SHARED_STDLIB_NOMINAL_MODULE: &str = "__sifr_project_nominals";
const SHARED_MODULE_CRATE_ROOT_NOMINAL_IDENTITIES: &[&str] = &["_sifr.fs.NativeFileHandle"];

pub(crate) struct ProjectStdlibNominalPlan {
    pub(crate) prelude: String,
    pub(crate) registry: ProjectNominalRegistry,
    pub(crate) used_stdlib_modules: HashSet<String>,
    pub(crate) required_features: HashSet<StdlibFeature>,
}

#[derive(Default)]
pub(crate) struct ProjectNominalRegistry {
    pub(crate) shared_rust_names: HashSet<String>,
    pub(crate) crate_root_rust_names: HashSet<String>,
    pub(crate) rust_paths: HashMap<String, String>,
}

impl ProjectNominalRegistry {
    fn register_shared(&mut self, identity: String, rust_name: String) {
        self.rust_paths.insert(
            identity,
            format!("crate::{SHARED_STDLIB_NOMINAL_MODULE}::{rust_name}"),
        );
        self.shared_rust_names.insert(rust_name);
    }

    fn register_crate_root(&mut self, identity: String, rust_name: String) {
        self.rust_paths
            .insert(identity, format!("crate::{rust_name}"));
        self.shared_rust_names.remove(&rust_name);
        self.crate_root_rust_names.insert(rust_name);
    }
}

impl ProjectStdlibNominalPlan {
    pub(crate) fn empty() -> Self {
        Self {
            prelude: String::new(),
            registry: ProjectNominalRegistry::default(),
            used_stdlib_modules: HashSet::new(),
            required_features: HashSet::new(),
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
    if plan.registry.shared_rust_names.is_empty() && plan.registry.crate_root_rust_names.is_empty()
    {
        return source.to_string();
    }
    let names = plan
        .registry
        .shared_rust_names
        .iter()
        .chain(&plan.registry.crate_root_rust_names)
        .map(String::as_str)
        .collect();
    let stripped = strip_relocated_rust_items_by_name(source, &names, local_class_rust_names);
    if crate_root_modules.contains(module_name) {
        return stripped;
    }
    let mut ordered_names = plan
        .registry
        .shared_rust_names
        .iter()
        .chain(&plan.registry.crate_root_rust_names)
        .collect::<Vec<_>>();
    ordered_names.sort();
    let mut imports = String::new();
    for name in ordered_names {
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
    stdlib_code: &StdlibCode,
    modules: &[(&str, &HirModule)],
    structural_interop_enabled: bool,
) -> ProjectStdlibNominalPlan {
    let mut declarations = BTreeMap::<String, HashSet<String>>::new();
    let mut builtin_types = BTreeMap::<String, Type>::new();
    for members in unions.values() {
        for member in members {
            collect_shared_nominals(member, &mut declarations, &mut builtin_types);
        }
    }
    for (_, module) in modules {
        collect_module_nominals(module, &mut declarations, &mut builtin_types);
        let intrinsic_functions =
            crate::error_refs::collect_module_intrinsic_function_names(module);
        for name in crate::error_refs::collect_complete_referenced_builtin_error_classes(
            module,
            "",
            &intrinsic_functions,
            false,
            crate::BUILTIN_ERROR_CLASSES,
        ) {
            if sifr_type_system::io_error_kind(&name).is_some() {
                continue;
            }
            builtin_types
                .entry(name.clone())
                .or_insert_with(|| builtin_error_type(&name));
        }
    }
    let mut python_error_rust_names = HashSet::new();
    if builtin_types.contains_key("Error") {
        for (_, module) in modules {
            if !crate::python_interop_common::module_uses_async_python_declaration(module) {
                continue;
            }
            for (rust_name, ty) in crate::python_interop_common::python_error_contract_types(module)
            {
                collect_shared_nominals(&ty, &mut declarations, &mut builtin_types);
                python_error_rust_names.insert(rust_name);
            }
        }
    }
    if declarations.is_empty() && builtin_types.is_empty() {
        return ProjectStdlibNominalPlan::empty();
    }

    let mut imports = Vec::new();
    let mut registry = ProjectNominalRegistry::default();
    for (module, names) in declarations {
        let mut names = names.into_iter().collect::<Vec<_>>();
        names.sort();
        for name in &names {
            let identity = format!("{module}.{name}");
            let rust_name = class_rust_name(Some(&identity), name);
            registry.register_shared(identity, rust_name);
        }
        imports.push(HirImport {
            module,
            names,
            aliases: Vec::new(),
        });
    }
    let mut probe_names = HashSet::new();
    let mut functions = Vec::new();
    for (index, (name, ty)) in builtin_types.into_iter().enumerate() {
        let probe_name = format!("__sifr_project_builtin_nominal_{index}");
        probe_names.insert(probe_name.clone());
        let rust_name = class_rust_name(None, &name);
        if let Type::Class {
            identity: Some(identity),
            ..
        } = &ty
        {
            registry.register_shared(identity.clone(), rust_name.clone());
        }
        registry.register_shared(name, rust_name);
        functions.push(HirFunction {
            name: probe_name,
            params: Vec::new(),
            return_type: ty.clone(),
            body: vec![HirStmt::Raise {
                value: HirExpr::Name {
                    name: "__sifr_project_builtin_probe_value".to_string(),
                    binding_id: None,
                    ty,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: Vec::new(),
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        });
    }
    let synthetic_module = HirModule {
        functions,
        classes: Vec::new(),
        imports,
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };
    let generated = generate_rust_with_stdlib_for_module_with_structural_policy(
        &synthetic_module,
        stdlib_code,
        Some("main"),
        structural_interop_enabled,
    );
    let probe_name_refs = probe_names.iter().map(String::as_str).collect();
    let shared_source = strip_rust_items_by_name(&generated.rust_source, &probe_name_refs);
    register_emitted_builtin_nominals(&shared_source, &mut registry);
    register_transitive_stdlib_nominals(&shared_source, stdlib_code, &mut registry);
    let (shared_source, relocated_project_unions) =
        relocate_project_unions(&shared_source, unions, &mut registry);
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
            registry.register_crate_root((*identity).to_string(), rust_name);
        }
    }
    let crate_root_source = publicize_generated_module_source(&crate_root_source);
    let mut shared_source = publicize_generated_module_source(&shared_source);
    if !python_error_rust_names.is_empty() {
        let mut names = python_error_rust_names.into_iter().collect::<Vec<_>>();
        names.sort();
        let items = names
            .iter()
            .map(|name| build_error_into_error_impl(name))
            .collect();
        shared_source.push_str(&Renderer::new().render_file(&RustFile { items }));
    }
    let mut prelude = crate_root_source;
    if !prelude.is_empty() {
        prelude.push('\n');
    }
    prelude.push_str("mod ");
    prelude.push_str(SHARED_STDLIB_NOMINAL_MODULE);
    prelude.push_str(" {\n");
    let mut crate_root_imports = registry
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
    for line in shared_source.lines() {
        prelude.push_str("    ");
        prelude.push_str(line);
        prelude.push('\n');
    }
    prelude.push_str("}\n");
    let ordered_names = shared_nominal_reexport_names(&registry, &relocated_project_unions);
    for name in ordered_names {
        prelude.push_str("pub use ");
        prelude.push_str(SHARED_STDLIB_NOMINAL_MODULE);
        prelude.push_str("::");
        prelude.push_str(&name);
        prelude.push_str(";\n");
    }

    ProjectStdlibNominalPlan {
        prelude,
        registry,
        used_stdlib_modules: generated.used_stdlib_modules,
        required_features: generated.required_features,
    }
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
    for name in crate::BUILTIN_ERROR_CLASSES {
        if sifr_type_system::io_error_kind(name).is_some() || !defined_names.contains(*name) {
            continue;
        }
        registry.register_shared((*name).to_string(), class_rust_name(None, name));
    }
}

fn collect_function_nominals(
    function: &FunctionType,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<String, Type>,
) {
    for (_, parameter, _) in &function.params {
        collect_shared_nominals(parameter, declarations, builtin_types);
    }
    collect_shared_nominals(&function.return_type, declarations, builtin_types);
}

fn collect_hir_function_nominals(
    function: &HirFunction,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<String, Type>,
) {
    for parameter in &function.params {
        collect_shared_nominals(&parameter.ty, declarations, builtin_types);
    }
    collect_shared_nominals(&function.return_type, declarations, builtin_types);
    for ty in crate::hir_analysis::queries::collect_let_declared_types(&function.body) {
        collect_shared_nominals(&ty, declarations, builtin_types);
    }
}

fn collect_module_nominals(
    module: &HirModule,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<String, Type>,
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
    builtin_types: &mut BTreeMap<String, Type>,
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
            if is_compiler_builtin_error(identity.as_deref(), name)
                && sifr_type_system::io_error_kind(name).is_none()
            {
                builtin_types
                    .entry(name.clone())
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
    crate::BUILTIN_ERROR_CLASSES.contains(&name)
        && identity.is_none_or(|identity| {
            identity.starts_with("sifr.builtin.")
                || sifr_type_system::is_global_rust_nominal_identity(identity)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut stdlib = StdlibCode::default();
        for module in ["sifr.csv", "sifr.configparser"] {
            let identity = format!("{module}.Error");
            let rust_name = class_rust_name(Some(&identity), "Error");
            stdlib.module_rust_code.insert(
                module.to_string(),
                crate::StdlibRustSource {
                    module: module.to_string(),
                    source_path: format!("stdlib/{}.sifr", module.replace('.', "/")),
                    source_sha256: "fixture".to_string(),
                    nominal_types: HashSet::from(["Error".to_string()]),
                    rust: format!(
                        "#[derive(Debug, Clone, PartialEq, Eq, Hash)]\npub struct {rust_name} {{ pub message: String }}\n"
                    ),
                },
            );
        }

        let plan = project_stdlib_nominal_plan(&unions, &stdlib, &[], false);
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

        let plan = project_stdlib_nominal_plan(&unions, &StdlibCode::default(), &[], false);

        assert!(plan.prelude.contains("pub struct TimeoutError"));
        assert!(
            plan.prelude
                .contains("pub use __sifr_project_nominals::TimeoutError")
        );
        assert!(!plan.prelude.contains("use crate::TimeoutError"));
        assert!(plan.registry.crate_root_rust_names.is_empty());
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

        let plan = project_stdlib_nominal_plan(
            &HashMap::new(),
            &StdlibCode::default(),
            &[("main", &module)],
            false,
        );

        assert!(plan.prelude.contains("impl From<ScopeFailure> for Error"));
        assert!(
            plan.prelude
                .contains("pub use __sifr_project_nominals::ScopeFailure")
        );
        assert_eq!(
            plan.registry.rust_paths.get("ScopeFailure"),
            Some(&"crate::__sifr_project_nominals::ScopeFailure".to_string())
        );

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
            registry.rust_paths.get("ParseError"),
            Some(&"crate::__sifr_project_nominals::ParseError".to_string())
        );
        assert!(!registry.shared_rust_names.contains("FileNotFoundError"));
        assert!(!registry.rust_paths.contains_key("FileNotFoundError"));
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

        let plan = project_stdlib_nominal_plan(&unions, &StdlibCode::default(), &[], false);

        assert!(!plan.prelude.contains("FileNotFoundError"));
        assert!(!plan.registry.rust_paths.contains_key("FileNotFoundError"));
        assert!(
            !plan
                .registry
                .rust_paths
                .contains_key("sifr.builtin.FileNotFoundError")
        );
        syn::parse_file(&plan.prelude)
            .expect("project nominal prelude should not contain a dangling re-export");
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
