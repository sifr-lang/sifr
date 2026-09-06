use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use crate::stdlib_filter::{
    filter_stdlib_ir_to_needed, rust_source_defined_item_names, strip_rust_items_by_name,
};
use sifr_ir::{HirExpr, HirFunction, HirModule, HirStmt};
use std::collections::HashSet;

use super::{RustItem, Type};
pub(crate) fn sync_channel_runtime_needed(rust_code: &str) -> bool {
    rust_code.contains("struct Channel<")
        || rust_code.contains("struct ChannelSender<")
        || rust_code.contains("struct ChannelReceiver<")
        || rust_code.contains("fn channel<")
        || rust_code.contains("fn bounded_channel<")
}

pub(crate) fn replace_sync_channel_runtime_items(rust_code: &str) -> String {
    let strip_names = HashSet::from([
        "Channel",
        "ChannelSender",
        "ChannelReceiver",
        "channel",
        "bounded_channel",
    ]);
    let demanded = rust_source_defined_item_names(rust_code)
        .into_iter()
        .filter(|name| strip_names.contains(name.as_str()))
        .collect::<HashSet<_>>();
    if demanded.is_empty() {
        return rust_code.to_string();
    }
    let mut replaced = strip_rust_items_by_name(rust_code, &strip_names);
    if !replaced.trim().is_empty() {
        replaced.push('\n');
    }
    replaced.push_str(&filter_stdlib_ir_to_needed(
        sync_channel_runtime_rust_code(),
        &demanded,
    ));
    replaced
}

pub(crate) fn sync_channel_runtime_rust_code() -> &'static str {
    r#"
#[derive(Debug)]
struct __SifrChannelState<T> {
    buffer: std::collections::VecDeque<T>,
    closed: bool,
    capacity: SifrInt,
    sender_count: i64,
    receiver_alive: bool,
}
enum __SifrChannelPushState<T> {
    Sent,
    Closed(T),
    Full(T),
}
enum __SifrChannelPopState<T> {
    Item(T),
    Empty,
    Closed,
}
#[derive(Debug)]
struct Channel<T: Clone> {
    _state: std::sync::Arc<std::sync::Mutex<__SifrChannelState<T>>>,
    _send_notify: std::sync::Arc<tokio::sync::Notify>,
    _recv_notify: std::sync::Arc<tokio::sync::Notify>,
}
impl<T: Clone> Clone for Channel<T> {
    fn clone(&self) -> Self {
        return Self {
            _state: std::sync::Arc::clone(&self._state),
            _send_notify: std::sync::Arc::clone(&self._send_notify),
            _recv_notify: std::sync::Arc::clone(&self._recv_notify),
        };
    }
}
impl<T: Clone> Channel<T> {
    fn new(buffer: Vec<T>, capacity: SifrInt) -> Self {
        return Self {
            _state: std::sync::Arc::new(std::sync::Mutex::new(__SifrChannelState {
                buffer: buffer.into_iter().collect(),
                closed: false,
                capacity,
                sender_count: 0,
                receiver_alive: true,
            })),
            _send_notify: std::sync::Arc::new(tokio::sync::Notify::new()),
            _recv_notify: std::sync::Arc::new(tokio::sync::Notify::new()),
        };
    }

    fn __sifr_with_state<R>(&self, f: impl FnOnce(&mut __SifrChannelState<T>) -> R) -> R {
        match self._state.lock() {
            Ok(mut state) => f(&mut state),
            Err(poisoned) => {
                let mut state = poisoned.into_inner();
                f(&mut state)
            }
        }
    }

    fn is_closed(&self) -> bool {
        return self.__sifr_with_state(|state| state.closed || !state.receiver_alive);
    }

    fn close(&mut self) {
        self.__sifr_with_state(|state| {
            state.closed = true;
        });
        self._send_notify.notify_waiters();
        self._recv_notify.notify_waiters();
    }

    fn register_sender(&self) {
        self.__sifr_with_state(|state| {
            state.sender_count += 1;
        });
    }

    fn release_sender(&self) {
        self.__sifr_with_state(|state| {
            if state.sender_count > 0 {
                state.sender_count -= 1;
            }
            if state.sender_count == 0 {
                state.closed = true;
            }
        });
        self._recv_notify.notify_waiters();
    }

