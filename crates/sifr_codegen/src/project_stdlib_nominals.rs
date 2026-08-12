use crate::stdlib_filter::strip_rust_items_by_name;
use crate::{
    build_error_into_error_impl, generate_rust_with_stdlib_for_module_with_structural_policy,
    publicize_generated_module_source, HirModule, Renderer, RustFile, StdlibCode,
};
use sifr_ir::{HirExpr, HirFunction, HirImport, HirStmt, MethodKind};
use sifr_stdlib_manifest::StdlibFeature;
use sifr_type_system::{class_rust_name, is_global_rust_nominal_identity, FunctionType, Type};
use std::collections::{BTreeMap, HashMap, HashSet};

const SHARED_STDLIB_NOMINAL_MODULE: &str = "__sifr_project_nominals";

pub(crate) struct ProjectStdlibNominalPlan {
    pub(crate) prelude: String,
    pub(crate) rust_names: HashSet<String>,
    pub(crate) nominal_paths: HashMap<String, String>,
    pub(crate) used_stdlib_modules: HashSet<String>,
    pub(crate) required_features: HashSet<StdlibFeature>,
}

impl ProjectStdlibNominalPlan {
    pub(crate) fn empty() -> Self {
        Self {
            prelude: String::new(),
            rust_names: HashSet::new(),
            nominal_paths: HashMap::new(),
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
) -> String {
    if plan.rust_names.is_empty() {
        return source.to_string();
    }
    let names = plan.rust_names.iter().map(String::as_str).collect();
    let stripped = strip_rust_items_by_name(source, &names);
    if crate_root_modules.contains(module_name) {
        return stripped;
    }
    let mut ordered_names = plan.rust_names.iter().collect::<Vec<_>>();
    ordered_names.sort();
    let mut imports = String::new();
    for name in ordered_names {
        if !stripped.contains(name) {
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
) -> ProjectStdlibNominalPlan {
    let mut declarations = BTreeMap::<String, HashSet<String>>::new();
    let mut builtin_types = BTreeMap::<String, Type>::new();
    for members in unions.values() {
        for member in members {
            collect_shared_nominals(member, &mut declarations, &mut builtin_types);
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
    let mut rust_names = HashSet::new();
    let mut nominal_paths = HashMap::new();
    for (module, names) in declarations {
        let mut names = names.into_iter().collect::<Vec<_>>();
        names.sort();
        for name in &names {
            let identity = format!("{module}.{name}");
            let rust_name = class_rust_name(Some(&identity), name);
            rust_names.insert(rust_name.clone());
            nominal_paths.insert(identity, format!("crate::{rust_name}"));
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
        rust_names.insert(rust_name.clone());
        let rust_path = format!("crate::{rust_name}");
        if let Type::Class {
            identity: Some(identity),
            ..
        } = &ty
        {
            nominal_paths.insert(identity.clone(), rust_path.clone());
        }
        nominal_paths.insert(name, rust_path);
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
        false,
    );
    let probe_name_refs = probe_names.iter().map(String::as_str).collect();
    let shared_source = strip_rust_items_by_name(&generated.rust_source, &probe_name_refs);
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
    let mut prelude = format!("mod {SHARED_STDLIB_NOMINAL_MODULE} {{\n");
    for line in shared_source.lines() {
        prelude.push_str("    ");
        prelude.push_str(line);
        prelude.push('\n');
    }
    prelude.push_str("}\n");
    let mut ordered_names = rust_names.iter().collect::<Vec<_>>();
    ordered_names.sort();
    for name in ordered_names {
        prelude.push_str("pub use ");
        prelude.push_str(SHARED_STDLIB_NOMINAL_MODULE);
        prelude.push_str("::");
        prelude.push_str(name);
        prelude.push_str(";\n");
    }

    ProjectStdlibNominalPlan {
        prelude,
        rust_names,
        nominal_paths,
        used_stdlib_modules: generated.used_stdlib_modules,
        required_features: generated.required_features,
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
            ..
        } => {
            if crate::BUILTIN_ERROR_CLASSES.contains(&name.as_str()) {
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
    if is_global_rust_nominal_identity(identity)
        || (!identity.starts_with("sifr.") && !identity.starts_with("_sifr."))
    {
        return;
    }
    let Some((module, name)) = identity.rsplit_once('.') else {
        return;
    };
    if crate::BUILTIN_ERROR_CLASSES.contains(&name) {
        return;
    }
    declarations
        .entry(module.to_string())
        .or_default()
        .insert(name.to_string());
}
