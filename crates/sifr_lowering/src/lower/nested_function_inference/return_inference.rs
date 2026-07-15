use super::{
    expression_inference::has_conflicting_inference, unify_types, HashMap, LocalFunctionState, Type,
};

pub(super) fn unify_function_return(
    function_name: &str,
    incoming: Type,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
) {
    let Some(state) = states.get_mut(function_name) else {
        return;
    };
    if !state.explicit_return && has_conflicting_inference(&state.return_type, &incoming) {
        if state.allow_union_return_inference {
            state.return_type =
                sifr_type_system::make_union(vec![state.return_type.clone(), incoming]);
        } else {
            state.return_type = Type::Unknown;
            state.inference_failed = true;
        }
    } else {
        state.return_type = unify_types(state.return_type.clone(), incoming);
    }
}