    fn release_receiver(&self) {
        self.__sifr_with_state(|state| {
            state.receiver_alive = false;
            state.closed = true;
        });
        self._send_notify.notify_waiters();
        self._recv_notify.notify_waiters();
    }

    fn try_push(&self, value: T) -> __SifrChannelPushState<T> {
        self.__sifr_with_state(|state| {
            if state.closed || !state.receiver_alive {
                return __SifrChannelPushState::Closed(value);
            }
            if &state.capacity >= &SifrInt::from_i64(0)
                && &SifrInt::from(state.buffer.len()) >= &state.capacity
            {
                return __SifrChannelPushState::Full(value);
            }
            state.buffer.push_back(value);
            self._recv_notify.notify_one();
            __SifrChannelPushState::Sent
        })
    }

    fn push(&mut self, value: T) -> Result<(), ClosedError> {
        self.__sifr_with_state(|state| {
            if state.closed || !state.receiver_alive {
                return Err(ClosedError::new("channel is closed".to_string()));
            }
            state.buffer.push_back(value);
            self._recv_notify.notify_one();
            Ok(())
        })
    }

    fn try_pop(&self) -> __SifrChannelPopState<T> {
        self.__sifr_with_state(|state| {
            if let Some(value) = state.buffer.pop_front() {
                self._send_notify.notify_one();
                return __SifrChannelPopState::Item(value);
            }
            if state.closed || state.sender_count == 0 {
                return __SifrChannelPopState::Closed;
            }
            __SifrChannelPopState::Empty
        })
    }

    fn pop(&mut self) -> Result<T, ClosedError> {
        match self.try_pop() {
            __SifrChannelPopState::Item(value) => Ok(value),
            __SifrChannelPopState::Empty | __SifrChannelPopState::Closed => Err(ClosedError::new("channel is closed".to_string())),
        }
    }
}

impl<T: Clone> std::fmt::Display for Channel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", "Channel".to_string());
    }
}

#[derive(Debug)]
struct ChannelSender<T: Clone> {
    _channel: Channel<T>,
}

impl<T: Clone> ChannelSender<T> {
    fn new(channel: Channel<T>) -> Self {
        channel.register_sender();
        return Self { _channel: channel };
    }

    async fn send(&mut self, mut value: T) -> Result<(), ClosedError> {
        loop {
            let notify = self._channel._send_notify.clone();
            let notified = notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            match self._channel.try_push(value) {
                __SifrChannelPushState::Sent => return Ok(()),
                __SifrChannelPushState::Closed(_) => return Err(ClosedError::new("channel is closed".to_string())),
                __SifrChannelPushState::Full(pending) => {
                    value = pending;
                    notified.await;
                }
            }
        }
    }

    fn close(&mut self) {
        self._channel.close();
    }
}

impl<T: Clone> Clone for ChannelSender<T> {
    fn clone(&self) -> Self {
        return ChannelSender::new(self._channel.clone());
    }
}

impl<T: Clone> Drop for ChannelSender<T> {
    fn drop(&mut self) {
        self._channel.release_sender();
    }
}

impl<T: Clone> std::fmt::Display for ChannelSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", "ChannelSender".to_string());
    }
}

#[derive(Debug)]
struct ChannelReceiver<T: Clone> {
    _channel: Channel<T>,
}

impl<T: Clone> ChannelReceiver<T> {
    fn new(channel: Channel<T>) -> Self {
        return Self { _channel: channel };
    }

    async fn receive(&mut self) -> Result<T, ClosedError> {
        loop {
            let notify = self._channel._recv_notify.clone();
            let notified = notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            match self._channel.try_pop() {
                __SifrChannelPopState::Item(value) => return Ok(value),
                __SifrChannelPopState::Closed => return Err(ClosedError::new("channel is closed".to_string())),
                __SifrChannelPopState::Empty => notified.await,
            }
        }
    }

