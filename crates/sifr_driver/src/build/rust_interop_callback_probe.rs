use super::rust_interop_probe::PendingRustBridgeProbe;
use sifr_codegen::{RustBridgeSignatureContract, RustBridgeTypeKind};

pub(super) fn signature_has_call_scoped_callback(probe: &PendingRustBridgeProbe) -> bool {
    probe
        .signature
        .as_ref()
        .is_some_and(signature_contract_has_call_scoped_callback)
}

pub(super) fn signature_contract_has_call_scoped_callback(
    signature: &RustBridgeSignatureContract,
) -> bool {
    signature
        .params
        .iter()
        .any(|param| param.ty.kind == RustBridgeTypeKind::CallScopedCallback)
}

pub(super) fn stderr_reports_callback_escape(stderr: &str) -> bool {
    stderr_reports_borrowed_escape(stderr, "CallScopedCallbackBridge")
}

pub(super) fn stderr_reports_slot_handler_escape(stderr: &str) -> bool {
    stderr_reports_borrowed_escape(stderr, "SlotHandler")
}

pub(super) fn stderr_reports_shared_context_mutation(stderr: &str) -> bool {
    (stderr.contains("SharedContext") || stderr.contains("context.get()"))
        && (stderr.contains("cannot borrow") || stderr.contains("cannot assign"))
        && stderr.contains("as mutable")
}

fn stderr_reports_borrowed_escape(stderr: &str, type_name: &str) -> bool {
    let mentions_bridge = stderr.contains(type_name);
    let reports_lifetime_escape = stderr.contains("E0521")
        || stderr.contains("E0597")
        || stderr.contains("E0759")
        || stderr.contains("borrowed data escapes")
        || stderr.contains("does not live long enough")
        || stderr.contains("lifetime may not live long enough");
    let reports_thread_escape = stderr.contains("cannot be sent between threads safely")
        || stderr.contains("cannot be shared between threads safely");
    mentions_bridge && (reports_lifetime_escape || reports_thread_escape)
}

#[cfg(test)]
mod tests {
    use super::{
        stderr_reports_callback_escape, stderr_reports_shared_context_mutation,
        stderr_reports_slot_handler_escape,
    };

    #[test]
    fn callback_escape_classifier_requires_bridge_and_escape_evidence() {
        assert!(stderr_reports_callback_escape(
            "error[E0521]: borrowed data escapes outside of function\n\
             CallScopedCallbackBridge<'call, (String,), ()>"
        ));
        assert!(stderr_reports_callback_escape(
            "CallScopedCallbackBridge<'_, (String,), ()> cannot be sent between threads safely"
        ));
        assert!(!stderr_reports_callback_escape(
            "error[E0308]: expected CallScopedCallbackBridge, found u64"
        ));
        assert!(!stderr_reports_callback_escape(
            "error[E0521]: borrowed data escapes outside of function"
        ));
        assert!(stderr_reports_slot_handler_escape(
            "SlotHandler<'_> cannot be sent between threads safely"
        ));
        assert!(stderr_reports_shared_context_mutation(
            "cannot borrow data in a `&` reference as mutable\nSharedContext<'_, String>"
        ));
    }
}
