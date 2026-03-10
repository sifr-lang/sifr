mod api;
mod cargo_manifest;
mod entrypoint;
mod materialize;
mod project_codegen;
mod workspace;

pub use api::{build, build_project, check_project};

pub(crate) use cargo_manifest::generate_dependency_cargo_toml;
pub(crate) use entrypoint::{
    build_rooted_entrypoint_binary, compile_single_file_entrypoint_with_metadata,
    compile_single_file_frontend, resolve_project_entrypoint_plan, RootedEntrypoint,
};
pub(crate) use workspace::{create_invocation_workspace, InvocationWorkspaceGuard};