    async fn anext(&mut self) -> Option<T> {
        match self.receive().await {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}

impl<T: Clone> Drop for ChannelReceiver<T> {
    fn drop(&mut self) {
        self._channel.release_receiver();
    }
}

impl<T: Clone> std::fmt::Display for ChannelReceiver<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", "ChannelReceiver".to_string());
    }
}

pub(crate) fn channel<T: Clone + 'static>() -> (ChannelSender<T>, ChannelReceiver<T>) {
    let shared_channel = Channel::new(vec![], SifrInt::from_i64(-1));
    return (
        ChannelSender::new(shared_channel.clone()),
        ChannelReceiver::new(shared_channel),
    );
}

pub(crate) fn bounded_channel<T: Clone + 'static>(capacity: SifrInt) -> (ChannelSender<T>, ChannelReceiver<T>) {
    let shared_channel = Channel::new(vec![], capacity);
    return (
        ChannelSender::new(shared_channel.clone()),
        ChannelReceiver::new(shared_channel),
    );
}
"#
}

pub(crate) fn annotate_async_main_entrypoint(items: &mut Vec<RustItem>) -> bool {
    for index in 0..items.len() {
        if let RustItem::Fn {
            name,
            is_async: true,
            ..
        } = &items[index]
        {
            if name == "main" {
                let already_annotated = index > 0
                    && matches!(
                        &items[index - 1],
                        RustItem::Attr(attr) if attr.contains("tokio::main")
                    );
                if !already_annotated {
                    items.insert(
                        index,
                        RustItem::Attr("#[tokio::main(flavor = \"current_thread\")]".to_string()),
                    );
                }
                return true;
            }
        }
    }
    false
}

