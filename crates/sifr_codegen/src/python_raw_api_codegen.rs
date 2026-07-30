//! Raw `sifr.python` ergonomics lowered through the declaration converter.

use crate::python_interop_direct::{input_conversion, output_value_expr};
use crate::python_interop_runtime_exprs::{mapped_try, runtime_call};
use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{HirExpr, RustEmitter, RustExpr, RustParam, RustStmt, RustType};
use sifr_ir::CompilerIntrinsicId;
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn try_lower_python_raw_intrinsic(
        &mut self,
        intrinsic: CompilerIntrinsicId,
        args: &[HirExpr],
        result_type: &Type,
    ) -> Option<RustExpr> {
        match intrinsic {
            CompilerIntrinsicId::PythonFromValue => self.lower_python_from_value(args, result_type),
            CompilerIntrinsicId::PythonToValue => self.lower_python_to_value(args, result_type),
            CompilerIntrinsicId::PythonKwarg => self.lower_python_kwarg(args, result_type),
            _ => None,
        }
    }

    fn lower_python_from_value(
        &mut self,
        args: &[HirExpr],
        result_type: &Type,
    ) -> Option<RustExpr> {
        let [value] = args else {
            return None;
        };
        let Type::Result(_ok_type, error_type) = result_type.resolve_alias() else {
            return None;
        };
        let converted = self.raw_python_input_result(value)?;
        Some(mapped_python_result(
            converted,
            runtime_call(
                "__sifr_declaration_object_result",
                vec![RustExpr::Ident("__sifr_python_value".to_string())],
            ),
            error_type,
        ))
    }

    fn lower_python_to_value(&mut self, args: &[HirExpr], result_type: &Type) -> Option<RustExpr> {
        let [object] = args else {
            return None;
        };
        let Type::Result(ok_type, error_type) = result_type.resolve_alias() else {
            return None;
        };
        let input = self.raw_python_input_result(object)?;
        let converted = output_value_expr(
            "__sifr_python_object",
            ok_type,
            error_type,
            &self.python_opaque_classes,
        )?;
        Some(result_closure(
            vec![RustStmt::Let {
                mutable: false,
                name: "__sifr_python_object".to_string(),
                ty: None,
                value: mapped_try(input, error_type),
            }],
            converted,
            ok_type,
            error_type,
        ))
    }

    fn lower_python_kwarg(&mut self, args: &[HirExpr], result_type: &Type) -> Option<RustExpr> {
        let [name, value] = args else {
            return None;
        };
        let Type::Result(ok_type, error_type) = result_type.resolve_alias() else {
            return None;
        };
        let lowered_name = self.lower_stmt_expr_for_ir(name).ok().flatten()?;
        let owned_name = Self::clone_non_copy_name_expr_for_ir(name, lowered_name);
        let input = self.raw_python_input_result(value)?;
        let object = runtime_call(
            "__sifr_declaration_object_result",
            vec![RustExpr::Ident("__sifr_python_value".to_string())],
        );
        Some(result_closure(
            vec![RustStmt::Let {
                mutable: false,
                name: "__sifr_python_value".to_string(),
                ty: None,
                value: mapped_try(input, error_type),
            }],
            RustExpr::Tuple(vec![owned_name, object]),
            ok_type,
            error_type,
        ))
    }

    fn raw_python_input_result(&mut self, value: &HirExpr) -> Option<RustExpr> {
        if let HirExpr::Name { name, .. } = value {
            return self.raw_python_named_input(name, value.ty());
        }
        let lowered = self.lower_stmt_expr_for_ir(value).ok().flatten()?;
        Some(RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: false,
                name: "__sifr_python_raw_input".to_string(),
                ty: None,
                value: lowered,
            }],
            expr: Some(Box::new(
                self.raw_python_named_input("__sifr_python_raw_input", value.ty())?,
            )),
        })
    }

    fn raw_python_named_input(&self, name: &str, ty: &Type) -> Option<RustExpr> {
        if ty.is_python_object_contract() {
            return Some(runtime_call(
                "__sifr_declaration_object_argument",
                vec![reference(RustExpr::Ident(name.to_string()))],
            ));
        }
        input_conversion(name, ty, &self.python_opaque_classes)
    }

    pub(crate) fn try_lower_python_raw_object_method(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        receiver_convention: Option<sifr_type_system::ReceiverConvention>,
        receiver_target: Option<&sifr_ir::MutableReceiverTarget>,
        result_type: &Type,
    ) -> Option<RustExpr> {
        // Dynamic method selection below is limited to the audited generated
        // substrates sifr_stdlib::python::py_call_keyed and
        // sifr_stdlib::python::py_call_attr_keyed (plus the active @rust
        // attribute/item adapters).
        if !object.ty().is_python_object_contract() {
            return None;
        }
        let Type::Result(_ok_type, error_type) = result_type.resolve_alias() else {
            return None;
        };
        let function = match (method, args.len()) {
            ("get_attr", 1) => "py_get_attr",
            ("get_item", 1) => "py_get_item_str",
            ("call", 2) => "py_call_keyed",
            ("call_method", 3) => "py_call_attr_keyed",
            _ => return None,
        };
        let lowered_object = self
            .lower_method_receiver_place_for_stmt(object, receiver_convention, receiver_target)
            .ok()
            .flatten()?;
        let mut call_args = vec![reference(lowered_object)];
        for arg in args {
            let lowered = self.lower_stmt_expr_for_ir(arg).ok().flatten()?;
            call_args.push(reference(lowered));
        }
        let call = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_stdlib".to_string(),
                "python".to_string(),
                function.to_string(),
            ])),
            args: call_args,
        };
        Some(map_python_error(call, error_type))
    }
}

fn mapped_python_result(value: RustExpr, mapped_value: RustExpr, error_type: &Type) -> RustExpr {
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(value),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_python_value".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(mapped_value),
            is_move: false,
        }],
    };
    RustExpr::MethodCall {
        receiver: Box::new(mapped),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_python_error".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(bridge_error_expr(
                RustExpr::Ident("__sifr_python_error".to_string()),
                error_type,
            )),
            is_move: false,
        }],
    }
}

fn map_python_error(value: RustExpr, error_type: &Type) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(value),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_python_error".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(bridge_error_expr(
                RustExpr::Ident("__sifr_python_error".to_string()),
                error_type,
            )),
            is_move: false,
        }],
    }
}

fn reference(value: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(value),
    }
}

fn result_closure(
    mut body: Vec<RustStmt>,
    value: RustExpr,
    ok_type: &Type,
    error_type: &Type,
) -> RustExpr {
    body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![format!(
            "Ok::<{}, {}>",
            ok_type.rust_type(),
            error_type.rust_type()
        )])),
        args: vec![value],
    })));
    RustExpr::FnCall {
        func: Box::new(RustExpr::Paren(Box::new(RustExpr::ClosureBlock {
            params: Vec::new(),
            body,
            is_move: false,
            is_async: false,
        }))),
        args: Vec::new(),
    }
}
