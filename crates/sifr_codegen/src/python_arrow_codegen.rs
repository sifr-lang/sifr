use crate::python_interop_direct_helpers::drop_value;
use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{RustEmitter, RustExpr, RustParam, RustStmt, RustType};
use sifr_ir::{
    HirExpr, HirFunction, PythonArrowDeclaration, PythonArrowSchemaMode,
    PythonInteropDecoratorKind, PythonParameterKind,
};
use sifr_type_system::{PythonArrowKind, Type};

#[derive(Clone, Copy)]
pub(crate) struct ArgumentPreparation<'a> {
    pub(crate) parameter_name: &'a str,
    pub(crate) index: usize,
    pub(crate) kind: PythonArrowKind,
    pub(crate) shape_kind: PythonParameterKind,
    pub(crate) shape_name: &'a str,
    pub(crate) forward_positional_by_name: bool,
    pub(crate) error_type: &'a Type,
}

pub(crate) fn append_argument_preparation(
    body: &mut Vec<RustStmt>,
    preparation: ArgumentPreparation<'_>,
) -> Option<String> {
    let ArgumentPreparation {
        parameter_name,
        index,
        kind,
        shape_kind,
        shape_name,
        forward_positional_by_name,
        error_type,
    } = preparation;
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

pub(crate) fn append_argument_reconciliation(
    body: &mut Vec<RustStmt>,
    guard_names: &[String],
    outcome_name: &str,
) {
    body.push(drop_value("__sifr_python_args"));
    body.push(drop_value("__sifr_python_kwargs"));
    for (index, guard_name) in guard_names.iter().enumerate() {
        let cleanup_name = format!("__sifr_python_arrow_cleanup_{index}");
        body.push(RustStmt::Let {
            mutable: false,
            name: cleanup_name.clone(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(guard_name.clone())),
                method: "finish".to_string(),
                args: Vec::new(),
            },
        });
        body.push(RustStmt::Let {
            mutable: false,
            name: outcome_name.to_string(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "sifr_stdlib".to_string(),
                    "python".to_string(),
                    "reconcile_arrow_argument".to_string(),
                ])),
                args: vec![
                    RustExpr::Ident(outcome_name.to_string()),
                    RustExpr::Ident(cleanup_name),
                ],
            },
        });
    }
}

pub(crate) fn acquire_python_arrow_from_foreign(
    producer: RustExpr,
    contract: &PythonArrowDeclaration,
    certification_target: &str,
    error_type: &Type,
) -> RustExpr {
    acquire_python_arrow(producer, contract, certification_target, error_type, true)
}

fn acquire_python_arrow_from_receiver(
    producer: RustExpr,
    contract: &PythonArrowDeclaration,
    certification_target: &str,
    error_type: &Type,
) -> RustExpr {
    acquire_python_arrow(producer, contract, certification_target, error_type, true)
}

fn acquire_python_arrow(
    producer: RustExpr,
    contract: &PythonArrowDeclaration,
    certification_target: &str,
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
    args.push(RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
            certification_target.to_string(),
        ))),
    });
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

pub(crate) fn receiver_interop_body(
    func: &HirFunction,
    owner_declaration: Option<&sifr_ir::PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
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
    let certification_target = owner_declaration?.target.as_ref()?.dotted();
    let acquired = acquire_python_arrow_from_receiver(
        RustExpr::Field {
            expr: Box::new(RustExpr::Ident("self".to_string())),
            field: "__sifr_python_object".to_string(),
        },
        declaration.arrow.as_ref()?,
        &certification_target,
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
