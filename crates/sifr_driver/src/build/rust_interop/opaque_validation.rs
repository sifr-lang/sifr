use super::{RustInteropOwner, RustInteropResolver, canonical_sifr_target_path, opaque_contract};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDeclaration, RustInteropDecoratorKind};
use std::collections::HashMap;

impl RustInteropResolver<'_> {
    pub(super) fn validate_opaque_declaration(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
    ) -> bool {
        match opaque_contract::parse_opaque_contract(declaration) {
            Ok(contract) => {
                self.opaque_contracts
                    .insert(canonical_sifr_target_path(declaration), contract);
                true
            }
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    self.push_diagnostic(
                        declaration,
                        diagnostic.span,
                        diagnostic.code,
                        diagnostic.message_template,
                        diagnostic.args,
                        diagnostic.notes,
                        diagnostic.help,
                    );
                }
                false
            }
        }
    }

    pub(super) fn validate_opaque_close_contracts(
        &mut self,
        declarations: &[sifr_codegen::RustInteropPlanDeclaration],
    ) {
        for declaration in declarations {
            let RustInteropOwner::Class { name: class_name } = &declaration.owner else {
                continue;
            };
            let target = canonical_sifr_target_path(declaration);
            let Some(contract) = self.opaque_contracts.get(&target) else {
                continue;
            };
            let Some(method_name) = opaque_contract::close_method_name(contract.close_policy)
            else {
                continue;
            };
            let found = declarations.iter().any(|candidate| {
                matches!(
                    &candidate.owner,
                    RustInteropOwner::Method { class_name: owner_class, name }
                        if owner_class == class_name && name == method_name
                ) && candidate.declaration.consumes_receiver
                    && close_method_kind_matches(method_name, &candidate.declaration)
            });
            if found {
                continue;
            }
            self.push_diagnostic(
                declaration,
                declaration.declaration.span,
                DiagnosticCode::RUST_HANDLE_CONTRACT,
                "opaque Rust handle `{target}` requires `{method}` cleanup method",
                vec![("target", target), ("method", method_name.to_string())],
                vec![
                    "declare a Rust interop method with an owning receiver for explicit handle cleanup"
                        .to_string(),
                ],
                None,
            );
        }
    }
}

fn close_method_kind_matches(method_name: &str, declaration: &RustInteropDeclaration) -> bool {
    if method_name == "aclose" {
        declaration.kind == RustInteropDecoratorKind::Async
            || declaration.abi_requirements.async_boundary
            || declaration.effect == sifr_ir::RustInteropEffect::Async
    } else {
        matches!(declaration.kind, RustInteropDecoratorKind::Function)
            && !declaration.abi_requirements.async_boundary
    }
}

pub(super) fn opaque_probe_obligations(
    declaration: &sifr_codegen::RustInteropPlanDeclaration,
    contracts: &HashMap<String, opaque_contract::OpaqueContract>,
) -> (bool, bool) {
    if declaration.declaration.kind != RustInteropDecoratorKind::Opaque {
        if declaration.declaration.kind == RustInteropDecoratorKind::View {
            return (false, false);
        }
        return (
            declaration.declaration.abi_requirements.async_boundary,
            declaration.declaration.abi_requirements.view,
        );
    }
    contracts
        .get(&canonical_sifr_target_path(declaration))
        .map_or((false, false), |contract| (contract.send, contract.sync))
}
