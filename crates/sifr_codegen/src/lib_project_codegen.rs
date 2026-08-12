use super::{
    generate_rust, generate_rust_with_stdlib_for_module_with_project_policy,
    publicize_generated_module_source, HashMap, HashSet, HirModule, MultiModuleCodegenResult,
    Renderer, RustFile, RustItem, StdlibCode,
};
use crate::lib_project_signatures::{project_class_fields, project_func_signatures};
use sifr_stdlib_manifest::{try_generated_cargo_dependencies, StdlibFeature};
use sifr_type_system::source_class_rust_name;

pub(crate) fn register_imported_union_types(
    emitter: &mut super::RustEmitter,
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

struct ProjectUnionUsage {
    owners: HashMap<String, String>,
    module_unions: HashMap<String, HashSet<String>>,
    ordinary_unions: HashSet<String>,
    try_error_unions: HashSet<String>,
}

fn project_union_usage(
    modules: &[(&str, &HirModule)],
    project_code: &StdlibCode,
) -> ProjectUnionUsage {
    let mut owners = HashMap::new();
    let mut module_unions = HashMap::new();
    let mut ordinary_unions = HashSet::new();
    let mut try_error_unions = HashSet::new();
    for (module_name, module) in modules {
        let mut emitter = super::RustEmitter::new();
        emitter.collect_union_types(module);
        let local_unions = emitter.union_enums.keys().cloned().collect::<HashSet<_>>();
        register_imported_union_types(&mut emitter, module, project_code);
        ordinary_unions.extend(emitter.ordinary_union_enums.iter().cloned());
        try_error_unions.extend(emitter.try_error_carrier_enums.iter().cloned());
        let names = emitter.union_enums.into_keys().collect::<HashSet<_>>();
        for name in local_unions {
            owners
                .entry(name)
                .and_modify(|owner: &mut String| {
                    if *module_name < owner.as_str() {
                        *owner = (*module_name).to_string();
                    }
                })
                .or_insert_with(|| (*module_name).to_string());
        }
        module_unions.insert((*module_name).to_string(), names);
    }
    let mut external_owners = HashMap::new();
    for (module_name, names) in &module_unions {
        for name in names {
            external_owners
                .entry(name.clone())
                .and_modify(|owner: &mut String| {
                    if module_name < owner {
                        owner.clone_from(module_name);
                    }
                })
                .or_insert_with(|| module_name.clone());
        }
    }
    for (name, owner) in external_owners {
        owners.entry(name).or_insert(owner);
    }
    ProjectUnionUsage {
        owners,
        module_unions,
        ordinary_unions,
        try_error_unions,
    }
}

fn render_project_union_imports(
    module_name: &str,
    module_unions: &HashSet<String>,
    owners: &HashMap<String, String>,
) -> String {
    let mut names = module_unions.iter().collect::<Vec<_>>();
    names.sort();
    let items = names
        .into_iter()
        .filter_map(|name| {
            let owner = owners.get(name)?;
            if owner == module_name {
                return None;
            }
            let mut path = vec!["crate".to_string()];
            if owner != "main" {
                path.extend(owner.split('.').map(str::to_string));
            }
            path.push(name.clone());
            Some(RustItem::Use(path))
        })
        .collect::<Vec<_>>();
    Renderer::new().render_file(&RustFile { items })
}

fn resolve_exported_rust_opaque_class<'a>(
    module_name: &str,
    export_name: &str,
    project_modules: &HashMap<&str, &'a HirModule>,
    visiting: &mut HashSet<(String, String)>,
) -> Option<(String, &'a sifr_ir::HirClass)> {
    if !visiting.insert((module_name.to_string(), export_name.to_string())) {
        return None;
    }
    let module = project_modules.get(module_name)?;
    if let Some(class) = module.classes.iter().find(|class| {
        class.name == export_name
            && class
                .rust_interop
                .iter()
                .any(|declaration| declaration.kind == sifr_ir::RustInteropDecoratorKind::Opaque)
            && class
                .methods
                .iter()
                .any(|method| !method.rust_interop.is_empty())
    }) {
        return Some((module_name.to_string(), class));
    }
    for import in &module.imports {
        for source_name in &import.names {
            let local_name = import
                .aliases
                .iter()
                .find(|(source, _)| source == source_name)
                .map_or(source_name.as_str(), |(_, local)| local.as_str());
            if local_name == export_name {
                if let Some(resolved) = resolve_exported_rust_opaque_class(
                    &import.module,
                    source_name,
                    project_modules,
                    visiting,
                ) {
                    return Some(resolved);
                }
            }
        }
    }
    None
}

