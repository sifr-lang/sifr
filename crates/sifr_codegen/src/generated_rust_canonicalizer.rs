//! Codegen consumes the generated-Rust syntax authority.
pub use sifr_generated_rust::{
    canonicalize_generated_rust_identifier, canonicalize_generated_rust_project,
    canonicalize_generated_rust_project_with_names, canonicalize_generated_rust_source,
    discover_project_const_function_names, finalize_formatted_generated_rust_source,
    finalize_formatted_generated_rust_source_with_project_consts,
};
pub(crate) use sifr_generated_rust::{
    import_generated_support_in_project_nominals, import_project_prelude_bindings,
    import_root_bindings_in_project_nominals, rewrite_named_project_borrows,
};

#[cfg(test)]
mod member_demand;

pub(crate) fn prune_generated_project_owners(
    prelude: &str,
    support: &str,
    consumers: &[&str],
) -> Result<(String, String), String> {
    sifr_generated_rust::prune_generated_project_owners(
        prelude,
        support,
        consumers,
        crate::BUILTIN_ERROR_CLASSES,
    )
}
