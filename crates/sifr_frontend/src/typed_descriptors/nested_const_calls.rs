use super::{
    ConstValue, DescriptorCollector, DescriptorResult, DeterministicConstEvaluator, DiagnosticCode,
    Expr, ExprCall, Ranged, Type, boxed_malformed, const_value_assignable, diagnostic,
};

impl DescriptorCollector<'_> {
    pub(super) fn nested_const_call(
        &self,
        call: &ExprCall,
        expected: &Type,
    ) -> DescriptorResult<ConstValue> {
        let Expr::Name(name) = call.func.as_ref() else {
            return Err(boxed_malformed(
                "descriptor const arguments require a direct const function name",
                call.range(),
            ));
        };
        let (module, function, functions) =
            self.const_call_target(name.id.as_str()).ok_or_else(|| {
                boxed_malformed(
                    "descriptor argument call is not a checked const function",
                    call.range(),
                )
            })?;
        if call.arguments.args.len() > function.params.len() {
            return Err(boxed_malformed(
                "descriptor const argument has too many positional arguments",
                call.range(),
            ));
        }
        let mut values = vec![None; function.params.len()];
        for (index, expression) in call.arguments.args.iter().enumerate() {
            values[index] = Some(self.argument_value(expression, &function.params[index].ty)?);
        }
        for keyword in &call.arguments.keywords {
            let Some(name) = &keyword.arg else {
                return Err(boxed_malformed(
                    "descriptor const arguments do not accept dictionary expansion",
                    keyword.range(),
                ));
            };
            let Some(index) = function
                .params
                .iter()
                .position(|parameter| parameter.name == name.as_str())
            else {
                return Err(boxed_malformed(
                    format!("descriptor const function has no parameter named '{name}'"),
                    keyword.range(),
                ));
            };
            if values[index].is_some() {
                return Err(boxed_malformed(
                    format!("descriptor const parameter '{name}' is supplied more than once"),
                    keyword.range(),
                ));
            }
            values[index] = Some(self.argument_value(&keyword.value, &function.params[index].ty)?);
        }
        let defaults = if module == self.module_name {
            self.result
                .function_defaults
                .get(&function.name)
                .cloned()
                .unwrap_or_default()
        } else {
            self.external_defs
                .function_defaults
                .get(&module)
                .and_then(|defaults| defaults.get(&function.name))
                .cloned()
                .unwrap_or_default()
        };
        for (index, value) in values.iter_mut().enumerate() {
            if value.is_none() {
                *value = defaults
                    .iter()
                    .find(|(default_index, _)| *default_index == index)
                    .and_then(|(_, expression)| {
                        crate::structural_shape::const_value_from_hir(expression)
                    });
            }
        }
        let arguments = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                value.ok_or_else(|| {
                    boxed_malformed(
                        format!(
                            "descriptor const call is missing required parameter '{}'",
                            function.params[index].name
                        ),
                        call.range(),
                    )
                })
            })
            .collect::<DescriptorResult<Vec<_>>>()?;
        let value = DeterministicConstEvaluator::new(&functions)
            .evaluate_function(&function.name, arguments)
            .map_err(|error| {
                Box::new(diagnostic(
                    DiagnosticCode::META_MALFORMED_DECLARATION,
                    format!(
                        "descriptor const argument failed evaluation: {}",
                        error.detail
                    ),
                    call.range(),
                ))
            })?;
        if const_value_assignable(&value, expected) {
            Ok(value)
        } else {
            Err(Box::new(diagnostic(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "descriptor const argument is not assignable to parameter type '{}'",
                    expected.display_name()
                ),
                call.range(),
            )))
        }
    }
}
