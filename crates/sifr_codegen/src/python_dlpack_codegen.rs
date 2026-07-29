use crate::python_interop_direct::{mapped_let, mapped_try};
use crate::python_interop_direct_helpers::{push_for_shape, push_named_keyword};
use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{RustEmitter, RustExpr, RustLiteral, RustParam, RustStmt, RustType};
use sifr_ir::{
    HirExpr, HirFunction, PythonDlpackDeclaration, PythonDlpackStreamMode,
    PythonInteropDeclaration, PythonInteropDecoratorKind, PythonParameterKind,
};
use sifr_type_system::Type;

#[derive(Clone, Copy)]
pub(crate) struct ArgumentPreparation<'a> {
    pub(crate) parameter_name: &'a str,
    pub(crate) index: usize,
    pub(crate) shape_kind: PythonParameterKind,
    pub(crate) shape_name: &'a str,
    pub(crate) forward_positional_by_name: bool,
    pub(crate) error_type: &'a Type,
}

pub(crate) fn append_argument_preparation(
    body: &mut Vec<RustStmt>,
    input: ArgumentPreparation<'_>,
) -> Option<String> {
    let ArgumentPreparation {
        parameter_name,
        index,
        shape_kind,
        shape_name,
        forward_positional_by_name,
        error_type,
    } = input;
    let guard_name = format!("__sifr_python_dlpack_argument_{index}");
    let handle_name = format!("__sifr_python_arg_{index}");
    body.push(mapped_let(
        &guard_name,
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(parameter_name.to_string())),
            method: "prepare_argument".to_string(),
            args: Vec::new(),
        },
        error_type,
    ));
    body.push(mapped_let(
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
            push_named_keyword(shape_name, &handle_name)
        } else {
            push_for_shape(shape_kind, shape_name, &handle_name)?
        },
    );
    Some(guard_name)
}

pub(crate) fn append_argument_reconciliation(
    body: &mut Vec<RustStmt>,
    guard_names: &[String],
    outcome_name: &str,
) {
    for (index, guard_name) in guard_names.iter().enumerate() {
        let cleanup_name = format!("__sifr_python_dlpack_cleanup_{index}");
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
                    "reconcile_dlpack_argument".to_string(),
                ])),
                args: vec![
                    RustExpr::Ident(outcome_name.to_string()),
                    RustExpr::Ident(cleanup_name),
                ],
            },
        });
    }
}

pub(crate) fn acquire_from_foreign(
    producer: RustExpr,
    contract: &PythonDlpackDeclaration,
    ok_type: &Type,
    error_type: &Type,
) -> Option<RustExpr> {
    let (rust_type, method) = match ok_type.resolve_alias() {
        Type::PythonDlpackTensor(element) => (
            format!(
                "::sifr_stdlib::python::PythonDlpackTensor::<{}>",
                element.rust_type()
            ),
            "acquire_foreign",
        ),
        Type::PythonDlpackStream => (
            "::sifr_stdlib::python::PythonDlpackStream".to_string(),
            "acquire_foreign",
        ),
        _ => return None,
    };
    let mut args = vec![RustExpr::Ref {
        mutable: false,
        expr: Box::new(producer),
    }];
    args.push(RustExpr::Ref {
        mutable: false,
        expr: Box::new(RustExpr::Literal(RustLiteral::Str(
            contract.device.label().to_string(),
        ))),
    });
    if matches!(ok_type.resolve_alias(), Type::PythonDlpackTensor(_)) {
        args.push(stream_option(&contract.stream));
    }
    Some(mapped_try(
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![rust_type, method.to_string()])),
            args,
        },
        error_type,
    ))
}

fn stream_option(stream: &PythonDlpackStreamMode) -> RustExpr {
    match stream {
        PythonDlpackStreamMode::None => RustExpr::Path(vec!["None".to_string()]),
        PythonDlpackStreamMode::Parameter { name, .. } => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Ident(name.clone())),
            }],
        },
    }
}

pub(crate) fn receiver_interop_body(
    func: &HirFunction,
    _owner_declaration: Option<&PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if !matches!(
        declaration.kind,
        PythonInteropDecoratorKind::Dlpack | PythonInteropDecoratorKind::DlpackStream
    ) {
        return None;
    }
    let Type::Result(ok_type, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    let target = declaration.target.as_ref()?;
    if target.segments.as_slice() != ["Self"] {
        return None;
    }
    let acquired = acquire_from_foreign(
        RustExpr::Field {
            expr: Box::new(RustExpr::Ident("self".to_string())),
            field: "__sifr_python_object".to_string(),
        },
        declaration.dlpack.as_ref()?,
        ok_type,
        error_type,
    )?;
    Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![acquired],
    }))])
}

pub(crate) fn lower_python_dlpack_method(
    emitter: &mut RustEmitter,
    object: &HirExpr,
    method: &str,
    args: &[HirExpr],
    places: crate::place_emitter::MethodCallPlaces<'_>,
    result_type: &Type,
) -> Option<RustExpr> {
    if !matches!(
        object.ty().resolve_alias(),
        Type::PythonDlpackTensor(_) | Type::PythonDlpackStream
    ) {
        return None;
    }
    let method_params = emitter.resolve_registry_method_params(object.ty(), method);
    let mut lowered_args = Vec::with_capacity(args.len());
    for (index, argument) in args.iter().enumerate() {
        let convention = method_params
            .as_ref()
            .and_then(|params| params.get(index))
            .map_or(
                sifr_type_system::ParamConvention::default(),
                |(_, convention)| *convention,
            );
        lowered_args.push(
            emitter.lower_method_argument_place_for_registry(
                argument,
                convention,
                places
                    .mutable_arg_places
                    .get(index)
                    .and_then(Option::as_ref),
            )?,
        );
    }
    let call = RustExpr::MethodCall {
        receiver: Box::new(emitter.lower_method_receiver_place_for_registry(
            object,
            places.receiver_convention,
            places.receiver_target,
        )?),
        method: method.to_string(),
        args: lowered_args,
    };
    let Type::Result(_, error_type) = result_type.resolve_alias() else {
        return Some(call);
    };
    let error_name = "__sifr_python_dlpack_error";
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
