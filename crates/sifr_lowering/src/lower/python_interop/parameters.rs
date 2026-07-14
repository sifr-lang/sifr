use super::is_python_omit;
use ruff_text_size::Ranged;
use sifr_ir::{PythonInteropParameter, PythonParameterKind};
use sifr_python_ast::{AstParamOwnership, Parameters};

pub(in crate::lower) fn parameter_metadata(parameters: &Parameters) -> Vec<PythonInteropParameter> {
    let mut result = Vec::with_capacity(parameters.len());
    for parameter in parameters.posonlyargs.iter().chain(&parameters.args) {
        result.push(PythonInteropParameter {
            name: parameter.parameter.name.to_string(),
            kind: PythonParameterKind::Positional,
            has_default: parameter.default.is_some(),
            omit_when_absent: parameter.default.as_deref().is_some_and(is_python_omit),
            span: parameter.range(),
        });
    }
    if let Some(parameter) = &parameters.vararg {
        result.push(PythonInteropParameter {
            name: parameter.name.to_string(),
            kind: PythonParameterKind::PositionalVariadic,
            has_default: false,
            omit_when_absent: false,
            span: parameter.range(),
        });
    }
    for parameter in &parameters.kwonlyargs {
        result.push(PythonInteropParameter {
            name: parameter.parameter.name.to_string(),
            kind: PythonParameterKind::KeywordOnly,
            has_default: parameter.default.is_some(),
            omit_when_absent: parameter.default.as_deref().is_some_and(is_python_omit),
            span: parameter.range(),
        });
    }
    if let Some(parameter) = &parameters.kwarg {
        result.push(PythonInteropParameter {
            name: parameter.name.to_string(),
            kind: PythonParameterKind::KeywordVariadic,
            has_default: false,
            omit_when_absent: false,
            span: parameter.range(),
        });
    }
    result
}

pub(in crate::lower) fn receiver_is_owned(parameters: &Parameters) -> bool {
    parameters
        .args
        .first()
        .is_some_and(|parameter| parameter.parameter.convention.ownership == AstParamOwnership::Own)
}
