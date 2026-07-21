use crate::python_interop_direct_helpers::drop_value;
use crate::RustStmt;
use sifr_ir::PythonParameterKind;
use sifr_type_system::Type;

#[derive(Default)]
pub(crate) struct ArgumentGuards {
    arrow: Vec<String>,
    dlpack: Vec<String>,
}

pub(crate) struct ArgumentPreparation<'a> {
    pub(crate) parameter_name: &'a str,
    pub(crate) index: usize,
    pub(crate) ty: &'a Type,
    pub(crate) shape_kind: PythonParameterKind,
    pub(crate) shape_name: &'a str,
    pub(crate) forward_positional_by_name: bool,
    pub(crate) error_type: &'a Type,
}

impl ArgumentGuards {
    pub(crate) fn append_preparation(
        &mut self,
        body: &mut Vec<RustStmt>,
        input: ArgumentPreparation<'_>,
    ) -> Option<bool> {
        match input.ty.resolve_alias() {
            Type::PythonArrow(kind) => {
                self.arrow
                    .push(crate::python_arrow_codegen::append_argument_preparation(
                        body,
                        crate::python_arrow_codegen::ArgumentPreparation {
                            parameter_name: input.parameter_name,
                            index: input.index,
                            kind: *kind,
                            shape_kind: input.shape_kind,
                            shape_name: input.shape_name,
                            forward_positional_by_name: input.forward_positional_by_name,
                            error_type: input.error_type,
                        },
                    )?);
                Some(true)
            }
            Type::PythonDlpackTensor(_) => {
                self.dlpack
                    .push(crate::python_dlpack_codegen::append_argument_preparation(
                        body,
                        crate::python_dlpack_codegen::ArgumentPreparation {
                            parameter_name: input.parameter_name,
                            index: input.index,
                            shape_kind: input.shape_kind,
                            shape_name: input.shape_name,
                            forward_positional_by_name: input.forward_positional_by_name,
                            error_type: input.error_type,
                        },
                    )?);
                Some(true)
            }
            _ => Some(false),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.arrow.is_empty() && self.dlpack.is_empty()
    }

    pub(crate) fn append_reconciliation(&self, body: &mut Vec<RustStmt>, outcome_name: &str) {
        if self.is_empty() {
            return;
        }
        body.push(drop_value("__sifr_python_args"));
        body.push(drop_value("__sifr_python_kwargs"));
        if !self.arrow.is_empty() {
            crate::python_arrow_codegen::append_argument_reconciliation(
                body,
                &self.arrow,
                outcome_name,
            );
        }
        if !self.dlpack.is_empty() {
            crate::python_dlpack_codegen::append_argument_reconciliation(
                body,
                &self.dlpack,
                outcome_name,
            );
        }
    }
}
