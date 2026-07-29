use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{RustEmitter, RustExpr, RustParam, RustStmt, RustType};
use sifr_ir::{
    HirExpr, HirFunction, PythonBufferAccess, PythonBufferDeclaration, PythonBufferLayout,
    PythonInteropDecoratorKind,
};
use sifr_type_system::Type;

pub(crate) fn acquire_python_buffer(
    producer: RustExpr,
    contract: &PythonBufferDeclaration,
    error_type: &Type,
) -> RustExpr {
    acquire_python_buffer_with_method(producer, contract, error_type, "acquire")
}

fn acquire_python_buffer_from_receiver(
    producer: RustExpr,
    contract: &PythonBufferDeclaration,
    error_type: &Type,
) -> RustExpr {
    acquire_python_buffer_with_method(producer, contract, error_type, "acquire_foreign")
}

fn acquire_python_buffer_with_method(
    producer: RustExpr,
    contract: &PythonBufferDeclaration,
    error_type: &Type,
    method: &str,
) -> RustExpr {
    let element = contract.element_type.rust_type();
    let access = match contract.access {
        PythonBufferAccess::Read => "Read",
        PythonBufferAccess::Write => "Write",
    };
    let layout = match contract.layout {
        PythonBufferLayout::Any => "Any",
        PythonBufferLayout::CContiguous => "CContiguous",
        PythonBufferLayout::FContiguous => "FContiguous",
    };
    crate::python_interop_runtime_exprs::mapped_try(
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_stdlib".to_string(),
                "python".to_string(),
                format!("PythonBuffer::<{element}>"),
                method.to_string(),
            ])),
            args: vec![
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(producer),
                },
                RustExpr::Path(vec![
                    "sifr_runtime".to_string(),
                    "python".to_string(),
                    "PythonBufferAccess".to_string(),
                    access.to_string(),
                ]),
                RustExpr::Path(vec![
                    "sifr_runtime".to_string(),
                    "python".to_string(),
                    "PythonBufferLayout".to_string(),
                    layout.to_string(),
                ]),
            ],
        },
        error_type,
    )
}

pub(crate) fn acquire_python_buffer_from_foreign(
    producer: RustExpr,
    contract: &PythonBufferDeclaration,
    error_type: &Type,
) -> RustExpr {
    let owned_handle = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "interop".to_string(),
            "Handle".to_string(),
            "new".to_string(),
        ])),
        args: vec![producer],
    };
    acquire_python_buffer(owned_handle, contract, error_type)
}

pub(crate) fn receiver_interop_body(func: &HirFunction) -> Option<Vec<RustStmt>> {
    let declaration = func.python_interop.first()?;
    if declaration.kind != PythonInteropDecoratorKind::Buffer {
        return None;
    }
    let Type::Result(ok_type, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    if !matches!(ok_type.resolve_alias(), Type::PythonBuffer(_)) {
        return None;
    }
    let target = declaration.target.as_ref()?;
    if target.segments.as_slice() != ["Self"] {
        return None;
    }
    let acquired = acquire_python_buffer_from_receiver(
        RustExpr::Field {
            expr: Box::new(RustExpr::Ident("self".to_string())),
            field: "__sifr_python_object".to_string(),
        },
        declaration.buffer.as_ref()?,
        error_type,
    );
    Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![acquired],
    }))])
}

pub(crate) fn lower_python_buffer_method(
    emitter: &mut RustEmitter,
    object: &HirExpr,
    method: &str,
    args: &[HirExpr],
    places: crate::place_emitter::MethodCallPlaces<'_>,
    result_type: &Type,
) -> Option<RustExpr> {
    if !matches!(object.ty().resolve_alias(), Type::PythonBuffer(_)) {
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
    let error_name = "__sifr_python_buffer_error";
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
