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
                "acquire".to_string(),
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
    let acquired = acquire_python_buffer(
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
    result_type: &Type,
) -> Option<RustExpr> {
    if !matches!(object.ty().resolve_alias(), Type::PythonBuffer(_)) {
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