pub(crate) fn module_uses_task_sleep(module: &HirModule) -> bool {
    fn expr_is_task_sleep(expr: &HirExpr) -> bool {
        matches!(expr, HirExpr::Call { func, .. } if func == "__sifr_task_sleep")
    }

    for (_, _, value) in &module.constants {
        let mut on_expr = |expr: &HirExpr| {
            if expr_is_task_sleep(expr) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        if matches!(
            traversal::walk_expr_until(value, &mut on_expr),
            TraversalControl::Stop
        ) {
            return true;
        }
    }

    for func in &module.functions {
        let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
        let mut on_expr = |expr: &HirExpr| {
            if expr_is_task_sleep(expr) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        if matches!(
            traversal::walk_stmts_until(
                &func.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut on_stmt,
                &mut on_expr
            ),
            TraversalControl::Stop
        ) {
            return true;
        }
    }

    for class in &module.classes {
        for method in &class.methods {
            let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
            let mut on_expr = |expr: &HirExpr| {
                if expr_is_task_sleep(expr) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            if matches!(
                traversal::walk_stmts_until(
                    &method.body,
                    TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                    &mut on_stmt,
                    &mut on_expr
                ),
                TraversalControl::Stop
            ) {
                return true;
            }
        }
    }

    false
}

pub(crate) fn type_contains_by(ty: &Type, predicate: fn(&Type) -> bool) -> bool {
    if predicate(ty) {
        return true;
    }

    match ty {
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Newtype { inner, .. }
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::Awaitable(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner) => type_contains_by(inner, predicate),
        Type::Dict(key, value)
        | Type::Result(key, value)
        | Type::Coroutine(key, value)
        | Type::Task(key, value)
        | Type::TaskResult(key, value)
        | Type::Select2(key, value)
        | Type::BlockingTask(key, value)
        | Type::JoinSet(key, value)
        | Type::AsyncIterator(key, value)
        | Type::AsyncGenerator(key, value) => {
            type_contains_by(key, predicate) || type_contains_by(value, predicate)
        }
        Type::Tuple(items)
        | Type::Template(items)
        | Type::Union(items)
        | Type::Intersection(items) => items.iter().any(|item| type_contains_by(item, predicate)),
        Type::Alias {
            type_args, body, ..
        } => {
            type_args.iter().any(|arg| type_contains_by(arg, predicate))
                || type_contains_by(body, predicate)
        }
        Type::Function(sig) | Type::AsyncFunction(sig) => {
            sig.params
                .iter()
                .any(|(_, param_ty, _)| type_contains_by(param_ty, predicate))
                || type_contains_by(&sig.return_type, predicate)
        }
        Type::Callable(params, _, ret) | Type::AsyncCallable(params, _, ret) => {
            params
                .iter()
                .any(|param| type_contains_by(param, predicate))
                || type_contains_by(ret, predicate)
        }
        Type::Class {
            fields, methods, ..
        } => {
            fields
                .iter()
                .any(|(_, field_ty)| type_contains_by(field_ty, predicate))
                || methods.iter().any(|(_, method_sig)| {
                    method_sig
                        .params
                        .iter()
                        .any(|(_, param_ty, _)| type_contains_by(param_ty, predicate))
                        || type_contains_by(&method_sig.return_type, predicate)
                })
        }
        _ => false,
    }
}

pub(crate) fn type_contains_failure(ty: &Type) -> bool {
    type_contains_by(ty, |candidate| matches!(candidate, Type::Failure(_)))
}

pub(crate) fn type_contains_timeout_result(ty: &Type) -> bool {
    type_contains_by(ty, |candidate| matches!(candidate, Type::TimeoutResult(_)))
}

pub(crate) fn type_contains_async_generator(ty: &Type) -> bool {
    type_contains_by(ty, |candidate| {
        matches!(candidate, Type::AsyncGenerator(_, _))
    })
}

pub(crate) fn type_contains_cancellation_error(ty: &Type) -> bool {
    type_contains_by(
        ty,
        |candidate| matches!(candidate, Type::Class { name, .. } if name == "CancellationError"),
    )
}

pub(crate) fn type_contains_async_exit_cause(ty: &Type) -> bool {
    type_contains_by(
        ty,
        |candidate| matches!(candidate, Type::Class { name, .. } if name == "AsyncExitCause"),
    )
}

pub(crate) fn module_uses_failure_type(module: &HirModule) -> bool {
    module.functions.iter().any(function_uses_failure_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_failure(field_ty))
                || class.methods.iter().any(function_uses_failure_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_failure(ty))
}

pub(crate) fn module_uses_cancellation_error_type(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_cancellation_error_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_cancellation_error(field_ty))
                || class
                    .methods
                    .iter()
                    .any(function_uses_cancellation_error_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_cancellation_error(ty))
}

pub(crate) fn module_uses_async_exit_cause_type(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_async_exit_cause_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_async_exit_cause(field_ty))
                || class
                    .methods
                    .iter()
                    .any(function_uses_async_exit_cause_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_async_exit_cause(ty))
}

pub(crate) fn module_uses_timeout_result_type(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_timeout_result_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_timeout_result(field_ty))
                || class.methods.iter().any(function_uses_timeout_result_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_timeout_result(ty))
}

pub(crate) fn module_uses_async_generator_type(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_async_generator_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_async_generator(field_ty))
                || class.methods.iter().any(function_uses_async_generator_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_async_generator(ty))
}

pub(crate) fn function_uses_cancellation_error_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_cancellation_error(&param.ty))
        || type_contains_cancellation_error(&func.return_type)
}

pub(crate) fn function_uses_async_exit_cause_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_async_exit_cause(&param.ty))
        || type_contains_async_exit_cause(&func.return_type)
}

pub(crate) fn function_uses_failure_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_failure(&param.ty))
        || type_contains_failure(&func.return_type)
}

pub(crate) fn function_uses_timeout_result_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_timeout_result(&param.ty))
        || type_contains_timeout_result(&func.return_type)
}

pub(crate) fn function_uses_async_generator_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_async_generator(&param.ty))
        || type_contains_async_generator(&func.return_type)
}

pub(crate) fn body_contains_await(body: &[HirStmt]) -> bool {
    let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
    let mut on_expr = |expr: &HirExpr| {
        if matches!(expr, HirExpr::Await { .. }) {
            TraversalControl::Stop
        } else {
            TraversalControl::Continue
        }
    };
    matches!(
        traversal::walk_stmts_until(
            body,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr
        ),
        TraversalControl::Stop
    )
}

