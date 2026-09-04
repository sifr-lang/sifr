use crate::hir_analysis::traversal::{self, TraversalConfig};
use sifr_ir::{HirAsyncWithKind, HirExpr, HirFunction, HirModule, HirStmt, PythonInteropEffect};
use sifr_type_system::Type;
use std::collections::HashSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ParallelRuntimeDemand {
    default_map: bool,
    default_try_map: bool,
    pool_map: bool,
    pool_try_map: bool,
}

impl ParallelRuntimeDemand {
    fn record_call(&mut self, name: &str) {
        match name {
            "__sifr_parallel_map" => self.default_map = true,
            "__sifr_parallel_try_map" => self.default_try_map = true,
            "__sifr_pool_map" => self.pool_map = true,
            "__sifr_pool_try_map" => self.pool_try_map = true,
            _ => {}
        }
    }

    fn merge(&mut self, other: &Self) {
        self.default_map |= other.default_map;
        self.default_try_map |= other.default_try_map;
        self.pool_map |= other.pool_map;
        self.pool_try_map |= other.pool_try_map;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    pub(crate) fn function_names(&self) -> HashSet<String> {
        [
            (self.default_map, "__sifr_parallel_map"),
            (self.default_try_map, "__sifr_parallel_try_map"),
            (self.pool_map, "__sifr_pool_map"),
            (self.pool_try_map, "__sifr_pool_try_map"),
        ]
        .into_iter()
        .filter(|(needed, _)| *needed)
        .map(|(_, name)| name.to_string())
        .collect()
    }
}

/// Complete runtime-support demand for one HIR module.
///
/// Code generation computes this value once and renders support exclusively
/// from it. Keeping the demand as data prevents independent scanners and
/// emitters from reintroducing overlapping runtime bodies.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RuntimeSupportDemand {
    pub(crate) failure_type: bool,
    pub(crate) cancellation_error_type: bool,
    pub(crate) async_exit_cause_type: bool,
    pub(crate) timeout_result_type: bool,
    pub(crate) async_generator: bool,
    pub(crate) sync_generator: bool,
    pub(crate) template: bool,
    pub(crate) task_sleep: bool,
    pub(crate) task_scope: bool,
    pub(crate) task_scope_offload: bool,
    pub(crate) task_scope_process: bool,
    pub(crate) task_scope_spawn_cpu: bool,
    pub(crate) join_set: bool,
    pub(crate) join_set_spawn_cpu: bool,
    pub(crate) spawn_cpu: bool,
    pub(crate) parallel: ParallelRuntimeDemand,
    pub(crate) async_python: bool,
    pub(crate) native_async_cleanup: bool,
}

impl RuntimeSupportDemand {
    pub(crate) fn for_module(module: &HirModule) -> Self {
        let mut demand = Self::default();
        for (_, ty, value) in &module.constants {
            demand.record_type(ty);
            demand.scan_expr(value);
        }
        for function in &module.functions {
            demand.scan_function(function);
        }
        for class in &module.classes {
            for (_, field_ty) in &class.fields {
                demand.record_type(field_ty);
            }
            for (_, default) in &class.field_defaults {
                demand.scan_expr(default);
            }
            for method in &class.methods {
                demand.scan_function(method);
            }
            for (_, operator) in &class.operator_impls {
                demand.scan_function(operator);
            }
        }
        demand
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        self.failure_type |= other.failure_type;
        self.cancellation_error_type |= other.cancellation_error_type;
        self.async_exit_cause_type |= other.async_exit_cause_type;
        self.timeout_result_type |= other.timeout_result_type;
        self.async_generator |= other.async_generator;
        self.sync_generator |= other.sync_generator;
        self.template |= other.template;
        self.task_sleep |= other.task_sleep;
        self.task_scope |= other.task_scope;
        self.task_scope_offload |= other.task_scope_offload;
        self.task_scope_process |= other.task_scope_process;
        self.task_scope_spawn_cpu |= other.task_scope_spawn_cpu;
        self.join_set |= other.join_set;
        self.join_set_spawn_cpu |= other.join_set_spawn_cpu;
        self.spawn_cpu |= other.spawn_cpu;
        self.parallel.merge(&other.parallel);
        self.async_python |= other.async_python;
        self.native_async_cleanup |= other.native_async_cleanup;
    }

