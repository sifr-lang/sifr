use sifr_codegen::RustInteropPlanDeclaration;
use sifr_ir::{RustInteropDeclaration, RustInteropValue};
use sifr_package::SifrPackageMetadata;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EffectivePanicPolicy {
    None,
    TrustedNoPanic,
    Abort,
    MapError,
    Invalid,
    InvalidSysrootImplicitTarget,
}

impl EffectivePanicPolicy {
    pub(super) fn is_some(self) -> bool {
        !matches!(self, Self::None)
    }

    pub(super) fn is_abort(self) -> bool {
        matches!(self, Self::Abort)
    }
}

pub(super) fn build_env_trust_entries(declaration: &RustInteropDeclaration) -> Vec<String> {
    declaration
        .arguments
        .iter()
        .filter_map(|argument| {
            if argument.name.as_deref() != Some("build_env") {
                return None;
            }
            match &argument.value {
                RustInteropValue::Symbol(name) => Some(name.clone()),
                _ => None,
            }
        })
        .collect()
}

pub(super) fn effective_panic_policy(
    declaration: &RustInteropPlanDeclaration,
    package: &SifrPackageMetadata,
    trusted_sysroot_package: bool,
) -> EffectivePanicPolicy {
    let explicit = explicit_panic_policy(&declaration.declaration);
    if explicit != EffectivePanicPolicy::None {
        return explicit;
    }
    if is_implicit_sysroot_stdlib_no_panic(declaration, package, trusted_sysroot_package) {
        return EffectivePanicPolicy::TrustedNoPanic;
    }
    if is_private_sysroot_declaration(declaration, package, trusted_sysroot_package) {
        return EffectivePanicPolicy::InvalidSysrootImplicitTarget;
    }
    EffectivePanicPolicy::None
}

fn explicit_panic_policy(declaration: &RustInteropDeclaration) -> EffectivePanicPolicy {
    let mut surface = EffectivePanicPolicy::None;
    for argument in declaration
        .arguments
        .iter()
        .filter(|argument| argument.name.as_deref() == Some("panic"))
    {
        let next = match &argument.value {
            RustInteropValue::Symbol(policy) if policy == "trusted_no_panic" => {
                EffectivePanicPolicy::TrustedNoPanic
            }
            RustInteropValue::Symbol(policy) if policy == "abort" => EffectivePanicPolicy::Abort,
            RustInteropValue::PolicyCall { name, argument, .. }
                if name == "map_error"
                    && matches!(argument.as_ref(), RustInteropValue::TargetPath(_)) =>
            {
                EffectivePanicPolicy::MapError
            }
            _ => EffectivePanicPolicy::Invalid,
        };
        if surface != EffectivePanicPolicy::None || next == EffectivePanicPolicy::Invalid {
            return EffectivePanicPolicy::Invalid;
        }
        surface = next;
    }
    surface
}

fn is_implicit_sysroot_stdlib_no_panic(
    declaration: &RustInteropPlanDeclaration,
    package: &SifrPackageMetadata,
    trusted_sysroot_package: bool,
) -> bool {
    is_private_sysroot_declaration(declaration, package, trusted_sysroot_package)
        && declaration
            .declaration
            .target
            .as_ref()
            .and_then(|target| target.segments.first())
            .is_some_and(|root| root == "sifr_stdlib")
}

fn is_private_sysroot_declaration(
    declaration: &RustInteropPlanDeclaration,
    package: &SifrPackageMetadata,
    trusted_sysroot_package: bool,
) -> bool {
    trusted_sysroot_package
        && package.sifr_name.0 == "_sifr"
        && declaration
            .module_name
            .as_deref()
            .is_some_and(|module| module.starts_with("_sifr."))
}
