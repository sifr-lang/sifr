use super::{
    HashMap, HashSet, HirModule, Renderer, RustFile, RustItem, StdlibCode,
    register_imported_union_types,
};

pub(crate) struct ProjectUnionUsage {
    pub(crate) unions: HashMap<String, Vec<sifr_type_system::Type>>,
    pub(crate) module_unions: HashMap<String, HashSet<String>>,
    pub(crate) ordinary_unions: HashSet<String>,
    pub(crate) try_error_unions: HashSet<String>,
    pub(crate) structural_unions: HashSet<String>,
    pub(crate) union_name_replacements: HashMap<String, String>,
}

pub(crate) fn project_union_usage(
    modules: &[(&str, &HirModule)],
    project_code: &StdlibCode,
    structural_interop_enabled: bool,
) -> ProjectUnionUsage {
    let mut unions = HashMap::new();
    let mut module_unions = HashMap::new();
    let mut ordinary_unions = HashSet::new();
    let mut try_error_unions = HashSet::new();
    let mut structural_unions = HashSet::new();
    let mut union_name_replacements = HashMap::new();
    for (module_name, module) in modules {
        let mut emitter = super::super::RustEmitter::new();
        emitter.collect_union_types(module);
        register_imported_union_types(&mut emitter, module, project_code);
        let mut names = HashSet::new();
        for (name, mut members) in emitter.union_enums {
            for member in &mut members {
                if let sifr_type_system::Type::Class { identity, name, .. } = member
                    && identity.is_none()
                    && let Some(canonical) = crate::builtin_error_identity(name)
                {
                    *identity = Some(canonical);
                }
            }
            let canonical_name = sifr_type_system::Type::Union(members.clone()).union_enum_name();
            union_name_replacements.insert(name.clone(), canonical_name.clone());
            if emitter.ordinary_union_enums.contains(&name) {
                ordinary_unions.insert(canonical_name.clone());
            }
            if emitter.try_error_carrier_enums.contains(&name) {
                try_error_unions.insert(canonical_name.clone());
            }
            names.insert(canonical_name.clone());
            unions.entry(canonical_name).or_insert(members);
        }
        module_unions.insert((*module_name).to_string(), names);
    }
    if structural_interop_enabled {
        structural_unions =
            crate::structural_impl_codegen::structural_union_names_for_project(&unions, modules);
    }
    ProjectUnionUsage {
        unions,
        module_unions,
        ordinary_unions,
        try_error_unions,
        structural_unions,
        union_name_replacements,
    }
}

pub(crate) fn render_project_union_imports(
    module_name: &str,
    module_unions: &HashSet<String>,
    crate_root_modules: &HashSet<&str>,
) -> String {
    if crate_root_modules.contains(module_name) {
        return String::new();
    }
    let mut names = module_unions.iter().collect::<Vec<_>>();
    names.sort();
    let items = names
        .into_iter()
        .map(|name| RustItem::Use(vec!["crate".to_string(), name.clone()]))
        .collect::<Vec<_>>();
    Renderer::new().render_file(&RustFile { items })
}
