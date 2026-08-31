use sifr_ir::{HirFunction, HirModule, HirStmt};
use sifr_type_system::Type;

use crate::{RustItem, build_generator_runtime_items};

fn function_uses_sync_generator_runtime(function: &HirFunction) -> bool {
    if matches!(function.return_type.resolve_alias(), Type::Iterator(_))
        && crate::hir_analysis::queries::body_contains_yield(&function.body)
    {
        return true;
    }
    function.body.iter().any(|statement| match statement {
        HirStmt::NestedFunction { func, .. } => function_uses_sync_generator_runtime(func),
        _ => false,
    })
}

fn module_uses_sync_generator_runtime(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_sync_generator_runtime)
        || module.classes.iter().any(|class| {
            class
                .methods
                .iter()
                .any(function_uses_sync_generator_runtime)
        })
}

pub(crate) fn build_generator_runtime_items_for_module(
    module: &HirModule,
    stdlib_preamble: &str,
) -> Vec<RustItem> {
    let stdlib_has_common = stdlib_preamble.contains("struct __SifrYielder<");
    let stdlib_has_async = stdlib_preamble.contains("struct AsyncGenerator<");
    let stdlib_has_sync = stdlib_preamble.contains("struct __SifrGenerator<");
    let needs_async = crate::module_uses_async_generator_type(module) && !stdlib_has_async;
    let needs_sync = module_uses_sync_generator_runtime(module) && !stdlib_has_sync;
    let needs_common = (needs_async || needs_sync) && !stdlib_has_common;
    build_generator_runtime_items(needs_common, needs_sync, needs_async)
}
