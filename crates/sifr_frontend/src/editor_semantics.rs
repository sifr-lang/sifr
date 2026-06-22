use crate::ModuleId;
use ruff_text_size::{Ranged, TextRange};
use sifr_lowering::{ExternalDefs, HirExpr, HirFunction, HirModule, HirStmt};
use sifr_python_ast::{Expr, Stmt};
use sifr_type_system::{FunctionType, Type};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorSemanticView {
    pub entries: Vec<EditorSemanticEntry>,
    pub calls: Vec<EditorCallExpressionView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorSemanticEntry {
    pub range: TextRange,
    pub name: String,
    pub detail: String,
    pub kind: EditorSemanticKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorSemanticKind {
    Function,
    Binding,
    Parameter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCallableSignatureView {
    pub label: String,
    pub parameters: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCallExpressionView {
    pub callee_range: TextRange,
    pub call_range: TextRange,
    pub argument_ranges: Vec<TextRange>,
    pub signature: EditorCallableSignatureView,
}

pub(super) fn editor_semantics_from_module(
    module_id: ModuleId,
    module_name: &str,
    source: &str,
    suite: &[Stmt],
    module: &HirModule,
    external_defs: &ExternalDefs,
) -> EditorSemanticView {
    let signatures = callable_signatures(module_name, module, external_defs);
    let mut collector = EditorSemanticCollector {
        signatures,
        source: source.to_string(),
        view: EditorSemanticView::default(),
    };
    for stmt in suite {
        match stmt {
            Stmt::ImportFrom(import_from) => {
                collector.import_from(import_from);
            }
            Stmt::FunctionDef(function) => {
                if let Some(hir_function) = module
                    .functions
                    .iter()
                    .find(|candidate| candidate.name == function.name.as_str())
                {
                    collector.function(function, hir_function);
                }
            }
            _ => {}
        }
    }
    collector.sort(module_id);
    collector.view
}

struct EditorSemanticCollector {
    signatures: BTreeMap<String, EditorCallableSignatureView>,
    source: String,
    view: EditorSemanticView,
}

impl EditorSemanticCollector {
    fn import_from(&mut self, import_from: &sifr_python_ast::StmtImportFrom) {
        for alias in &import_from.names {
            let local_name = alias
                .asname
                .as_ref()
                .map_or_else(|| alias.name.to_string(), ToString::to_string);
            if let Some(signature) = self.signatures.get(&local_name).cloned() {
                let range = alias
                    .asname
                    .as_ref()
                    .map_or_else(|| alias.name.range(), Ranged::range);
                self.entry(
                    range,
                    local_name,
                    signature.label,
                    EditorSemanticKind::Function,
                );
            }
        }
    }

    fn function(&mut self, function: &sifr_python_ast::StmtFunctionDef, hir: &HirFunction) {
        if let Some(signature) = self.signatures.get(&hir.name).cloned() {
            self.entry(
                function.name.range(),
                hir.name.clone(),
                signature.label,
                EditorSemanticKind::Function,
            );
        }
        for parameter in function.parameters.iter_non_variadic_params() {
            if let Some(hir_param) = hir
                .params
                .iter()
                .find(|candidate| candidate.name == parameter.parameter.name.as_str())
            {
                self.entry(
                    parameter.parameter.name.range(),
                    hir_param.name.clone(),
                    binding_detail(&hir_param.name, &hir_param.ty),
                    EditorSemanticKind::Parameter,
                );
            }
        }
        for (stmt, hir_stmt) in function.body.iter().zip(&hir.body) {
            self.stmt(stmt, hir_stmt);
        }
        for stmt in &function.body {
            self.ast_call_semantics_from_stmt(stmt);
        }
        let mut binding_details = collect_hir_binding_types(&hir.body)
            .into_iter()
            .map(|(name, ty)| {
                let detail = binding_detail(&name, &ty);
                (name, detail)
            })
            .collect::<BTreeMap<_, _>>();
        collect_ast_annotation_details(&function.body, &self.source, &mut binding_details);
        for stmt in &function.body {
            self.ast_names_from_bindings(stmt, &binding_details);
        }
    }

    fn stmt(&mut self, stmt: &Stmt, hir: &HirStmt) {
        match (stmt, hir) {
            (
                Stmt::AnnAssign(assign),
                HirStmt::Let {
                    name, ty, value, ..
                },
            ) => {
                self.binding_target(&assign.target, name, ty);
                if let Some(value_expr) = &assign.value {
                    self.expr(value_expr, value);
                }
            }
            (
                Stmt::Assign(assign),
                HirStmt::Let {
                    name, ty, value, ..
                },
            ) => {
                if let Some(target) = assign.targets.first() {
                    self.binding_target(target, name, ty);
                }
                self.expr(&assign.value, value);
            }
            (Stmt::Assign(assign), HirStmt::Assign { name, value }) => {
                if let Some(target) = assign.targets.first() {
                    self.binding_target(target, name, value.ty());
                }
                self.expr(&assign.value, value);
            }
            (Stmt::Return(ret), HirStmt::Return { value }) => {
                if let (Some(expr), Some(hir_expr)) = (&ret.value, value) {
                    self.expr(expr, hir_expr);
                }
            }
            (Stmt::Expr(expr_stmt), HirStmt::Expr { expr }) => {
                self.expr(&expr_stmt.value, expr);
            }
            (Stmt::Try(try_stmt), HirStmt::TryExcept { body, handlers, .. }) => {
                for (stmt, hir_stmt) in try_stmt.body.iter().zip(body) {
                    self.stmt(stmt, hir_stmt);
                }
                for (handler, hir_handler) in try_stmt.handlers.iter().zip(handlers) {
                    let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                    for (stmt, hir_stmt) in handler.body.iter().zip(&hir_handler.body) {
                        self.stmt(stmt, hir_stmt);
                    }
                }
            }
            (
                Stmt::If(if_stmt),
                HirStmt::If {
                    condition,
                    then_body,
                    elif_clauses,
                    else_body,
                },
            ) => {
                self.expr(&if_stmt.test, condition);
                for (stmt, hir_stmt) in if_stmt.body.iter().zip(then_body) {
                    self.stmt(stmt, hir_stmt);
                }
                for (clause, (hir_condition, hir_body)) in
                    if_stmt.elif_else_clauses.iter().zip(elif_clauses)
                {
                    if let Some(test) = &clause.test {
                        self.expr(test, hir_condition);
                    }
                    for (stmt, hir_stmt) in clause.body.iter().zip(hir_body) {
                        self.stmt(stmt, hir_stmt);
                    }
                }
                if let Some(last_clause) = if_stmt.elif_else_clauses.last() {
                    if last_clause.test.is_none() {
                        if let Some(hir_else) = else_body {
                            for (stmt, hir_stmt) in last_clause.body.iter().zip(hir_else) {
                                self.stmt(stmt, hir_stmt);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn expr(&mut self, expr: &Expr, hir: &HirExpr) {
        match (expr, hir) {
            (Expr::Name(name), HirExpr::Name { name: hir_name, ty })
                if name.id.as_str() == hir_name =>
            {
                self.entry(
                    name.range(),
                    hir_name.clone(),
                    binding_detail(hir_name, ty),
                    EditorSemanticKind::Binding,
                );
            }
            (Expr::Call(call), HirExpr::Call { func, args, .. }) => {
                if let Expr::Name(name) = call.func.as_ref() {
                    if let Some(signature) = self.signatures.get(func).cloned() {
                        self.entry(
                            name.range(),
                            func.clone(),
                            signature.label.clone(),
                            EditorSemanticKind::Function,
                        );
                        self.call(EditorCallExpressionView {
                            callee_range: name.range(),
                            call_range: call.range(),
                            argument_ranges: call_argument_ranges(call),
                            signature,
                        });
                    }
                }
                for (arg, hir_arg) in call.arguments.args.iter().zip(args) {
                    self.expr(arg, hir_arg);
                }
            }
            (Expr::BinOp(binop), HirExpr::BinOp { left, right, .. }) => {
                self.expr(&binop.left, left);
                self.expr(&binop.right, right);
            }
            (
                Expr::Compare(compare),
                HirExpr::Compare {
                    left, comparators, ..
                },
            ) => {
                self.expr(&compare.left, left);
                for (expr, hir_expr) in compare.comparators.iter().zip(comparators) {
                    self.expr(expr, hir_expr);
                }
            }
            (
                Expr::If(if_expr),
                HirExpr::IfExpr {
                    condition,
                    then_expr,
                    else_expr,
                    ..
                },
            ) => {
                self.expr(&if_expr.test, condition);
                self.expr(&if_expr.body, then_expr);
                self.expr(&if_expr.orelse, else_expr);
            }
            _ => {}
        }
    }

    fn ast_call_semantics_from_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::AnnAssign(assign) => {
                if let Some(value) = &assign.value {
                    self.ast_call_semantics_from_expr(value);
                }
            }
            Stmt::Assign(assign) => {
                self.ast_call_semantics_from_expr(&assign.value);
            }
            Stmt::Return(ret) => {
                if let Some(value) = &ret.value {
                    self.ast_call_semantics_from_expr(value);
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.ast_call_semantics_from_expr(&expr_stmt.value);
            }
            Stmt::Try(try_stmt) => {
                for stmt in &try_stmt.body {
                    self.ast_call_semantics_from_stmt(stmt);
                }
                for handler in &try_stmt.handlers {
                    let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                    for stmt in &handler.body {
                        self.ast_call_semantics_from_stmt(stmt);
                    }
                }
            }
            Stmt::If(if_stmt) => {
                self.ast_call_semantics_from_expr(&if_stmt.test);
                for stmt in &if_stmt.body {
                    self.ast_call_semantics_from_stmt(stmt);
                }
                for clause in &if_stmt.elif_else_clauses {
                    if let Some(test) = &clause.test {
                        self.ast_call_semantics_from_expr(test);
                    }
                    for stmt in &clause.body {
                        self.ast_call_semantics_from_stmt(stmt);
                    }
                }
            }
            _ => {}
        }
    }

    fn ast_call_semantics_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Call(call) => {
                if let Expr::Name(name) = call.func.as_ref() {
                    if let Some(signature) = self.signatures.get(name.id.as_str()).cloned() {
                        self.entry(
                            name.range(),
                            name.id.to_string(),
                            signature.label.clone(),
                            EditorSemanticKind::Function,
                        );
                        self.call(EditorCallExpressionView {
                            callee_range: name.range(),
                            call_range: call.range(),
                            argument_ranges: call_argument_ranges(call),
                            signature,
                        });
                    }
                } else {
                    self.ast_call_semantics_from_expr(&call.func);
                }
                for arg in &call.arguments.args {
                    self.ast_call_semantics_from_expr(arg);
                }
            }
            Expr::BinOp(binop) => {
                self.ast_call_semantics_from_expr(&binop.left);
                self.ast_call_semantics_from_expr(&binop.right);
            }
            Expr::Compare(compare) => {
                self.ast_call_semantics_from_expr(&compare.left);
                for comparator in &compare.comparators {
                    self.ast_call_semantics_from_expr(comparator);
                }
            }
            Expr::If(if_expr) => {
                self.ast_call_semantics_from_expr(&if_expr.test);
                self.ast_call_semantics_from_expr(&if_expr.body);
                self.ast_call_semantics_from_expr(&if_expr.orelse);
            }
            _ => {}
        }
    }

    fn binding_target(&mut self, target: &Expr, name: &str, ty: &Type) {
        if let Expr::Name(target_name) = target {
            if target_name.id.as_str() == name {
                self.entry(
                    target_name.range(),
                    name.to_string(),
                    binding_detail(name, ty),
                    EditorSemanticKind::Binding,
                );
            }
        }
    }

    fn ast_names_from_bindings(&mut self, stmt: &Stmt, binding_details: &BTreeMap<String, String>) {
        match stmt {
            Stmt::AnnAssign(assign) => {
                self.ast_expr_names(&assign.target, binding_details);
                if let Some(value) = &assign.value {
                    self.ast_expr_names(value, binding_details);
                }
            }
            Stmt::Assign(assign) => {
                for target in &assign.targets {
                    self.ast_expr_names(target, binding_details);
                }
                self.ast_expr_names(&assign.value, binding_details);
            }
            Stmt::Return(ret) => {
                if let Some(value) = &ret.value {
                    self.ast_expr_names(value, binding_details);
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.ast_expr_names(&expr_stmt.value, binding_details);
            }
            Stmt::Try(try_stmt) => {
                for stmt in &try_stmt.body {
                    self.ast_names_from_bindings(stmt, binding_details);
                }
                for handler in &try_stmt.handlers {
                    let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                    for stmt in &handler.body {
                        self.ast_names_from_bindings(stmt, binding_details);
                    }
                }
            }
            Stmt::If(if_stmt) => {
                self.ast_expr_names(&if_stmt.test, binding_details);
                for stmt in &if_stmt.body {
                    self.ast_names_from_bindings(stmt, binding_details);
                }
                for clause in &if_stmt.elif_else_clauses {
                    if let Some(test) = &clause.test {
                        self.ast_expr_names(test, binding_details);
                    }
                    for stmt in &clause.body {
                        self.ast_names_from_bindings(stmt, binding_details);
                    }
                }
            }
            _ => {}
        }
    }

    fn ast_expr_names(&mut self, expr: &Expr, binding_details: &BTreeMap<String, String>) {
        match expr {
            Expr::Name(name) => {
                if let Some(detail) = binding_details.get(name.id.as_str()) {
                    self.entry(
                        name.range(),
                        name.id.to_string(),
                        detail.clone(),
                        EditorSemanticKind::Binding,
                    );
                }
            }
            Expr::Call(call) => {
                for arg in &call.arguments.args {
                    self.ast_expr_names(arg, binding_details);
                }
            }
            Expr::BinOp(binop) => {
                self.ast_expr_names(&binop.left, binding_details);
                self.ast_expr_names(&binop.right, binding_details);
            }
            Expr::Compare(compare) => {
                self.ast_expr_names(&compare.left, binding_details);
                for comparator in &compare.comparators {
                    self.ast_expr_names(comparator, binding_details);
                }
            }
            _ => {}
        }
    }

    fn entry(&mut self, range: TextRange, name: String, detail: String, kind: EditorSemanticKind) {
        if self
            .view
            .entries
            .iter()
            .any(|entry| entry.range == range && entry.name == name && entry.detail == detail)
        {
            return;
        }
        self.view.entries.push(EditorSemanticEntry {
            range,
            name,
            detail,
            kind,
        });
    }

    fn call(&mut self, call: EditorCallExpressionView) {
        if self.view.calls.iter().any(|existing| {
            existing.callee_range == call.callee_range && existing.call_range == call.call_range
        }) {
            return;
        }
        self.view.calls.push(call);
    }

    fn sort(&mut self, _module: ModuleId) {
        self.view.entries.sort_by_key(|entry| {
            (
                entry.range.start(),
                entry.range.len(),
                entry.name.clone(),
                entry.detail.clone(),
            )
        });
        self.view
            .calls
            .sort_by_key(|call| (call.call_range.start(), call.call_range.len()));
    }
}

fn callable_signatures(
    module_name: &str,
    module: &HirModule,
    external_defs: &ExternalDefs,
) -> BTreeMap<String, EditorCallableSignatureView> {
    let mut signatures = BTreeMap::new();
    if let Some(exports) = external_defs.functions.get(module_name) {
        for (name, function_type) in exports {
            signatures.insert(
                name.clone(),
                signature_from_function_type(name, function_type),
            );
        }
    }
    for function in &module.functions {
        signatures.entry(function.name.clone()).or_insert_with(|| {
            let function_type = FunctionType {
                params: function
                    .params
                    .iter()
                    .map(|param| (param.name.clone(), param.ty.clone(), param.convention))
                    .collect(),
                return_type: Box::new(function.return_type.clone()),
            };
            signature_from_function_type(&function.name, &function_type)
        });
    }
    signatures
}

fn signature_from_function_type(
    name: &str,
    function_type: &FunctionType,
) -> EditorCallableSignatureView {
    let parameters = function_type
        .params
        .iter()
        .map(|(param_name, ty, _)| binding_detail(param_name, ty))
        .collect::<Vec<_>>();
    EditorCallableSignatureView {
        label: format!(
            "{}({}) -> {}",
            name,
            parameters.join(", "),
            function_type.return_type.display_name()
        ),
        parameters,
    }
}

fn binding_detail(name: &str, ty: &Type) -> String {
    format!("{}: {}", name, ty.display_name())
}

fn collect_ast_annotation_details(
    stmts: &[Stmt],
    source: &str,
    details: &mut BTreeMap<String, String>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::AnnAssign(assign) => {
                if let Expr::Name(name) = assign.target.as_ref() {
                    if let Some(annotation) =
                        source_text_for_range(source, assign.annotation.range())
                    {
                        details
                            .entry(name.id.to_string())
                            .or_insert_with(|| format!("{}: {annotation}", name.id));
                    }
                }
            }
            Stmt::Try(try_stmt) => {
                collect_ast_annotation_details(&try_stmt.body, source, details);
                for handler in &try_stmt.handlers {
                    let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                    collect_ast_annotation_details(&handler.body, source, details);
                }
            }
            Stmt::If(if_stmt) => {
                collect_ast_annotation_details(&if_stmt.body, source, details);
                for clause in &if_stmt.elif_else_clauses {
                    collect_ast_annotation_details(&clause.body, source, details);
                }
            }
            _ => {}
        }
    }
}

fn source_text_for_range(source: &str, range: TextRange) -> Option<String> {
    let start = usize::try_from(range.start().to_u32()).ok()?;
    let end = usize::try_from(range.end().to_u32()).ok()?;
    Some(source.get(start..end)?.trim().to_string())
}

fn collect_hir_binding_types(stmts: &[HirStmt]) -> BTreeMap<String, Type> {
    let mut bindings = BTreeMap::new();
    for stmt in stmts {
        match stmt {
            HirStmt::Let { name, ty, .. } => {
                bindings.insert(name.clone(), ty.clone());
            }
            HirStmt::For {
                target,
                target_ty,
                body,
                else_body,
                ..
            }
            | HirStmt::AsyncFor {
                target,
                target_ty,
                body,
                else_body,
                ..
            } => {
                bindings.insert(target.clone(), target_ty.clone());
                bindings.extend(collect_hir_binding_types(body));
                if let Some(else_body) = else_body {
                    bindings.extend(collect_hir_binding_types(else_body));
                }
            }
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                bindings.extend(collect_hir_binding_types(then_body));
                for (_, body) in elif_clauses {
                    bindings.extend(collect_hir_binding_types(body));
                }
                if let Some(else_body) = else_body {
                    bindings.extend(collect_hir_binding_types(else_body));
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                bindings.extend(collect_hir_binding_types(body));
                for handler in handlers {
                    bindings.extend(collect_hir_binding_types(&handler.body));
                }
            }
            _ => {}
        }
    }
    bindings
}

fn call_argument_ranges(call: &sifr_python_ast::ExprCall) -> Vec<TextRange> {
    let mut ranges = call
        .arguments
        .args
        .iter()
        .map(Ranged::range)
        .collect::<Vec<_>>();
    ranges.extend(call.arguments.keywords.iter().map(Ranged::range));
    ranges
}
