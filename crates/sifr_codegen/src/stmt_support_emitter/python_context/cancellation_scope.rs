use crate::RustExpr;

/// Build the task-local scope call used for cancellation-masked Python cleanup.
pub(super) fn cleanup_scope_call(carrier: &str, manager: &str, cause: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Path(vec!["__SIFR_TASK_CANCELLATION".to_string()])),
        method: "scope".to_string(),
        args: vec![
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(carrier.to_string())),
                method: "clone".to_string(),
                args: Vec::new(),
            },
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "sifr_runtime".to_string(),
                    "python".to_string(),
                    "submit_async_context_exit_with_callbacks".to_string(),
                ])),
                args: vec![
                    RustExpr::Field {
                        expr: Box::new(RustExpr::Ident(manager.to_string())),
                        field: "__sifr_python_object".to_string(),
                    },
                    cause,
                    RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident(carrier.to_string())),
                        }],
                    },
                    RustExpr::Field {
                        expr: Box::new(RustExpr::Ident(manager.to_string())),
                        field: "__sifr_python_callbacks".to_string(),
                    },
                ],
            },
        ],
    }
}
