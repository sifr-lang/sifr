use crate::Type;

pub(super) fn cause_variant(error_type: &Type) -> &'static str {
    match error_type.resolve_alias() {
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.builtin.CancellationError" => "Cancellation",
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.builtin.TimeoutError" => "Timeout",
        Type::Class {
            identity: None,
            name,
            ..
        } if name == "RuntimeFault" => "RuntimeFault",
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.parallel.WorkerRuntimeError" => "RuntimeFault",
        _ => "OrdinaryError",
    }
}

pub(super) fn cause_label(cause: &str) -> &'static str {
    match cause {
        "Cancellation" => "cancellation",
        "Timeout" => "timeout",
        "RuntimeFault" => "runtime-fault",
        _ => "ordinary-error",
    }
}
