use ruff_text_size::TextRange;
use sifr_codegen::{RustInteropOwner, RustInteropPlanDeclaration};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropValue, RustTargetPath};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OpaqueContract {
    pub(super) rust_type: RustTargetPath,
    pub(super) structural_mapping: Option<RustTargetPath>,
    pub(super) send: bool,
    pub(super) sync: bool,
    pub(super) clone_policy: OpaqueClonePolicy,
    pub(super) close_policy: OpaqueClosePolicy,
    pub(super) borrow_policy: OpaqueBorrowPolicy,
    pub(super) thread_affinity: OpaqueThreadAffinity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OpaqueClonePolicy {
    None,
    Copy,
    Arc,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OpaqueClosePolicy {
    Drop,
    Close,
    AsyncClose,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OpaqueBorrowPolicy {
    Shared,
    Exclusive,
    Owned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OpaqueThreadAffinity {
    None,
    TokioCurrentThread,
    CurrentOsThread,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OpaqueContractDiagnostic {
    pub(super) code: DiagnosticCode,
    pub(super) span: TextRange,
    pub(super) message_template: &'static str,
    pub(super) args: Vec<(&'static str, String)>,
    pub(super) notes: Vec<String>,
    pub(super) help: Option<String>,
}

pub(super) fn parse_opaque_contract(
    declaration: &RustInteropPlanDeclaration,
) -> Result<OpaqueContract, Vec<OpaqueContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    if !matches!(declaration.owner, RustInteropOwner::Class { .. }) {
        diagnostics.push(malformed(
            declaration.declaration.span,
            "`@rust.opaque(...)` is valid only on classes",
        ));
    }

    let mut rust_type = None;
    let mut structural_mapping = None;
    let mut send = false;
    let mut sync = false;
    let mut clone_policy = OpaqueClonePolicy::None;
    let mut close_policy = OpaqueClosePolicy::Drop;
    let mut borrow_policy = OpaqueBorrowPolicy::Shared;
    let mut thread_affinity = OpaqueThreadAffinity::None;

    for argument in &declaration.declaration.arguments {
        let Some(name) = argument.name.as_deref() else {
            diagnostics.push(malformed(
                argument.span,
                "`@rust.opaque(...)` requires named arguments",
            ));
            continue;
        };
        match name {
            "type" => match &argument.value {
                RustInteropValue::TargetPath(path) => rust_type = Some(path.clone()),
                _ => diagnostics.push(malformed(
                    argument.span,
                    "`type=` must be a dotted Rust target path",
                )),
            },
            "structural" => match &argument.value {
                RustInteropValue::TargetPath(path) => structural_mapping = Some(path.clone()),
                _ => diagnostics.push(malformed(
                    argument.span,
                    "`structural=` must be a dotted Rust mapping type path",
                )),
            },
            "send" => match &argument.value {
                RustInteropValue::Boolean(value) => send = *value,
                _ => diagnostics.push(malformed(argument.span, "`send=` must be True or False")),
            },
            "sync" => match &argument.value {
                RustInteropValue::Boolean(value) => sync = *value,
                _ => diagnostics.push(malformed(argument.span, "`sync=` must be True or False")),
            },
            "clone" => match clone_policy_value(&argument.value) {
                Some(policy) => clone_policy = policy,
                None => diagnostics.push(malformed(
                    argument.span,
                    "`clone=` must be none, copy, arc, or custom(path)",
                )),
            },
            "close" => match symbol_value(&argument.value) {
                Some("drop") => close_policy = OpaqueClosePolicy::Drop,
                Some("close") => close_policy = OpaqueClosePolicy::Close,
                Some("async_close") => close_policy = OpaqueClosePolicy::AsyncClose,
                Some("none") => close_policy = OpaqueClosePolicy::None,
                _ => diagnostics.push(malformed(
                    argument.span,
                    "`close=` must be drop, close, async_close, or none",
                )),
            },
            "borrow" => match symbol_value(&argument.value) {
                Some("shared") => borrow_policy = OpaqueBorrowPolicy::Shared,
                Some("exclusive") => borrow_policy = OpaqueBorrowPolicy::Exclusive,
                Some("owned") => borrow_policy = OpaqueBorrowPolicy::Owned,
                _ => diagnostics.push(malformed(
                    argument.span,
                    "`borrow=` must be shared, exclusive, or owned",
                )),
            },
            "thread_affinity" => match symbol_value(&argument.value) {
                Some("none") => thread_affinity = OpaqueThreadAffinity::None,
                Some("tokio_current_thread") => {
                    thread_affinity = OpaqueThreadAffinity::TokioCurrentThread;
                }
                Some("current_os_thread") => {
                    thread_affinity = OpaqueThreadAffinity::CurrentOsThread;
                }
                _ => diagnostics.push(malformed(
                    argument.span,
                    "`thread_affinity=` must be none, tokio_current_thread, or current_os_thread",
                )),
            },
            other => diagnostics.push(malformed(
                argument.span,
                format!("unsupported `@rust.opaque(...)` key `{other}`"),
            )),
        }
    }

    let Some(rust_type) = rust_type else {
        diagnostics.push(malformed(
            declaration.declaration.span,
            "`@rust.opaque(...)` requires `type=`",
        ));
        return Err(diagnostics);
    };
    if structural_mapping.is_some()
        && matches!(
            close_policy,
            OpaqueClosePolicy::Close | OpaqueClosePolicy::AsyncClose
        )
    {
        diagnostics.push(malformed(
            declaration.declaration.span,
            "structurally mapped opaque values cannot declare an explicit close method",
        ));
    }
    if structural_mapping.is_some() && thread_affinity != OpaqueThreadAffinity::None {
        diagnostics.push(malformed(
            declaration.declaration.span,
            "structurally mapped opaque values cannot declare thread affinity",
        ));
    }
    if structural_mapping.is_some()
        && matches!(
            clone_policy,
            OpaqueClonePolicy::Arc | OpaqueClonePolicy::Custom
        )
    {
        diagnostics.push(malformed(
            declaration.declaration.span,
            "structurally mapped opaque values cannot use resource-sharing clone policies",
        ));
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(OpaqueContract {
        rust_type,
        structural_mapping,
        send,
        sync,
        clone_policy,
        close_policy,
        borrow_policy,
        thread_affinity,
    })
}

pub(super) fn close_method_name(policy: OpaqueClosePolicy) -> Option<&'static str> {
    match policy {
        OpaqueClosePolicy::Close => Some("close"),
        OpaqueClosePolicy::AsyncClose => Some("aclose"),
        OpaqueClosePolicy::Drop | OpaqueClosePolicy::None => None,
    }
}

fn clone_policy_value(value: &RustInteropValue) -> Option<OpaqueClonePolicy> {
    match value {
        RustInteropValue::Symbol(symbol) if symbol == "none" => Some(OpaqueClonePolicy::None),
        RustInteropValue::Symbol(symbol) if symbol == "copy" => Some(OpaqueClonePolicy::Copy),
        RustInteropValue::Symbol(symbol) if symbol == "arc" => Some(OpaqueClonePolicy::Arc),
        RustInteropValue::PolicyCall { name, argument, .. }
            if name == "custom" && matches!(argument.as_ref(), RustInteropValue::TargetPath(_)) =>
        {
            Some(OpaqueClonePolicy::Custom)
        }
        _ => None,
    }
}

fn symbol_value(value: &RustInteropValue) -> Option<&str> {
    match value {
        RustInteropValue::Symbol(symbol) => Some(symbol),
        _ => None,
    }
}

fn malformed(span: TextRange, reason: impl Into<String>) -> OpaqueContractDiagnostic {
    OpaqueContractDiagnostic {
        code: DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
        span,
        message_template: "malformed Rust interop decorator: {reason}",
        args: vec![("reason", reason.into())],
        notes: Vec::new(),
        help: None,
    }
}
