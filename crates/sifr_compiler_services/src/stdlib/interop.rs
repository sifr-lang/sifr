use crate::stdlib::types::{StdlibRustInterop, StdlibRustInteropModuleSource};
use sifr_ir::{HirClass, HirModule};
use sifr_stdlib_manifest::{LoadedStdlibSource, LoadedStdlibSourceKind};
use sifr_sysroot::ResolvedSysroot;
use std::collections::HashMap;

pub(crate) struct PendingStdlibInteropModule {
    pub(crate) module_name: String,
    pub(crate) module: HirModule,
    pub(crate) source: String,
    pub(crate) display_path: String,
}

pub(crate) fn pending_private_interop_module(
    source: &LoadedStdlibSource,
    module: &HirModule,
) -> Option<PendingStdlibInteropModule> {
    if source.kind != LoadedStdlibSourceKind::PrivateDeclaration || !module_has_rust_interop(module)
    {
        return None;
    }
    Some(PendingStdlibInteropModule {
        module_name: source.module.clone(),
        module: module.clone(),
        source: source.source.clone(),
        display_path: source.path.display().to_string(),
    })
}

pub(crate) fn build_stdlib_rust_interop(
    sysroot: Option<ResolvedSysroot>,
    modules: &[PendingStdlibInteropModule],
) -> StdlibRustInterop {
    if modules.is_empty() {
        return StdlibRustInterop {
            sysroot,
            ..StdlibRustInterop::default()
        };
    }

    let module_refs = modules
        .iter()
        .map(|module| (Some(module.module_name.as_str()), &module.module));
    let module_sources = modules
        .iter()
        .map(|module| {
            (
                module.module_name.clone(),
                StdlibRustInteropModuleSource {
                    source: module.source.clone(),
                    display_path: module.display_path.clone(),
                },
            )
        })
        .collect::<HashMap<_, _>>();

    StdlibRustInterop {
        plan: sifr_codegen::interop_build_plan_for_named_modules(module_refs),
        module_sources,
        sysroot,
    }
}

fn module_has_rust_interop(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(|function| !function.rust_interop.is_empty())
        || module.classes.iter().any(class_has_rust_interop)
}

fn class_has_rust_interop(class: &HirClass) -> bool {
    !class.rust_interop.is_empty()
        || class
            .methods
            .iter()
            .any(|method| !method.rust_interop.is_empty())
        || class
            .operator_impls
            .iter()
            .any(|(_, method)| !method.rust_interop.is_empty())
}