pub(super) fn render_local_module_imports(
    module: &HirModule,
    project_modules: &HashMap<&str, &HirModule>,
) -> String {
    let mut module_import_items: Vec<RustItem> = Vec::new();
    let mut imported_opaque_traits = HashSet::new();
    for import in &module.imports {
        if import.module.starts_with("sifr.") || import.module.starts_with("_sifr.") {
            continue;
        }
        let mut module_path = vec!["crate".to_string()];
        module_path.extend(import.module.split('.').map(str::to_string));
        for name in &import.names {
            if let Some((_, alias)) = import.aliases.iter().find(|(orig, _)| orig == name) {
                let mut alias_path = module_path.clone();
                alias_path.push(name.clone());
                module_import_items.push(RustItem::UseAlias {
                    path: alias_path,
                    alias: alias.clone(),
                });
            } else {
                let mut import_path = module_path.clone();
                import_path.push(name.clone());
                module_import_items.push(RustItem::Use(import_path));
            }
            if let Some((owner_module, class)) = resolve_exported_rust_opaque_class(
                &import.module,
                name,
                project_modules,
                &mut HashSet::new(),
            ) {
                let trait_name =
                    format!("__SifrOpaque{}Methods", source_class_rust_name(&class.name));
                if imported_opaque_traits.insert((owner_module.clone(), trait_name.clone())) {
                    let mut trait_path = vec!["crate".to_string()];
                    trait_path.extend(owner_module.split('.').map(str::to_string));
                    trait_path.push(trait_name);
                    module_import_items.push(RustItem::UseAlias {
                        path: trait_path,
                        alias: "_".to_string(),
                    });
                }
            }
        }
    }

    if module_import_items.is_empty() {
        String::new()
    } else {
        Renderer::new().render_file(&RustFile {
            items: module_import_items,
        })
    }
}

fn resolve_exported_generic_class<'a>(
    module_name: &str,
    export_name: &str,
    project_modules: &HashMap<&str, &'a HirModule>,
    visiting: &mut HashSet<(String, String)>,
) -> Option<&'a sifr_ir::HirClass> {
    if !visiting.insert((module_name.to_string(), export_name.to_string())) {
        return None;
    }
    let module = project_modules.get(module_name)?;
    if let Some(class) = module
        .classes
        .iter()
        .find(|class| class.name == export_name && !class.type_params.is_empty())
    {
        return Some(class);
    }
    for import in &module.imports {
        for source_name in &import.names {
            let local_name = import
                .aliases
                .iter()
                .find(|(source, _)| source == source_name)
                .map_or(source_name.as_str(), |(_, local)| local.as_str());
            if local_name == export_name {
                if let Some(class) = resolve_exported_generic_class(
                    &import.module,
                    source_name,
                    project_modules,
                    visiting,
                ) {
                    return Some(class);
                }
            }
        }
    }
    None
}

fn register_imported_generic_classes(
    code: &mut StdlibCode,
    module: &HirModule,
    project_modules: &HashMap<&str, &HirModule>,
) {
    for import in &module.imports {
        for source_name in &import.names {
            let Some(source_class) = resolve_exported_generic_class(
                &import.module,
                source_name,
                project_modules,
                &mut HashSet::new(),
            ) else {
                continue;
            };
            let local_name = import
                .aliases
                .iter()
                .find(|(original, _)| original == source_name)
                .map_or(source_name.as_str(), |(_, alias)| alias.as_str());
            let mut template = source_class.clone();
            template.name = local_name.to_string();
            code.generic_classes.insert(local_name.to_string());
            code.generic_class_params
                .insert(local_name.to_string(), source_class.type_params.clone());
            code.generic_class_templates
                .insert(local_name.to_string(), template);
        }
    }
}

