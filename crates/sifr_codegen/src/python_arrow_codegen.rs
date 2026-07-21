use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{RustEmitter, RustExpr, RustParam, RustStmt, RustType};
use sifr_ir::{
    HirExpr, HirFunction, PythonArrowDeclaration, PythonArrowSchemaMode,
    PythonInteropDecoratorKind, PythonParameterKind,
};
use sifr_type_system::{PythonArrowKind, Type};

pub(crate) fn append_argument_preparation(
    body: &mut Vec<RustStmt>,
    parameter_name: &str,
    index: usize,
    kind: PythonArrowKind,
    shape_kind: PythonParameterKind,
    shape_name: &str,
    forward_positional_by_name: bool,
    error_type: &Type,
) -> Option<String> {
    let guard_name = format!("__sifr_python_arrow_argument_{index}");
    let handle_name = format!("__sifr_python_arg_{index}");
    body.push(crate::python_interop_runtime_exprs::mapped_let(
        &guard_name,
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_stdlib".to_string(),
                "python".to_string(),
                kind.rust_name().to_string(),
                "prepare_argument".to_string(),
            ])),
            args: vec![RustExpr::Ident(parameter_name.to_string())],
        },
        error_type,
    ));
    body.push(crate::python_interop_runtime_exprs::mapped_let(
        &handle_name,
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(guard_name.clone())),
            method: "object".to_string(),
            args: Vec::new(),
        },
        error_type,
    ));
    body.push(
        if shape_kind == PythonParameterKind::Positional && forward_positional_by_name {
            crate::python_interop_direct_helpers::push_named_keyword(shape_name, &handle_name)
        } else {
            crate::python_interop_direct_helpers::push_for_shape(
                shape_kind,
                shape_name,
                &handle_name,
            )?
        },
    );
    Some(guard_name)
}

pub(crate) fn acquire_python_arrow_from_foreign(
    producer: RustExpr,
    contract: &PythonArrowDeclaration,
    error_type: &Type,
) -> RustExpr {
    acquire_python_arrow(producer, contract, error_type, true)
}

fn acquire_python_arrow_from_receiver(
    producer: RustExpr,
    contract: &PythonArrowDeclaration,
    error_type: &Type,
) -> RustExpr {
    acquire_python_arrow(producer, contract, error_type, true)
}

fn acquire_python_arrow(
    producer: RustExpr,
    contract: &PythonArrowDeclaration,
    error_type: &Type,
    foreign: bool,
) -> RustExpr {
    let method = match (&contract.schema, foreign) {
        (PythonArrowSchemaMode::Omitted, true) => "acquire_foreign",
        (PythonArrowSchemaMode::Omitted, false) => "acquire",
        (PythonArrowSchemaMode::Parameter { .. }, true) => "acquire_foreign_with_schema",
        (PythonArrowSchemaMode::Parameter { .. }, false) => "acquire_with_schema",
    };
    let mut args = vec![RustExpr::Ref {
        mutable: false,
        expr: Box::new(producer),
    }];
    if let PythonArrowSchemaMode::Parameter { name, .. } = &contract.schema {
        args.push(RustExpr::Ident(name.clone()));
    }
    crate::python_interop_runtime_exprs::mapped_try(
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_stdlib".to_string(),
                "python".to_string(),
                contract.kind.rust_name().to_string(),
                method.to_string(),
            ])),
            args,
        },
        error_type,
    )
}

pub(crate) fn receiver_interop_body(func: &HirFunction) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if declaration.kind != PythonInteropDecoratorKind::Arrow {
        return None;
    }
    let Type::Result(ok_type, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    if !matches!(ok_type.resolve_alias(), Type::PythonArrow(_)) {
        return None;
    }
    let target = declaration.target.as_ref()?;
    if target.segments.as_slice() != ["Self"] {
        return None;
    }
    let acquired = acquire_python_arrow_from_receiver(
        RustExpr::Field {
            expr: Box::new(RustExpr::Ident("self".to_string())),
            field: "__sifr_python_object".to_string(),
        },
        declaration.arrow.as_ref()?,
        error_type,
    );
    Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![acquired],
    }))])
}

pub(crate) fn lower_python_arrow_method(
    emitter: &mut RustEmitter,
    object: &HirExpr,
    method: &str,
    args: &[HirExpr],
    result_type: &Type,
) -> Option<RustExpr> {
    if !matches!(object.ty().resolve_alias(), Type::PythonArrow(_)) {
        return None;
    }
    let call = RustExpr::MethodCall {
        receiver: Box::new(emitter.try_lower_registry_expr_strict(object)?),
        method: method.to_string(),
        args: emitter.try_lower_registry_exprs_strict(args)?,
    };
    let Type::Result(_, error_type) = result_type.resolve_alias() else {
        return Some(call);
    };
    let error_name = "__sifr_python_arrow_error";
    Some(RustExpr::MethodCall {
        receiver: Box::new(call),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: error_name.to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(bridge_error_expr(
                RustExpr::Ident(error_name.to_string()),
                error_type,
            )),
            is_move: false,
        }],
    })
}
