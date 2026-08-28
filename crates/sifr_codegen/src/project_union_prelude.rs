use crate::ir_imports::collect_import_needs_from_items;
use crate::lib_project_codegen::ProjectUnionUsage;
use crate::{HashMap, Renderer, RustFile, RustItem, publicize_generated_module_source};
use sifr_type_system::Type;
use std::fmt::Write;

const PROJECT_UNION_MODULE: &str = "__sifr_project_unions";

pub(crate) fn render_project_union_prelude(
    usage: &ProjectUnionUsage,
    nominal_type_paths: &HashMap<String, String>,
) -> crate::CodegenOutcome<String> {
    if usage.unions.is_empty() {
        return Ok(String::new());
    }
    validate_project_union_nominal_paths(usage, nominal_type_paths)?;
    let mut emitter = crate::RustEmitter::new();
    emitter.union_enums.clone_from(&usage.unions);
    emitter
        .ordinary_union_enums
        .clone_from(&usage.ordinary_unions);
    emitter
        .try_error_carrier_enums
        .clone_from(&usage.try_error_unions);
    emitter
        .structural_union_enums
        .clone_from(&usage.structural_unions);
    emitter
        .project_nominal_type_paths
        .clone_from(nominal_type_paths);
    emitter.generate_enum_definitions();

    let import_needs = collect_import_needs_from_items(&emitter.enum_items);
    let mut imports = Vec::new();
    let mut add_import = |needed: bool, path: &[&str]| {
        if needed {
            imports.push(RustItem::Use(
                path.iter().map(|part| (*part).to_string()).collect(),
            ));
        }
    };
    add_import(
        import_needs.collections.needs_hashmap,
        &["std", "collections", "HashMap"],
    );
    add_import(
        import_needs.collections.needs_hashset,
        &["std", "collections", "HashSet"],
    );
    add_import(
        import_needs.collections.needs_vecdeque,
        &["std", "collections", "VecDeque"],
    );
    add_import(
        import_needs.runtime.numeric.needs_bigint,
        &["num_bigint", "BigInt"],
    );
    add_import(
        import_needs.runtime.numeric.needs_decimal,
        &["rust_decimal", "Decimal"],
    );
    add_import(
        import_needs.runtime.numeric.needs_bigdecimal,
        &["bigdecimal", "BigDecimal"],
    );
    add_import(
        import_needs.runtime.needs_sifr_int,
        &["", "sifr_runtime", "SifrInt"],
    );
    add_import(import_needs.runtime.needs_mutex, &["std", "sync", "Mutex"]);

    let import_source = Renderer::new().render_file(&RustFile { items: imports });
    let enum_source = publicize_generated_module_source(&Renderer::new().render_file(&RustFile {
        items: emitter.enum_items,
    }))?;
    let mut prelude = format!("mod {PROJECT_UNION_MODULE} {{\n");
    for line in import_source.lines().chain(enum_source.lines()) {
        prelude.push_str("    ");
        prelude.push_str(line);
        prelude.push('\n');
    }
    prelude.push_str("}\n");
    let mut names = usage.unions.keys().collect::<Vec<_>>();
    names.sort();
    for name in names {
        let _ = writeln!(prelude, "pub use {PROJECT_UNION_MODULE}::{name};");
    }
    Ok(prelude)
}

fn validate_project_union_nominal_paths(
    usage: &ProjectUnionUsage,
    nominal_type_paths: &HashMap<String, String>,
) -> crate::CodegenOutcome<()> {
    if nominal_type_paths.is_empty() {
        return Ok(());
    }
    for member in usage.unions.values().flatten() {
        validate_member_nominal_path(member, nominal_type_paths)?;
    }
    Ok(())
}

fn validate_member_nominal_path(
    ty: &Type,
    nominal_type_paths: &HashMap<String, String>,
) -> crate::CodegenOutcome<()> {
    let resolved = crate::resolve_alias_type_for_plain_call(ty);
    if let Some(member) = resolved.optional_member_type() {
        return validate_member_nominal_path(&member, nominal_type_paths);
    }
    match resolved {
        class @ Type::Class {
            identity,
            type_args,
            name,
            ..
        } => {
            if !class.is_python_object_contract() && !class.is_python_resource_identity_contract() {
                validate_nominal_path(identity.as_deref(), name, nominal_type_paths)?;
            }
            for type_arg in type_args {
                validate_member_nominal_path(type_arg, nominal_type_paths)?;
            }
        }
        Type::Protocol { identity, name, .. }
        | Type::Newtype { identity, name, .. }
        | Type::Enum { identity, name, .. } => {
            validate_nominal_path(identity.as_deref(), name, nominal_type_paths)?;
        }
        Type::List(inner) | Type::Iterable(inner) | Type::Set(inner) => {
            validate_member_nominal_path(inner, nominal_type_paths)?;
        }
        Type::Dict(key, value) | Type::Result(key, value) => {
            validate_member_nominal_path(key, nominal_type_paths)?;
            validate_member_nominal_path(value, nominal_type_paths)?;
        }
        Type::Tuple(items) => {
            for item in items {
                validate_member_nominal_path(item, nominal_type_paths)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_nominal_path(
    identity: Option<&str>,
    name: &str,
    nominal_type_paths: &HashMap<String, String>,
) -> crate::CodegenOutcome<()> {
    let key = identity.unwrap_or(name);
    if nominal_type_paths.contains_key(key)
        || identity.is_some_and(sifr_type_system::is_crate_root_rust_nominal_identity)
    {
        return Ok(());
    }
    Err(crate::CodegenError::new(format!(
        "project union nominal identity `{key}` has no registered Rust path"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn missing_project_union_nominal_path_is_a_structured_error() {
        let member = Type::Class {
            identity: Some("models.Missing".to_string()),
            type_args: vec![],
            name: "Missing".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let usage = ProjectUnionUsage {
            unions: HashMap::from([("MissingOrInt".to_string(), vec![member, Type::Int])]),
            module_unions: HashMap::new(),
            ordinary_unions: HashSet::new(),
            try_error_unions: HashSet::new(),
            structural_unions: HashSet::new(),
        };
        let registered =
            HashMap::from([("other.Model".to_string(), "crate::other::Model".to_string())]);

        let error = render_project_union_prelude(&usage, &registered)
            .expect_err("missing project nominal path must fail before rendering");

        assert!(error.message.contains("models.Missing"));
    }
}