/// Generate Rust source code for a multi-module project, returning aggregate dependency metadata.
pub fn generate_rust_multi_with_metadata(
    modules: &[(&str, &HirModule)],
    stdlib_code: &StdlibCode,
) -> MultiModuleCodegenResult {
    let mut files = HashMap::new();
    let mut used_stdlib_modules = HashSet::new();
    let mut required_features = HashSet::new();
    let mut project_codegen_code = stdlib_code.clone();
    let project_modules = modules.iter().copied().collect::<HashMap<_, _>>();
    let structural_interop_enabled = modules
        .iter()
        .any(|(_, module)| crate::rust_interop_plan::module_uses_structural_interop(module));
    project_codegen_code
        .func_signatures
        .extend(project_func_signatures(modules));
    project_codegen_code
        .module_class_fields
        .extend(project_class_fields(modules));
    let union_usage = project_union_usage(modules, &project_codegen_code);

    for (module_name, module) in modules {
        let module_public = *module_name != "main";
        let mut module_codegen_code = project_codegen_code.clone();
        register_imported_generic_classes(&mut module_codegen_code, module, &project_modules);
        let used_unions = union_usage
            .module_unions
            .get(*module_name)
            .cloned()
            .unwrap_or_default();
        let owned_unions = used_unions
            .iter()
            .filter(|name| {
                union_usage
                    .owners
                    .get(*name)
                    .is_some_and(|owner| owner == module_name)
            })
            .cloned()
            .collect::<HashSet<_>>();
        let codegen_result = generate_rust_with_stdlib_for_module_with_project_policy(
            module,
            &module_codegen_code,
            Some(module_name),
            structural_interop_enabled,
            Some(&owned_unions),
            Some(&union_usage.ordinary_unions),
            Some(&union_usage.try_error_unions),
        );
        let local_imports = render_local_module_imports(module, &project_modules);
        let union_imports =
            render_project_union_imports(module_name, &used_unions, &union_usage.owners);
        let mut rust_source = codegen_result.rust_source;
        let imports = [local_imports, union_imports]
            .into_iter()
            .filter(|source| !source.trim().is_empty())
            .map(|source| source.trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        if !imports.is_empty() {
            rust_source = format!("{imports}\n\n{rust_source}");
        }
        if module_public {
            rust_source = publicize_generated_module_source(&rust_source);
        }
        if rust_source.contains("::sifr_stdlib::fs::") {
            required_features.insert(StdlibFeature::Fs);
        }

        files.insert((*module_name).to_string(), rust_source);
        used_stdlib_modules.extend(codegen_result.used_stdlib_modules);
        required_features.extend(codegen_result.required_features);
    }

    MultiModuleCodegenResult {
        rust_files: files,
        used_stdlib_modules,
        required_features,
        interop: crate::rust_interop_plan::interop_build_plan_for_named_modules(
            modules.iter().map(|(name, module)| (Some(*name), *module)),
        ),
    }
}

/// Generate Rust source code for a multi-module project.
/// Returns a map of filename -> Rust source code.
pub fn generate_rust_multi(modules: &[(&str, &HirModule)]) -> HashMap<String, String> {
    generate_rust_multi_with_metadata(modules, &StdlibCode::default())
        .rust_files
        .into_iter()
        .collect()
}

/// Generate a complete Rust project (Cargo.toml + main.rs content).
pub fn generate_project(module: &HirModule, project_name: &str) -> (String, String) {
    generate_project_with_deps(module, project_name, &HashSet::new())
}

/// Generate a complete Rust project with stdlib dependencies.
pub fn generate_project_with_deps(
    module: &HirModule,
    project_name: &str,
    stdlib_modules: &HashSet<String>,
) -> (String, String) {
    generate_project_with_deps_and_crates(module, project_name, stdlib_modules, &HashSet::new())
}

/// Generate a complete Rust project with stdlib and explicit crate dependencies.
#[allow(
    clippy::expect_used,
    reason = "infallible codegen project helper has a tuple return type; driver build paths use fallible sysroot planning"
)]
pub fn generate_project_with_deps_and_crates(
    module: &HirModule,
    project_name: &str,
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> (String, String) {
    let mut cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[workspace]
"#
    );

    let deps = try_generated_cargo_dependencies(stdlib_modules, required_features)
        .expect("infallible project generation should resolve the Sifr sysroot");

    if !deps.is_empty() {
        cargo_toml.push_str("\n[dependencies]\n");
        for dep in &deps {
            cargo_toml.push_str(dep);
            cargo_toml.push('\n');
        }
    }

    let main_rs = generate_rust(module);
    (cargo_toml, main_rs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruff_text_size::TextRange;
    use sifr_ir::{
        HirClass, HirClassKind, HirExceptHandler, HirFunction, HirImport, MethodKind,
        RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
        RustInteropEffect,
    };

    fn empty_function(name: &str, return_type: sifr_type_system::Type) -> HirFunction {
        HirFunction {
            name: name.to_string(),
            params: Vec::new(),
            return_type,
            body: vec![sifr_ir::HirStmt::Return {
                value: Some(sifr_ir::HirExpr::IntLiteral(1)),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: Vec::new(),
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }
    }

    fn module_with(functions: Vec<HirFunction>, imports: Vec<HirImport>) -> HirModule {
        HirModule {
            functions,
            classes: Vec::new(),
            imports,
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        }
    }

    fn error_type(name: &str) -> sifr_type_system::Type {
        sifr_type_system::Type::Class {
            identity: Some(format!("errors.{name}")),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: vec![("message".to_string(), sifr_type_system::Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        }
    }

    #[test]
    fn project_unions_have_one_owner_and_are_imported_by_other_modules() {
        let union = sifr_type_system::Type::Union(vec![
            sifr_type_system::Type::Int,
            sifr_type_system::Type::Str,
        ]);
        let enum_name = union.union_enum_name();
        let provider = module_with(vec![empty_function("produce", union.clone())], Vec::new());
        let consumer = module_with(
            vec![empty_function("marker", sifr_type_system::Type::Int)],
            vec![HirImport {
                module: "provider".to_string(),
                names: vec!["produce".to_string()],
                aliases: Vec::new(),
            }],
        );

        let generated = generate_rust_multi_with_metadata(
            &[("main", &consumer), ("provider", &provider)],
            &StdlibCode::default(),
        );
        let provider_source = &generated.rust_files["provider"];
        let consumer_source = &generated.rust_files["main"];

        assert!(
            provider_source.contains(&format!("pub enum {enum_name}")),
            "{provider_source}"
        );
        assert_eq!(
            provider_source
                .matches(&format!("enum {enum_name}"))
                .count(),
            1
        );
        assert!(
            consumer_source.contains(&format!("use crate::provider::{enum_name};")),
            "{consumer_source}"
        );
        assert!(!consumer_source.contains(&format!("enum {enum_name}")));
    }

    #[test]
    fn main_owned_union_is_imported_from_the_crate_root() {
        let union = sifr_type_system::Type::Union(vec![
            sifr_type_system::Type::Int,
            sifr_type_system::Type::Str,
        ]);
        let enum_name = union.union_enum_name();
        let owner = module_with(vec![empty_function("produce", union.clone())], Vec::new());
        let consumer = module_with(vec![empty_function("relay", union.clone())], Vec::new());

        let generated = generate_rust_multi_with_metadata(
            &[("main", &owner), ("support", &consumer)],
            &StdlibCode::default(),
        );

        assert!(generated.rust_files["main"].contains(&format!("enum {enum_name}")));
        assert!(
            generated.rust_files["support"].contains(&format!("use crate::{enum_name};")),
            "{}",
            generated.rust_files["support"]
        );
    }

    #[test]
    fn dotted_union_owner_is_imported_through_its_module_path() {
        let union = sifr_type_system::Type::Union(vec![
            sifr_type_system::Type::Int,
            sifr_type_system::Type::Str,
        ]);
        let enum_name = union.union_enum_name();
        let owner = module_with(vec![empty_function("produce", union.clone())], Vec::new());
        let consumer = module_with(
            vec![empty_function("marker", sifr_type_system::Type::Int)],
            vec![HirImport {
                module: "pkg.errors".to_string(),
                names: vec!["produce".to_string()],
                aliases: Vec::new(),
            }],
        );

        let generated = generate_rust_multi_with_metadata(
            &[("pkg.errors", &owner), ("main", &consumer)],
            &StdlibCode::default(),
        );

        assert!(
            generated.rust_files["main"].contains(&format!("use crate::pkg::errors::{enum_name};")),
            "{}",
            generated.rust_files["main"]
        );
    }

    #[test]
    fn owner_combines_try_carrier_conversions_with_ordinary_union_traits() {
        let first = error_type("FirstError");
        let second = error_type("SecondError");
        let union = sifr_type_system::Type::Union(vec![first.clone(), second.clone()]);
        let enum_name = union.union_enum_name();
        let mut try_function = empty_function("guarded", sifr_type_system::Type::None);
        try_function.body = vec![sifr_ir::HirStmt::TryExcept {
            body: vec![sifr_ir::HirStmt::Pass],
            handlers: vec![HirExceptHandler {
                error_type: Some("FirstError".to_string()),
                error_resolved_type: Some(first.clone()),
                name: None,
                body: vec![sifr_ir::HirStmt::Pass],
            }],
            body_error_types: vec![first, second],
        }];
        let owner = module_with(vec![try_function], Vec::new());
        let consumer = module_with(vec![empty_function("ordinary", union)], Vec::new());

        let generated = generate_rust_multi_with_metadata(
            &[("errors", &owner), ("main", &consumer)],
            &StdlibCode::default(),
        );
        let owner_source = &generated.rust_files["errors"];

        assert!(
            owner_source.contains("#[derive(Debug, Clone, PartialEq, Eq, Hash)]"),
            "{owner_source}"
        );
        assert!(
            owner_source.contains("impl From<FirstError>")
                && owner_source.contains(&format!("for {enum_name}")),
            "{owner_source}"
        );
        assert!(
            owner_source.contains("impl From<SecondError>"),
            "{owner_source}"
        );
    }

    #[test]
    fn owner_election_distinguishes_non_class_nominal_identities() {
        let nominal_pairs = [
            (
                sifr_type_system::Type::Newtype {
                    identity: Some("left.Token".to_string()),
                    name: "Token".to_string(),
                    inner: Box::new(sifr_type_system::Type::Int),
                },
                sifr_type_system::Type::Newtype {
                    identity: Some("right.Token".to_string()),
                    name: "Token".to_string(),
                    inner: Box::new(sifr_type_system::Type::Int),
                },
            ),
            (
                sifr_type_system::Type::Enum {
                    identity: Some("left.Status".to_string()),
                    name: "Status".to_string(),
                    variants: vec![("READY".to_string(), Some(1))],
                },
                sifr_type_system::Type::Enum {
                    identity: Some("right.Status".to_string()),
                    name: "Status".to_string(),
                    variants: vec![("READY".to_string(), Some(1))],
                },
            ),
            (
                sifr_type_system::Type::Protocol {
                    identity: Some("left.Readable".to_string()),
                    name: "Readable".to_string(),
                    methods: Vec::new(),
                },
                sifr_type_system::Type::Protocol {
                    identity: Some("right.Readable".to_string()),
                    name: "Readable".to_string(),
                    methods: Vec::new(),
                },
            ),
        ];

        for (left, right) in nominal_pairs {
            let left = module_with(
                vec![empty_function(
                    "left",
                    sifr_type_system::Type::Union(vec![sifr_type_system::Type::Int, left]),
                )],
                Vec::new(),
            );
            let right = module_with(
                vec![empty_function(
                    "right",
                    sifr_type_system::Type::Union(vec![sifr_type_system::Type::Int, right]),
                )],
                Vec::new(),
            );
            let usage = project_union_usage(
                &[("left", &left), ("right", &right)],
                &StdlibCode::default(),
            );

            assert_eq!(usage.owners.len(), 2, "{:?}", usage.owners);
            assert!(usage.owners.values().any(|owner| owner == "left"));
            assert!(usage.owners.values().any(|owner| owner == "right"));
        }
    }

    #[test]
    fn local_imports_bring_opaque_extension_traits_into_scope() {
        let declaration = |kind| RustInteropDeclaration {
            kind,
            target: None,
            arguments: Vec::new(),
            span: TextRange::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
            consumes_receiver: false,
        };
        let provider = HirModule {
            functions: Vec::new(),
            classes: vec![HirClass {
                name: "Resource".to_string(),
                identity: None,
                fields: Vec::new(),
                field_defaults: Vec::new(),
                declaration_metadata: Vec::new(),
                methods: vec![HirFunction {
                    name: "close".to_string(),
                    params: Vec::new(),
                    return_type: sifr_type_system::Type::None,
                    body: Vec::new(),
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    receiver: None,
                    decorators: Vec::new(),
                    rust_interop: vec![declaration(RustInteropDecoratorKind::Function)],
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: Vec::new(),
                }],
                is_hashable: false,
                is_error_type: false,
                kind: HirClassKind::Regular,
                operator_impls: Vec::new(),
                newtype_inner: None,
                implements_protocols: Vec::new(),
                parent_class: None,
                parent_type: None,
                type_params: Vec::new(),
                enum_variants: Vec::new(),
                rust_interop: vec![declaration(RustInteropDecoratorKind::Opaque)],
            }],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let consumer = HirModule {
            functions: Vec::new(),
            classes: Vec::new(),
            imports: vec![HirImport {
                module: "resources".to_string(),
                names: vec!["Resource".to_string()],
                aliases: Vec::new(),
            }],
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let modules = HashMap::from([("resources", &provider), ("main", &consumer)]);

        let imports = render_local_module_imports(&consumer, &modules);

        assert!(
            imports.contains("use crate::resources::Resource;"),
            "{imports}"
        );
        assert!(
            imports.contains("use crate::resources::__SifrOpaqueResourceMethods as _;"),
            "{imports}"
        );
    }
}
