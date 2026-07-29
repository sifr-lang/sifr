use crate::hir_nodes::MethodCallSource;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::{Expr, ExprCall};

pub(super) fn source_method_call(call: &ExprCall) -> MethodCallSource {
    let receiver_range = match call.func.as_ref() {
        Expr::Attribute(attribute) => attribute.value.range(),
        other => other.range(),
    };
    source_call_with_receiver(call, receiver_range)
}

pub(super) fn source_call_with_receiver(
    call: &ExprCall,
    receiver_range: TextRange,
) -> MethodCallSource {
    MethodCallSource {
        call_range: call.range(),
        receiver_range,
        arg_ranges: call
            .arguments
            .args
            .iter()
            .map(Ranged::range)
            .chain(
                call.arguments
                    .keywords
                    .iter()
                    .map(|keyword| keyword.value.range()),
            )
            .collect(),
    }
}

pub(super) fn source_call_with_first_arg_as_receiver(
    call: &ExprCall,
    receiver_range: TextRange,
) -> MethodCallSource {
    let mut source = source_call_with_receiver(call, receiver_range);
    if !source.arg_ranges.is_empty() {
        source.arg_ranges.remove(0);
    }
    source
}