pub(crate) fn module_uses_task_scope(module: &HirModule) -> bool {
    fn stmt_uses_task_scope_runtime(stmt: &HirStmt) -> bool {
        matches!(
            stmt,
            HirStmt::AsyncWith {
                kind: sifr_ir::HirAsyncWithKind::TaskScope
                    | sifr_ir::HirAsyncWithKind::TaskGroup { .. },
                ..
            }
        )
    }
    fn expr_uses_task_scope_runtime(expr: &HirExpr) -> bool {
        matches!(expr, HirExpr::Call { func, .. } if func == "__sifr_task_gather" || func == "__sifr_task_race" || func == "__sifr_task_select" || func == "__sifr_spawn_blocking_infallible" || func == "__sifr_spawn_blocking_result" || func == "__sifr_spawn_cpu_infallible" || func == "__sifr_spawn_cpu_result")
    }

    for func in &module.functions {
        let mut on_stmt = |stmt: &HirStmt| {
            if stmt_uses_task_scope_runtime(stmt) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        let mut on_expr = |expr: &HirExpr| {
            if expr_uses_task_scope_runtime(expr) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        if matches!(
            traversal::walk_stmts_until(
                &func.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut on_stmt,
                &mut on_expr
            ),
            TraversalControl::Stop
        ) {
            return true;
        }
    }

    for class in &module.classes {
        for method in &class.methods {
            let mut on_stmt = |stmt: &HirStmt| {
                if stmt_uses_task_scope_runtime(stmt) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            let mut on_expr = |expr: &HirExpr| {
                if expr_uses_task_scope_runtime(expr) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            if matches!(
                traversal::walk_stmts_until(
                    &method.body,
                    TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                    &mut on_stmt,
                    &mut on_expr
                ),
                TraversalControl::Stop
            ) {
                return true;
            }
        }
    }

    false
}

pub(crate) fn module_uses_spawn_cpu(module: &HirModule) -> bool {
    fn expr_uses_spawn_cpu_runtime(expr: &HirExpr) -> bool {
        matches!(expr, HirExpr::Call { func, .. } if func == "__sifr_spawn_cpu_infallible" || func == "__sifr_spawn_cpu_result")
    }

    for func in &module.functions {
        let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
        let mut on_expr = |expr: &HirExpr| {
            if expr_uses_spawn_cpu_runtime(expr) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        if matches!(
            traversal::walk_stmts_until(
                &func.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut on_stmt,
                &mut on_expr
            ),
            TraversalControl::Stop
        ) {
            return true;
        }
    }

    for class in &module.classes {
        for method in &class.methods {
            let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
            let mut on_expr = |expr: &HirExpr| {
                if expr_uses_spawn_cpu_runtime(expr) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            if matches!(
                traversal::walk_stmts_until(
                    &method.body,
                    TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                    &mut on_stmt,
                    &mut on_expr
                ),
                TraversalControl::Stop
            ) {
                return true;
            }
        }
    }

    false
}

pub(crate) fn module_uses_join_set(module: &HirModule) -> bool {
    fn expr_uses_join_set_runtime(expr: &HirExpr) -> bool {
        match expr {
            HirExpr::Call { func, .. } => func == "__sifr_join_set_new",
            HirExpr::MethodCall { method, .. } => matches!(
                method.as_str(),
                "__sifr_add_task"
                    | "__sifr_add_blocking_task"
                    | "__sifr_spawn_blocking"
                    | "__sifr_spawn_cpu"
                    | "__sifr_join_all"
                    | "__sifr_cancel_all"
            ),
            _ => false,
        }
    }

    for func in &module.functions {
        let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
        let mut on_expr = |expr: &HirExpr| {
            if expr_uses_join_set_runtime(expr) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        if matches!(
            traversal::walk_stmts_until(
                &func.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut on_stmt,
                &mut on_expr
            ),
            TraversalControl::Stop
        ) {
            return true;
        }
    }

    for class in &module.classes {
        for method in &class.methods {
            let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
            let mut on_expr = |expr: &HirExpr| {
                if expr_uses_join_set_runtime(expr) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            if matches!(
                traversal::walk_stmts_until(
                    &method.body,
                    TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                    &mut on_stmt,
                    &mut on_expr
                ),
                TraversalControl::Stop
            ) {
                return true;
            }
        }
    }

    false
}
