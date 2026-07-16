use super::{
    expression_inference::infer_expr_type, infer_type_var_bindings, substitute_type_vars, ExprCall,
    FunctionEnv, FunctionType, HashMap, LocalFunctionState, LowerCtx, Type,
};

pub(super) struct InferredCall {
    pub(super) positional_types: Vec<Type>,
    pub(super) return_type: Type,
}

pub(super) fn infer_registered_call(
    call: &ExprCall,
    function_type: &FunctionType,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> InferredCall {
    let positional_types = call
        .arguments
        .args
        .iter()
        .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
        .collect::<Vec<_>>();
    let mut bindings = HashMap::new();
    for (argument, (_, parameter, _)) in positional_types.iter().zip(&function_type.params) {
        infer_type_var_bindings(parameter, argument, &mut bindings);
    }
    for keyword in &call.arguments.keywords {
        let keyword_type = infer_expr_type(&keyword.value, env, states, current_function, ctx);
        let Some(name) = keyword.arg.as_ref() else {
            continue;
        };
        if let Some((_, parameter, _)) = function_type
            .params
            .iter()
            .find(|(parameter_name, _, _)| parameter_name == name.as_str())
        {
            infer_type_var_bindings(parameter, &keyword_type, &mut bindings);
        }
    }
    InferredCall {
        positional_types,
        return_type: substitute_type_vars(&function_type.return_type, &bindings),
    }
}
