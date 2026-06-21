use sifr_codegen::{RustBridgeProbeKind, RustInteropOwner};
use sifr_ir::{RustInteropDeclaration, RustInteropDecoratorKind};

pub(super) fn probe_kind(
    declaration: &RustInteropDeclaration,
    owner: &RustInteropOwner,
) -> Option<RustBridgeProbeKind> {
    match declaration.kind {
        RustInteropDecoratorKind::Function => {
            if declaration.abi_requirements.async_boundary {
                Some(RustBridgeProbeKind::AsyncFunction)
            } else if matches!(owner, RustInteropOwner::Method { .. }) {
                Some(RustBridgeProbeKind::Method)
            } else if matches!(owner, RustInteropOwner::Function { .. }) {
                Some(RustBridgeProbeKind::Function)
            } else {
                None
            }
        }
        RustInteropDecoratorKind::Opaque => matches!(owner, RustInteropOwner::Class { .. })
            .then_some(RustBridgeProbeKind::OpaqueHandle),
        RustInteropDecoratorKind::Async => matches!(
            owner,
            RustInteropOwner::Function { .. } | RustInteropOwner::Method { .. }
        )
        .then_some(RustBridgeProbeKind::AsyncFunction),
        RustInteropDecoratorKind::Callback => None,
        RustInteropDecoratorKind::ZeroCopy => matches!(
            owner,
            RustInteropOwner::Function { .. } | RustInteropOwner::Method { .. }
        )
        .then_some(RustBridgeProbeKind::ZeroCopy),
        RustInteropDecoratorKind::View => matches!(
            owner,
            RustInteropOwner::Function { .. } | RustInteropOwner::Method { .. }
        )
        .then_some(RustBridgeProbeKind::View),
    }
}
