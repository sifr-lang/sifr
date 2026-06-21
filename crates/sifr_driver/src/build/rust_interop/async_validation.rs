use super::RustInteropResolver;
use super::{canonical_sifr_target_path, opaque_contract::OpaqueThreadAffinity, RustInteropOwner};
use crate::build::rust_interop_probe::AsyncThreadAffinity;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropEffect, RustInteropValue};

impl<'a> RustInteropResolver<'a> {
    pub(super) fn collect_async_contracts(
        &mut self,
        declarations: &[sifr_codegen::RustInteropPlanDeclaration],
    ) {
        self.async_contracts = declarations
            .iter()
            .filter(|declaration| declaration.declaration.kind == RustInteropDecoratorKind::Async)
            .filter_map(|declaration| {
                explicit_async_thread_affinity(declaration)
                    .map(|affinity| (canonical_sifr_target_path(declaration), affinity))
            })
            .collect();
    }

    pub(super) fn validate_async_declaration(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
    ) -> bool {
        let mut valid = true;
        if declaration.declaration.abi_requirements.async_boundary {
            match declaration.declaration.effect {
                RustInteropEffect::BlockingIo | RustInteropEffect::CpuHeavy => {
                    self.push_async_diagnostic(
                        declaration,
                        declaration.declaration.span,
                        "Rust async interop cannot be combined with blocking or CPU-heavy classification",
                    );
                    valid = false;
                }
                RustInteropEffect::Sync | RustInteropEffect::Async => {}
            }
        }

        if declaration.declaration.kind != RustInteropDecoratorKind::Async {
            return valid;
        }

        for argument in &declaration.declaration.arguments {
            let Some(name) = argument.name.as_deref() else {
                self.push_async_diagnostic(
                    declaration,
                    argument.span,
                    "`@rust.async(...)` requires named arguments",
                );
                valid = false;
                continue;
            };
            match name {
                "thread_affinity" => match &argument.value {
                    RustInteropValue::Symbol(symbol)
                        if symbol == "none" || symbol == "tokio_current_thread" => {}
                    _ => {
                        self.push_async_diagnostic(
                            declaration,
                            argument.span,
                            "`thread_affinity=` must be none or tokio_current_thread",
                        );
                        valid = false;
                    }
                },
                other => {
                    self.push_async_diagnostic(
                        declaration,
                        argument.span,
                        format!("unsupported `@rust.async(...)` key `{other}`"),
                    );
                    valid = false;
                }
            }
        }

        valid
    }

    pub(super) fn async_thread_affinity_for_probe(
        &self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
    ) -> AsyncThreadAffinity {
        let key = canonical_sifr_target_path(declaration);
        if let Some(affinity) = self.async_contracts.get(&key).copied() {
            return affinity;
        }
        self.opaque_owner_thread_affinity(declaration)
            .unwrap_or_default()
    }

    fn opaque_owner_thread_affinity(
        &self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
    ) -> Option<AsyncThreadAffinity> {
        let RustInteropOwner::Method { class_name, .. } = &declaration.owner else {
            return None;
        };
        let key = canonical_class_target_path(declaration.module_name.as_deref(), class_name);
        let contract = self.opaque_contracts.get(&key)?;
        match contract.thread_affinity {
            OpaqueThreadAffinity::TokioCurrentThread => {
                Some(AsyncThreadAffinity::TokioCurrentThread)
            }
            OpaqueThreadAffinity::None | OpaqueThreadAffinity::CurrentOsThread => {
                Some(AsyncThreadAffinity::None)
            }
        }
    }

    fn push_async_diagnostic(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        range: ruff_text_size::TextRange,
        reason: impl Into<String>,
    ) {
        self.push_diagnostic(
            declaration,
            range,
            DiagnosticCode::RUST_ASYNC_CONTRACT,
            "invalid Rust async contract: {reason}",
            vec![("reason", reason.into())],
            Vec::new(),
            None,
        );
    }
}

fn canonical_class_target_path(module_name: Option<&str>, class_name: &str) -> String {
    let mut path = module_name.unwrap_or("main").to_string();
    path.push('.');
    path.push_str(class_name);
    path
}

fn explicit_async_thread_affinity(
    declaration: &sifr_codegen::RustInteropPlanDeclaration,
) -> Option<AsyncThreadAffinity> {
    declaration
        .declaration
        .arguments
        .iter()
        .find(|argument| argument.name.as_deref() == Some("thread_affinity"))
        .and_then(|argument| match &argument.value {
            RustInteropValue::Symbol(symbol) if symbol == "tokio_current_thread" => {
                Some(AsyncThreadAffinity::TokioCurrentThread)
            }
            RustInteropValue::Symbol(symbol) if symbol == "none" => Some(AsyncThreadAffinity::None),
            _ => None,
        })
}