    pub(crate) fn needs_worker_panic_hook(&self) -> bool {
        self.spawn_cpu
            || self.join_set_spawn_cpu
            || self.task_scope_spawn_cpu
            || !self.parallel.is_empty()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    fn scan_function(&mut self, function: &HirFunction) {
        for param in &function.params {
            self.record_type(&param.ty);
            if let Some(default) = &param.default {
                self.scan_expr(default);
            }
        }
        self.record_type(&function.return_type);
        self.async_python |= function
            .python_interop
            .iter()
            .any(|declaration| declaration.effect == PythonInteropEffect::Async);

        let returns_sync_iterator =
            matches!(function.return_type.resolve_alias(), Type::Iterator(_));
        let demand = std::cell::RefCell::new(self);
        let mut on_stmt = |stmt: &HirStmt| {
            let mut demand = demand.borrow_mut();
            demand.record_stmt(stmt, returns_sync_iterator);
            if let HirStmt::NestedFunction { func, .. } = stmt {
                demand.scan_function(func);
            }
        };
        let mut on_expr = |expr: &HirExpr| demand.borrow_mut().record_expr(expr);
        traversal::walk_stmts(
            &function.body,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
    }

    fn scan_expr(&mut self, expr: &HirExpr) {
        traversal::walk_expr(expr, &mut |expr| self.record_expr(expr));
    }

    fn record_stmt(&mut self, stmt: &HirStmt, returns_sync_iterator: bool) {
        match stmt {
            HirStmt::Yield { .. } if returns_sync_iterator => self.sync_generator = true,
            HirStmt::AsyncWith { kind, .. } => match kind {
                HirAsyncWithKind::TaskScope | HirAsyncWithKind::TaskGroup { .. } => {
                    self.task_scope = true;
                }
                HirAsyncWithKind::UserDefined { .. } => self.native_async_cleanup = true,
                HirAsyncWithKind::Python { .. } => self.async_python = true,
                HirAsyncWithKind::TaskTimeout { .. } => {}
            },
            HirStmt::Let { ty, .. } => self.record_type(ty),
            HirStmt::For { target_ty, .. } => self.record_type(target_ty),
            HirStmt::AsyncFor {
                target_ty,
                iter_error_ty,
                close_error_ty,
                active_error_ty,
                ..
            } => {
                self.native_async_cleanup |= close_error_ty.is_some();
                self.record_type(target_ty);
                self.record_type(iter_error_ty);
                if let Some(close_error_ty) = close_error_ty {
                    self.record_type(close_error_ty);
                }
                self.record_type(active_error_ty);
            }
            HirStmt::TryExcept {
                body_error_types,
                handlers,
                ..
            } => {
                for ty in body_error_types {
                    self.record_type(ty);
                }
                for handler in handlers {
                    if let Some(ty) = &handler.error_resolved_type {
                        self.record_type(ty);
                    }
                }
            }
            _ => {}
        }
    }

    fn record_expr(&mut self, expr: &HirExpr) {
        self.record_type(expr.ty());
        match expr {
            HirExpr::TemplateString(_) => self.template = true,
            HirExpr::Call { func, .. } => match func.as_str() {
                "__sifr_task_sleep" => self.task_sleep = true,
                "__sifr_task_gather"
                | "__sifr_task_race"
                | "__sifr_task_select"
                | "__sifr_spawn_blocking_infallible"
                | "__sifr_spawn_blocking_result" => self.task_scope = true,
                "__sifr_spawn_cpu_infallible" | "__sifr_spawn_cpu_result" => {
                    self.task_scope = true;
                    self.spawn_cpu = true;
                }
                "__sifr_join_set_new" => self.join_set = true,
                name => self.parallel.record_call(name),
            },
            HirExpr::MethodCall { method, .. } => match method.as_str() {
                "__sifr_add_task"
                | "__sifr_add_blocking_task"
                | "__sifr_spawn_blocking"
                | "__sifr_join_all"
                | "__sifr_cancel_all" => self.join_set = true,
                "__sifr_spawn_cpu" => {
                    self.join_set = true;
                    self.join_set_spawn_cpu = true;
                }
                "__sifr_scope_spawn_blocking_infallible" | "__sifr_scope_spawn_blocking_result" => {
                    self.task_scope = true;
                    self.task_scope_offload = true;
                }
                "__sifr_scope_spawn_cpu_infallible" | "__sifr_scope_spawn_cpu_result" => {
                    self.task_scope = true;
                    self.task_scope_offload = true;
                    self.task_scope_spawn_cpu = true;
                }
                "__sifr_scope_spawn_process" => {
                    self.task_scope = true;
                    self.task_scope_process = true;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn record_type(&mut self, ty: &Type) {
        match ty {
            Type::Failure(_) => self.failure_type = true,
            Type::TimeoutResult(_) => self.timeout_result_type = true,
            Type::AsyncGenerator(_, _) => self.async_generator = true,
            Type::Template(_) => self.template = true,
            Type::Class { name, .. } if name == "CancellationError" => {
                self.cancellation_error_type = true;
            }
            Type::Class { name, .. } if name == "AsyncExitCause" => {
                self.async_exit_cause_type = true;
            }
            _ => {}
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
            | Type::PythonDlpackTensor(inner) => self.record_type(inner),
            Type::Dict(left, right)
            | Type::Result(left, right)
            | Type::Coroutine(left, right)
            | Type::Task(left, right)
            | Type::TaskResult(left, right)
            | Type::Select2(left, right)
            | Type::BlockingTask(left, right)
            | Type::JoinSet(left, right)
            | Type::AsyncIterator(left, right)
            | Type::AsyncGenerator(left, right) => {
                self.record_type(left);
                self.record_type(right);
            }
            Type::Tuple(items)
            | Type::Template(items)
            | Type::Union(items)
            | Type::Intersection(items) => {
                for item in items {
                    self.record_type(item);
                }
            }
            Type::Alias {
                type_args, body, ..
            } => {
                for argument in type_args {
                    self.record_type(argument);
                }
                self.record_type(body);
            }
            Type::Function(signature) | Type::AsyncFunction(signature) => {
                for (_, parameter, _) in &signature.params {
                    self.record_type(parameter);
                }
                self.record_type(&signature.return_type);
            }
            Type::Callable(parameters, _, result) | Type::AsyncCallable(parameters, _, result) => {
                for parameter in parameters {
                    self.record_type(parameter);
                }
                self.record_type(result);
            }
            Type::Class {
                fields, methods, ..
            } => {
                for (_, field) in fields {
                    self.record_type(field);
                }
                for (_, signature) in methods {
                    for (_, parameter, _) in &signature.params {
                        self.record_type(parameter);
                    }
                    self.record_type(&signature.return_type);
                }
            }
            Type::StructuralRecord(record) => {
                for field in record.fields() {
                    self.record_type(field.ty());
                }
            }
            _ => {}
        }
    }
}
