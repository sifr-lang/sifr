use super::RustInteropResolver;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::RustInteropDecoratorKind;
use std::collections::BTreeSet;

impl RustInteropResolver<'_> {
    pub(super) fn validate_structural_contracts(
        &mut self,
        declarations: &[sifr_codegen::RustInteropPlanDeclaration],
    ) {
        let mut diagnosed_packages = BTreeSet::new();
        for declaration in declarations.iter().filter(|declaration| {
            declaration.declaration.kind == RustInteropDecoratorKind::Structural
        }) {
            let Some(package_id) = self.package_id_for_module(declaration.module_name.as_deref())
            else {
                continue;
            };
            if diagnosed_packages.contains(&package_id) {
                continue;
            }
            let Some(package) = self.context.graph.packages.get(&package_id) else {
                continue;
            };
            if super::panic_validation::selected_panic_strategy(package).as_deref() != Some("abort")
            {
                continue;
            }
            diagnosed_packages.insert(package_id);
            self.push_diagnostic(
                declaration,
                declaration.declaration.span,
                DiagnosticCode::RUST_PANIC_CONTRACT,
                "invalid structural Rust bridge contract: generated structural bridges require an unwind-capable Cargo profile; `panic=abort` cannot preserve typed `RustPanicError` failures",
                Vec::new(),
                vec![
                    "Structural bridge code catches backend panics and translates them into the declared typed error channel.".to_string(),
                ],
                None,
            );
        }
    }
}
