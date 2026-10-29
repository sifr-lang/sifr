//! Codegen consumes the generated-Rust syntax authority.
pub(crate) use sifr_generated_rust::*;

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
