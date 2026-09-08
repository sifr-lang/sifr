use crate::stdlib::types::{StdlibRustInterop, StdlibRustInteropModuleSource};
use sifr_ir::{HirClass, HirModule};
use sifr_stdlib_manifest::{LoadedStdlibSource, LoadedStdlibSourceKind};
use sifr_sysroot::ResolvedSysroot;
use std::collections::HashMap;

pub(crate) struct PendingStdlibInteropModule<'a> {
    pub(crate) module: std::sync::Arc<HirModule>,
    pub(crate) source: &'a LoadedStdlibSource,
}

pub(crate) fn pending_private_interop_module<'a>(
    source: &'a LoadedStdlibSource,
    module: &std::sync::Arc<HirModule>,
) -> Option<PendingStdlibInteropModule<'a>> {
    if source.kind != LoadedStdlibSourceKind::PrivateDeclaration || !module_has_rust_interop(module)
    {
        return None;
    }
    Some(PendingStdlibInteropModule {
        module: std::sync::Arc::clone(module),
        source,
    })
}

pub(crate) fn build_stdlib_rust_interop(
    sysroot: Option<ResolvedSysroot>,
    modules: &[PendingStdlibInteropModule<'_>],
) -> StdlibRustInterop {
    let _b50_plan = sifr_ir::b50_phase_diagnostic::span(sifr_ir::b50_phase_diagnostic::Phase::PrivatePlan);
    if modules.is_empty() {
        return StdlibRustInterop {
            sysroot,
            ..StdlibRustInterop::default()
        };
    }

    let module_refs = modules
        .iter()
        .map(|module| (Some(module.source.module.as_str()), module.module.as_ref()));
    let module_sources = modules
        .iter()
        .map(|module| {
            (
                module.source.module.clone(),
                StdlibRustInteropModuleSource {
                    source: module.source.source.clone(),
                    display_path: module.source.path.display().to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn private_interop_pending_inventory_preserves_lifetime_and_source_order() {
        let sources = ["zeta", "alpha"].map(|name| LoadedStdlibSource {
            module: format!("_sifr.{name}"),
            source: format!("@rust(sifr_stdlib.{name})\ndef {name}() -> int: ...\n"),
            path: std::path::PathBuf::from(format!("{name}.sifr")),
            kind: LoadedStdlibSourceKind::PrivateDeclaration,
        });
        let mut pending = Vec::new();
        for source in &sources {
            let parsed = sifr_syntax::parse_module_raw(&source.source, None).unwrap();
            let lowered = sifr_lowering::lower_module_sysroot_private_declaration_with_externals(
                parsed.suite(),
                &sifr_lowering::ExternalDefs::default(),
            )
            .unwrap();
            let canonical = Arc::new(lowered.module);
            let retained = pending_private_interop_module(source, &canonical).unwrap();
            assert!(Arc::ptr_eq(&canonical, &retained.module));
            pending.push(retained);
            // Pending contracts remain valid after this handle to canonical HIR is dropped.
        }
        let interop = build_stdlib_rust_interop(None, &pending);
        drop(pending);
        assert_eq!(
            interop
                .plan
                .rust
                .declarations
                .iter()
                .map(|entry| entry.module_name.as_deref())
                .collect::<Vec<_>>(),
            [Some("_sifr.zeta"), Some("_sifr.alpha")],
        );
        assert_eq!(interop.plan.rust.bridge_contracts.signatures.len(), 2);
        for source in &sources {
            let retained = &interop.module_sources[&source.module];
            assert_eq!(retained.source, source.source);
            assert_eq!(retained.display_path, source.path.display().to_string());
        }
    }
}
