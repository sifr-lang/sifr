use super::collect_type_vars;
use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::ExprCall;
use sifr_type_system::FunctionType;

pub(in crate::lower) fn call_argument_ranges_by_param(
    call: &ExprCall,
    ft: &FunctionType,
) -> Vec<Option<TextRange>> {
    let mut ranges = vec![None; ft.params.len()];

    for (index, arg) in call.arguments.args.iter().enumerate().take(ft.params.len()) {
        ranges[index] = Some(arg.range());
    }

    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            continue;
        };
        let Some(index) = ft
            .params
            .iter()
            .position(|(param_name, _, _)| param_name == name.as_str())
        else {
            continue;
        };
        ranges[index] = Some(keyword.value.range());
    }

    ranges
}

pub(in crate::lower) fn type_param_argument_range(
    call: &ExprCall,
    ft: &FunctionType,
    type_param_name: &str,
) -> Option<TextRange> {
    let arg_ranges = call_argument_ranges_by_param(call, ft);
    for (index, (_, param_ty, _)) in ft.params.iter().enumerate() {
        let mut type_vars = Vec::new();
        collect_type_vars(param_ty, &mut type_vars);
        if type_vars.iter().any(|name| name == type_param_name) {
            return arg_ranges.get(index).copied().flatten();
        }
    }
    None
}

pub(in crate::lower) fn first_call_keyword_range(call: &ExprCall) -> TextRange {
    call.arguments
        .keywords
        .first()
        .map_or_else(|| call.range(), Ranged::range)
}

pub(in crate::lower) fn call_arity_range(call: &ExprCall) -> TextRange {
    if call.arguments.args.is_empty() {
        call.range()
    } else {
        call.arguments.range()
    }
}
