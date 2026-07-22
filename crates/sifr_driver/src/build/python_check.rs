use super::project_codegen::GeneratedBinaryProject;
use super::report::{
    PythonDeclarationCheck, PythonEnvironmentCheck, PythonInteropCheckReport, PythonTargetCheck,
    PythonTargetCheckStatus,
};

pub(super) fn python_interop_check_report(
    generated: &GeneratedBinaryProject,
) -> PythonInteropCheckReport {
    let python = &generated.interop.python;
    let declarations = python
        .declarations
        .iter()
        .map(|declaration| PythonDeclarationCheck {
            module_name: declaration.module_name.clone(),
            function_name: declaration.function_name.clone(),
            target: declaration
                .declaration
                .target
                .as_ref()
                .map(sifr_ir::PythonTargetPath::dotted),
            kind: python_decorator_kind_name(declaration.declaration.kind).to_string(),
        })
        .collect();
    let target_probes = python
        .target_probes
        .iter()
        .map(|probe| PythonTargetCheck {
            target: probe.target_path.clone(),
            status: match probe.status {
                sifr_codegen::PythonTargetProbeStatus::Planned => PythonTargetCheckStatus::Deferred,
                sifr_codegen::PythonTargetProbeStatus::Verified => {
                    PythonTargetCheckStatus::Verified
                }
                sifr_codegen::PythonTargetProbeStatus::RuntimeChecked => {
                    PythonTargetCheckStatus::RuntimeChecked
                }
            },
        })
        .collect();
    let environment = generated.python_runtime.as_ref().map_or_else(
        || {
            if python.declarations.is_empty() {
                PythonEnvironmentCheck::NotRequired
            } else {
                PythonEnvironmentCheck::Deferred
            }
        },
        |runtime| PythonEnvironmentCheck::Resolved {
            interpreter: runtime.interpreter().to_path_buf(),
            digest: runtime.environment_digest().to_string(),
        },
    );
    PythonInteropCheckReport {
        declarations,
        required_import_roots: python.required_import_roots.clone(),
        target_probes,
        bridge_package_count: python.bridge_packages.len(),
        requires_async_loop: python.requires_async_loop,
        environment,
    }
}

const fn python_decorator_kind_name(kind: sifr_ir::PythonInteropDecoratorKind) -> &'static str {
    use sifr_ir::PythonInteropDecoratorKind;
    match kind {
        PythonInteropDecoratorKind::Function => "function",
        PythonInteropDecoratorKind::Coroutine => "coroutine",
        PythonInteropDecoratorKind::Opaque => "opaque",
        PythonInteropDecoratorKind::Attribute => "attribute",
        PythonInteropDecoratorKind::Item => "item",
        PythonInteropDecoratorKind::ContextEnter => "context-enter",
        PythonInteropDecoratorKind::ContextExit => "context-exit",
        PythonInteropDecoratorKind::ContextAsyncEnter => "async-context-enter",
        PythonInteropDecoratorKind::ContextAsyncExit => "async-context-exit",
        PythonInteropDecoratorKind::Callback => "callback",
        PythonInteropDecoratorKind::Buffer => "buffer",
        PythonInteropDecoratorKind::Arrow => "arrow",
        PythonInteropDecoratorKind::Dlpack => "dlpack",
        PythonInteropDecoratorKind::DlpackStream => "dlpack-stream",
    }
}
