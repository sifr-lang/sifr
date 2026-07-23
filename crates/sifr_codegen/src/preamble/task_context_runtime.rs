use super::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};
use crate::RustLiteral;

pub fn task_context_label_field() -> (String, RustType) {
    (
        "context_label".to_string(),
        RustType::Option(Box::new(RustType::Named("String".to_string()))),
    )
}

pub fn task_context_label_capture_stmt() -> RustStmt {
    RustStmt::Let {
        mutable: false,
        name: "child_context_label".to_string(),
        ty: None,
        value: RustExpr::Verbatim("self.context_label.clone()".to_string()),
    }
}

pub fn build_task_context_scope_extension_items(include_task_local: bool) -> Vec<RustItem> {
    let mut items = Vec::new();
    if include_task_local {
        items.push(RustItem::Attr(
            "tokio::task_local! { static __SIFR_TASK_CONTEXT_LABEL: String; }".to_string(),
        ));
    }
    items.push(RustItem::Attr(
        r#"impl __SifrTaskScope {
    fn new_task_group_with_context<C: std::fmt::Display>(context: C) -> Self {
        return Self {
            children: Vec::new(),
            fail_fast: true,
            context_label: Some(format!("{}", context)),
        };
    }

    fn __sifr_spawn_infallible_with_context<C: std::fmt::Display, T: Send + 'static, F: std::future::Future<Output = T> + Send + 'static>(&mut self, context: C, future: F) -> __SifrTask<T, std::convert::Infallible> {
        let previous_context_label = self.context_label.replace(format!("{}", context));
        let task = self.__sifr_spawn_infallible(future);
        self.context_label = previous_context_label;
        return task;
    }

    fn __sifr_spawn_result_with_context<C: std::fmt::Display, T: Send + 'static, E: Send + 'static, F: std::future::Future<Output = Result<T, E>> + Send + 'static>(&mut self, context: C, future: F) -> __SifrTask<T, E> {
        let previous_context_label = self.context_label.replace(format!("{}", context));
        let task = self.__sifr_spawn_result(future);
        self.context_label = previous_context_label;
        return task;
    }
}"#
        .to_string(),
    ));
    items
}

pub fn build_task_current_context_items(include_task_local: bool) -> Vec<RustItem> {
    let context_type = sifr_type_system::stdlib_class_rust_name("sifr.task", "Context");
    let mut items = Vec::new();
    if include_task_local {
        items.push(RustItem::Attr(
            "tokio::task_local! { static __SIFR_TASK_CONTEXT_LABEL: String; }".to_string(),
        ));
    }
    let default_label = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Literal(RustLiteral::Str("Context".to_string()))),
        method: "to_string".to_string(),
        args: Vec::new(),
    };
    let label = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__SIFR_TASK_CONTEXT_LABEL".to_string())),
            method: "try_with".to_string(),
            args: vec![RustExpr::Path(vec![
                "Clone".to_string(),
                "clone".to_string(),
            ])],
        }),
        method: "unwrap_or_else".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "_".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(default_label),
            is_move: false,
        }],
    };
    items.push(RustItem::Fn {
        name: "__sifr_task_current_context".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![],
        ret: Some(RustType::Named(context_type.clone())),
        body: vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![context_type, "new".to_string()])),
            args: vec![label],
        }))],
        is_async: false,
    });
    items
}
