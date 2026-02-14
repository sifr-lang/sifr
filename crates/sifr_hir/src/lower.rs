//! AST to HIR lowering with type checking and name resolution.

use sifr_python_ast::*;
use sifr_type_system::{
    Type, FunctionType,
    type_check_binary_op, type_check_unary_op, type_check_comparison, type_check_bool_op,
};
use sifr_type_system::infer::resolve_type_annotation;
use crate::hir_nodes::*;
use crate::scope::Scope;
use std::collections::HashMap;

/// Errors produced during lowering.
#[derive(Debug, Clone)]
pub struct LoweringError {
    pub message: String,
    pub line: Option<u32>,
    pub col: Option<u32>,
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(line), Some(col)) = (self.line, self.col) {
            write!(f, "{}:{}: {}", line, col, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// The lowering context that tracks state during AST->HIR conversion.
struct LowerCtx {
    /// Function signatures (name -> type)
    functions: HashMap<String, FunctionType>,
    /// Current scope for name resolution
    scope: Scope,
    /// Collected errors
    errors: Vec<LoweringError>,
    /// Loop nesting depth (for break/continue validation)
    loop_depth: usize,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            scope: Scope::new(),
            errors: Vec::new(),
            loop_depth: 0,
        }
    }

    fn error(&mut self, message: String) {
        self.errors.push(LoweringError {
            message,
            line: None,
            col: None,
        });
    }

    fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }
}

/// Lower a parsed module AST into a typed HIR module.
pub fn lower_module(stmts: &[Stmt]) -> Result<HirModule, Vec<LoweringError>> {
    let mut ctx = LowerCtx::new();

    // Register built-in functions
    register_builtins(&mut ctx);

    // First pass: collect all function signatures
    for stmt in stmts {
        if let Stmt::FunctionDef(func) = stmt {
            if let Some(ft) = extract_function_type(func, &mut ctx) {
                ctx.functions.insert(func.name.to_string(), ft);
            }
        }
    }

    // Second pass: lower function bodies
    let mut functions = Vec::new();
    for stmt in stmts {
        if let Stmt::FunctionDef(func) = stmt {
            if let Some(hir_func) = lower_function(func, &mut ctx) {
                functions.push(hir_func);
            }
        }
    }

    if ctx.errors.is_empty() {
        Ok(HirModule { functions })
    } else {
        Err(ctx.errors)
    }
}

fn register_builtins(ctx: &mut LowerCtx) {
    // print() accepts any single argument and returns None
    ctx.functions.insert(
        "print".to_string(),
        FunctionType {
            params: vec![("value".to_string(), Type::Any)],
            return_type: Box::new(Type::None),
        },
    );
}

fn extract_function_type(func: &StmtFunctionDef, ctx: &mut LowerCtx) -> Option<FunctionType> {
    let mut params = Vec::new();

    for param in &func.parameters.args {
        let name = param.parameter.name.to_string();
        let ty = if let Some(annotation) = &param.parameter.annotation {
            resolve_annotation_expr(annotation, ctx)
        } else {
            ctx.error(format!(
                "parameter '{}' in function '{}' is missing a type annotation",
                name, func.name
            ));
            Type::Any
        };
        params.push((name, ty));
    }

    let return_type = if let Some(returns) = &func.returns {
        resolve_annotation_expr(returns, ctx)
    } else {
        Type::None // default return type
    };

    Some(FunctionType {
        params,
        return_type: Box::new(return_type),
    })
}

fn resolve_annotation_expr(expr: &Expr, ctx: &mut LowerCtx) -> Type {
    match expr {
        Expr::Name(name) => {
            resolve_type_annotation(&name.id).unwrap_or_else(|| {
                ctx.error(format!("unknown type: '{}'", name.id));
                Type::Any
            })
        }
        Expr::NoneLiteral(_) => Type::None,
        Expr::Subscript(sub) => {
            // Handle generic type annotations: list[int], dict[str, int], tuple[int, str]
            let base_name = match sub.value.as_ref() {
                Expr::Name(n) => n.id.to_string(),
                _ => {
                    ctx.error("unsupported type annotation base".to_string());
                    return Type::Any;
                }
            };
            match base_name.as_str() {
                "list" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::List(Box::new(elem_ty))
                }
                "dict" => {
                    // dict[K, V] -- the slice is a Tuple expression
                    match sub.slice.as_ref() {
                        Expr::Tuple(tuple) => {
                            if tuple.elts.len() != 2 {
                                ctx.error("dict type annotation requires exactly 2 type parameters".to_string());
                                return Type::Any;
                            }
                            let key_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                            let val_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                            Type::Dict(Box::new(key_ty), Box::new(val_ty))
                        }
                        _ => {
                            ctx.error("dict type annotation requires [K, V] syntax".to_string());
                            Type::Any
                        }
                    }
                }
                "tuple" => {
                    // tuple[A, B, ...] -- the slice is a Tuple expression
                    match sub.slice.as_ref() {
                        Expr::Tuple(tuple) => {
                            let elem_types: Vec<Type> = tuple.elts.iter()
                                .map(|e| resolve_annotation_expr(e, ctx))
                                .collect();
                            Type::Tuple(elem_types)
                        }
                        _ => {
                            // Single-element tuple: tuple[int]
                            let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                            Type::Tuple(vec![elem_ty])
                        }
                    }
                }
                _ => {
                    ctx.error(format!("unknown generic type: '{}'", base_name));
                    Type::Any
                }
            }
        }
        _ => {
            ctx.error("unsupported type annotation expression".to_string());
            Type::Any
        }
    }
}

fn lower_function(func: &StmtFunctionDef, ctx: &mut LowerCtx) -> Option<HirFunction> {
    let ft = ctx.functions.get(&func.name.to_string())?.clone();

    ctx.scope.push();

    // Define parameters in scope
    let mut params = Vec::new();
    for (name, ty) in &ft.params {
        ctx.scope.define(name.clone(), ty.clone());
        params.push(HirParam {
            name: name.clone(),
            ty: ty.clone(),
        });
    }

    // Lower body
    let body = lower_stmts(&func.body, &ft, ctx);

    ctx.scope.pop();

    Some(HirFunction {
        name: func.name.to_string(),
        params,
        return_type: *ft.return_type,
        body,
    })
}

fn lower_stmts(stmts: &[Stmt], func_type: &FunctionType, ctx: &mut LowerCtx) -> Vec<HirStmt> {
    let mut result = Vec::new();
    for stmt in stmts {
        if let Some(hir_stmt) = lower_stmt(stmt, func_type, ctx) {
            result.push(hir_stmt);
        }
    }
    result
}

fn lower_stmt(stmt: &Stmt, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    match stmt {
        Stmt::AnnAssign(ann) => lower_ann_assign(ann, ctx),
        Stmt::Assign(assign) => lower_assign(assign, ctx),
        Stmt::Return(ret) => lower_return(ret, func_type, ctx),
        Stmt::Expr(expr_stmt) => {
            let expr = lower_expr(&expr_stmt.value, ctx)?;
            Some(HirStmt::Expr { expr })
        }
        Stmt::If(if_stmt) => lower_if(if_stmt, func_type, ctx),
        Stmt::While(while_stmt) => lower_while(while_stmt, func_type, ctx),
        Stmt::For(for_stmt) => lower_for(for_stmt, func_type, ctx),
        Stmt::Break(_) => {
            if !ctx.in_loop() {
                ctx.error("'break' outside of loop".to_string());
                return None;
            }
            Some(HirStmt::Break)
        }
        Stmt::Continue(_) => {
            if !ctx.in_loop() {
                ctx.error("'continue' outside of loop".to_string());
                return None;
            }
            Some(HirStmt::Continue)
        }
        Stmt::Pass(_) => Some(HirStmt::Pass),
        _ => {
            ctx.error("unsupported statement type".to_string());
            None
        }
    }
}

fn lower_ann_assign(ann: &StmtAnnAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let name = match ann.target.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("annotated assignment target must be a simple name".to_string());
            return None;
        }
    };

    let declared_type = resolve_annotation_expr(&ann.annotation, ctx);

    let value = if let Some(val) = &ann.value {
        let expr = lower_expr(val, ctx)?;
        // Type check: value must be assignable to declared type
        let expr_ty = expr.ty().clone();
        if !expr_ty.is_assignable_to(&declared_type) {
            ctx.error(format!(
                "type mismatch: expected '{}', got '{}'",
                declared_type.display_name(),
                expr_ty.display_name()
            ));
        }
        expr
    } else {
        ctx.error(format!("variable '{}' must be initialized", name));
        return None;
    };

    ctx.scope.define(name.clone(), declared_type.clone());

    Some(HirStmt::Let {
        name,
        ty: declared_type,
        value,
        is_mutable: true,
    })
}

fn lower_assign(assign: &StmtAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    if assign.targets.len() != 1 {
        ctx.error("multiple assignment targets not supported yet".to_string());
        return None;
    }

    // Handle tuple unpacking: a, b = expr
    if let Expr::Tuple(tuple) = &assign.targets[0] {
        return lower_tuple_unpack_assign(tuple, &assign.value, ctx);
    }

    let name = match &assign.targets[0] {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("assignment target must be a simple name".to_string());
            return None;
        }
    };

    let value = lower_expr(&assign.value, ctx)?;
    let value_ty = value.ty().clone();

    // Check if variable already exists
    if let Some(info) = ctx.scope.lookup(&name) {
        // Reassignment: check type compatibility
        if !value_ty.is_assignable_to(&info.ty) {
            ctx.error(format!(
                "type mismatch: cannot assign '{}' to variable '{}' of type '{}'",
                value_ty.display_name(),
                name,
                info.ty.display_name()
            ));
        }
        // Reset moved state on reassignment
        ctx.scope.reset_moved(&name);
        Some(HirStmt::Assign { name, value })
    } else {
        // New variable (type inferred)
        ctx.scope.define(name.clone(), value_ty.clone());
        Some(HirStmt::Let {
            name,
            ty: value_ty,
            value,
            is_mutable: true,
        })
    }
}

fn lower_return(ret: &StmtReturn, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let value = if let Some(val) = &ret.value {
        let expr = lower_expr(val, ctx)?;
        let expr_ty = expr.ty().clone();
        if !expr_ty.is_assignable_to(&func_type.return_type) {
            ctx.error(format!(
                "return type mismatch: expected '{}', got '{}'",
                func_type.return_type.display_name(),
                expr_ty.display_name()
            ));
        }
        Some(expr)
    } else {
        if *func_type.return_type != Type::None {
            ctx.error(format!(
                "function expects return type '{}', but returns nothing",
                func_type.return_type.display_name()
            ));
        }
        None
    };

    Some(HirStmt::Return { value })
}

fn lower_if(if_stmt: &StmtIf, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let condition = lower_expr(&if_stmt.test, ctx)?;

    ctx.scope.push();
    let then_body = lower_stmts(&if_stmt.body, func_type, ctx);
    ctx.scope.pop();

    let mut elif_clauses = Vec::new();
    for clause in &if_stmt.elif_else_clauses {
        if let Some(test) = &clause.test {
            let cond = lower_expr(test, ctx)?;
            ctx.scope.push();
            let body = lower_stmts(&clause.body, func_type, ctx);
            ctx.scope.pop();
            elif_clauses.push((cond, body));
        }
    }

    let else_body = if_stmt.elif_else_clauses.iter().find(|c| c.test.is_none()).map(|clause| {
        ctx.scope.push();
        let body = lower_stmts(&clause.body, func_type, ctx);
        ctx.scope.pop();
        body
    });

    Some(HirStmt::If {
        condition,
        then_body,
        elif_clauses,
        else_body,
    })
}

fn lower_while(while_stmt: &StmtWhile, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let condition = lower_expr(&while_stmt.test, ctx)?;

    ctx.scope.push();
    ctx.loop_depth += 1;
    let body = lower_stmts(&while_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

    Some(HirStmt::While { condition, body })
}

fn lower_for(for_stmt: &StmtFor, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    // Lower the iterable expression
    let iter_expr = lower_expr(&for_stmt.iter, ctx)?;
    let iter_ty = iter_expr.ty().clone();

    // Determine the element type from the iterable
    let elem_ty = iter_ty.iterable_element_type().unwrap_or_else(|| {
        ctx.error(format!(
            "cannot iterate over type '{}'",
            iter_ty.display_name()
        ));
        Type::Any
    });

    // Extract the target variable name
    let target_name = match for_stmt.target.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("for loop target must be a simple name".to_string());
            return None;
        }
    };

    // Create a new scope for the loop body, define the loop variable
    ctx.scope.push();
    ctx.scope.define(target_name.clone(), elem_ty.clone());
    ctx.loop_depth += 1;
    let body = lower_stmts(&for_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

    Some(HirStmt::For {
        target: target_name,
        target_ty: elem_ty,
        iter: iter_expr,
        body,
    })
}

fn lower_expr(expr: &Expr, ctx: &mut LowerCtx) -> Option<HirExpr> {
    match expr {
        Expr::NumberLiteral(num) => lower_number_literal(num),
        Expr::StringLiteral(s) => {
            let value = s.value.to_str().to_string();
            Some(HirExpr::StringLiteral(value))
        }
        Expr::BooleanLiteral(b) => Some(HirExpr::BoolLiteral(b.value)),
        Expr::NoneLiteral(_) => Some(HirExpr::NoneLiteral),
        Expr::Name(name) => lower_name(name, ctx),
        Expr::BinOp(binop) => lower_binop(binop, ctx),
        Expr::UnaryOp(unary) => lower_unaryop(unary, ctx),
        Expr::Compare(cmp) => lower_compare(cmp, ctx),
        Expr::BoolOp(boolop) => lower_boolop(boolop, ctx),
        Expr::Call(call) => lower_call(call, ctx),
        Expr::If(if_expr) => lower_if_expr(if_expr, ctx),
        Expr::List(list) => lower_list_literal(list, ctx),
        Expr::Dict(dict) => lower_dict_literal(dict, ctx),
        Expr::Tuple(tuple) => lower_tuple_literal(tuple, ctx),
        Expr::Subscript(sub) => lower_subscript(sub, ctx),
        Expr::Attribute(attr) => lower_attribute(attr, ctx),
        Expr::FString(fstring) => lower_fstring(fstring, ctx),
        _ => {
            ctx.error("unsupported expression type".to_string());
            None
        }
    }
}

fn lower_number_literal(num: &ExprNumberLiteral) -> Option<HirExpr> {
    match &num.value {
        Number::Int(i) => {
            let val = i.as_i64()?;
            Some(HirExpr::IntLiteral(val))
        }
        Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
        Number::Complex { .. } => None, // Not supported in M1
    }
}

fn lower_name(name: &ExprName, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let var_name = name.id.to_string();

    // Check if it's a known variable
    if let Some(info) = ctx.scope.lookup(&var_name) {
        let is_moved = info.is_moved;
        let ty = info.ty.clone();
        if is_moved {
            ctx.error(format!(
                "use of moved value: '{}'",
                var_name
            ));
        }
        return Some(HirExpr::Name {
            name: var_name,
            ty,
        });
    }

    // Check if it's a known function
    if let Some(ft) = ctx.functions.get(&var_name) {
        let ft = ft.clone();
        return Some(HirExpr::Name {
            name: var_name,
            ty: Type::Function(ft),
        });
    }

    // Check built-in constants
    match var_name.as_str() {
        "True" => return Some(HirExpr::BoolLiteral(true)),
        "False" => return Some(HirExpr::BoolLiteral(false)),
        _ => {}
    }

    ctx.error(format!("undefined variable: '{}'", var_name));
    None
}

fn lower_binop(binop: &ExprBinOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&binop.left, ctx)?;
    let right = lower_expr(&binop.right, ctx)?;

    let op_str = match binop.op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mult => "*",
        Operator::Div => "/",
        Operator::FloorDiv => "//",
        Operator::Mod => "%",
        Operator::Pow => "**",
        _ => {
            ctx.error(format!("unsupported binary operator"));
            return None;
        }
    };

    match type_check_binary_op(left.ty(), op_str, right.ty()) {
        Ok(result_ty) => Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        }),
        Err(e) => {
            ctx.error(e.message);
            None
        }
    }
}

fn lower_unaryop(unary: &ExprUnaryOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let operand = lower_expr(&unary.operand, ctx)?;

    let op_str = match unary.op {
        UnaryOp::USub => "-",
        UnaryOp::UAdd => "+",
        UnaryOp::Not => "not",
        _ => {
            ctx.error("unsupported unary operator".to_string());
            return None;
        }
    };

    match type_check_unary_op(op_str, operand.ty()) {
        Ok(result_ty) => Some(HirExpr::UnaryOp {
            op: op_str.to_string(),
            operand: Box::new(operand),
            ty: result_ty,
        }),
        Err(e) => {
            ctx.error(e.message);
            None
        }
    }
}

fn lower_compare(cmp: &ExprCompare, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&cmp.left, ctx)?;

    // Handle `in` and `not in` operators specially
    if cmp.ops.len() == 1 {
        match &cmp.ops[0] {
            CmpOp::In => {
                let collection = lower_expr(&cmp.comparators[0], ctx)?;
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        ctx.error(format!(
                            "'in' operator: element type '{}' is not compatible with collection element type '{}'",
                            left.ty().display_name(),
                            elem_ty.display_name()
                        ));
                    }
                } else {
                    ctx.error(format!(
                        "'in' operator not supported for type '{}'",
                        collection_ty.display_name()
                    ));
                }
                return Some(HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                });
            }
            CmpOp::NotIn => {
                let collection = lower_expr(&cmp.comparators[0], ctx)?;
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        ctx.error(format!(
                            "'not in' operator: element type '{}' is not compatible with collection element type '{}'",
                            left.ty().display_name(),
                            elem_ty.display_name()
                        ));
                    }
                } else {
                    ctx.error(format!(
                        "'not in' operator not supported for type '{}'",
                        collection_ty.display_name()
                    ));
                }
                // Wrap in a UnaryOp not
                let contains = HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                };
                return Some(HirExpr::UnaryOp {
                    op: "not".to_string(),
                    operand: Box::new(contains),
                    ty: Type::Bool,
                });
            }
            _ => {}
        }
    }

    let mut ops = Vec::new();
    let mut comparators = Vec::new();

    for (op, comparator) in cmp.ops.iter().zip(cmp.comparators.iter()) {
        let op_str = match op {
            CmpOp::Eq => "==",
            CmpOp::NotEq => "!=",
            CmpOp::Lt => "<",
            CmpOp::Gt => ">",
            CmpOp::LtE => "<=",
            CmpOp::GtE => ">=",
            _ => {
                ctx.error("unsupported comparison operator".to_string());
                return None;
            }
        };

        let right = lower_expr(comparator, ctx)?;

        if let Err(e) = type_check_comparison(left.ty(), op_str, right.ty()) {
            ctx.error(e.message);
            return None;
        }

        ops.push(op_str.to_string());
        comparators.push(right);
    }

    Some(HirExpr::Compare {
        left: Box::new(left),
        ops,
        comparators,
        ty: Type::Bool,
    })
}

fn lower_boolop(boolop: &ExprBoolOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let op_str = match boolop.op {
        BoolOp::And => "and",
        BoolOp::Or => "or",
    };

    let mut values = Vec::new();
    for val in &boolop.values {
        let expr = lower_expr(val, ctx)?;
        values.push(expr);
    }

    // Check all values are Bool
    for val in &values {
        if let Err(e) = type_check_bool_op(val.ty(), op_str, &Type::Bool) {
            ctx.error(e.message);
            return None;
        }
    }

    Some(HirExpr::BoolOp {
        op: op_str.to_string(),
        values,
        ty: Type::Bool,
    })
}

fn lower_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    // Handle method calls: obj.method(args)
    if let Expr::Attribute(attr) = call.func.as_ref() {
        return lower_method_call(attr, call, ctx);
    }

    let func_name = match call.func.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("only simple function calls are supported".to_string());
            return None;
        }
    };

    // Special handling for range() built-in
    if func_name == "range" {
        return lower_range_call(call, ctx);
    }

    // Special handling for len() built-in
    if func_name == "len" {
        return lower_len_call(call, ctx);
    }

    let ft = ctx.functions.get(&func_name).cloned().or_else(|| {
        ctx.error(format!("undefined function: '{}'", func_name));
        None
    })?;

    // Lower arguments
    let mut args = Vec::new();
    for arg in &call.arguments.args {
        let expr = lower_expr(arg, ctx)?;
        args.push(expr);
    }

    // Check argument count (special case for print which accepts any number)
    if func_name != "print" && args.len() != ft.params.len() {
        ctx.error(format!(
            "function '{}' expects {} argument(s), got {}",
            func_name,
            ft.params.len(),
            args.len()
        ));
        return None;
    }

    // Check argument types (skip for print)
    if func_name != "print" {
        for (i, (arg, (param_name, param_ty))) in args.iter().zip(ft.params.iter()).enumerate() {
            if !arg.ty().is_assignable_to(param_ty) {
                ctx.error(format!(
                    "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                    i + 1,
                    param_name,
                    func_name,
                    param_ty.display_name(),
                    arg.ty().display_name()
                ));
            }
        }
    }

    // Track ownership: move arguments of move types
    for arg in &args {
        if let HirExpr::Name { name, ty } = arg {
            if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                ctx.scope.mark_moved(name);
            }
        }
    }

    Some(HirExpr::Call {
        func: func_name,
        args,
        ty: *ft.return_type,
    })
}

fn lower_fstring(fstring: &ExprFString, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut parts = Vec::new();

    for part in &fstring.value {
        match part {
            sifr_python_ast::FStringPart::Literal(s) => {
                parts.push(HirFStringPart::Literal(s.to_string()));
            }
            sifr_python_ast::FStringPart::FString(fs) => {
                for element in fs.elements.iter() {
                    match element {
                        FStringElement::Literal(lit) => {
                            parts.push(HirFStringPart::Literal(lit.value.to_string()));
                        }
                        FStringElement::Expression(expr_elem) => {
                            let expr = lower_expr(&expr_elem.expression, ctx)?;
                            parts.push(HirFStringPart::Expr(expr));
                        }
                    }
                }
            }
        }
    }

    Some(HirExpr::FString {
        parts,
        ty: Type::Str,
    })
}

fn lower_tuple_unpack_assign(tuple: &ExprTuple, value: &Expr, ctx: &mut LowerCtx) -> Option<HirStmt> {
    // Extract target names
    let mut target_names = Vec::new();
    for elt in &tuple.elts {
        match elt {
            Expr::Name(n) => target_names.push(n.id.to_string()),
            _ => {
                ctx.error("tuple unpacking target must be a simple name".to_string());
                return None;
            }
        }
    }

    // Lower the value expression
    let value_expr = lower_expr(value, ctx)?;
    let value_ty = value_expr.ty().clone();

    // Check that the value is a tuple with matching length
    let elem_types = match &value_ty {
        Type::Tuple(elems) => {
            if elems.len() != target_names.len() {
                ctx.error(format!(
                    "tuple unpacking: expected {} values, got {}",
                    target_names.len(),
                    elems.len()
                ));
                return None;
            }
            elems.clone()
        }
        _ => {
            ctx.error(format!(
                "cannot unpack non-tuple type '{}'",
                value_ty.display_name()
            ));
            return None;
        }
    };

    // Define variables in scope
    let mut targets = Vec::new();
    for (name, ty) in target_names.into_iter().zip(elem_types.into_iter()) {
        ctx.scope.define(name.clone(), ty.clone());
        targets.push((name, ty));
    }

    Some(HirStmt::TupleUnpack {
        targets,
        value: value_expr,
    })
}

fn lower_list_literal(list: &ExprList, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_ty: Option<Type> = None;

    for elt in &list.elts {
        let expr = lower_expr(elt, ctx)?;
        let ty = expr.ty().clone();
        if let Some(ref expected) = elem_ty {
            if !ty.is_assignable_to(expected) {
                ctx.error(format!(
                    "list element type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    ty.display_name()
                ));
            }
        } else {
            elem_ty = Some(ty);
        }
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    let list_ty = Type::List(Box::new(final_elem_ty));

    Some(HirExpr::ListLiteral {
        elements,
        ty: list_ty,
    })
}

fn lower_dict_literal(dict: &ExprDict, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut key_ty: Option<Type> = None;
    let mut val_ty: Option<Type> = None;

    for item in &dict.items {
        if let Some(ref key_expr) = item.key {
            let key = lower_expr(key_expr, ctx)?;
            let kt = key.ty().clone();
            if let Some(ref expected) = key_ty {
                if !kt.is_assignable_to(expected) {
                    ctx.error(format!(
                        "dict key type mismatch: expected '{}', got '{}'",
                        expected.display_name(),
                        kt.display_name()
                    ));
                }
            } else {
                key_ty = Some(kt);
            }
            keys.push(key);
        } else {
            ctx.error("dict unpacking (**) not supported".to_string());
            return None;
        }

        let val = lower_expr(&item.value, ctx)?;
        let vt = val.ty().clone();
        if let Some(ref expected) = val_ty {
            if !vt.is_assignable_to(expected) {
                ctx.error(format!(
                    "dict value type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    vt.display_name()
                ));
            }
        } else {
            val_ty = Some(vt);
        }
        values.push(val);
    }

    let final_key_ty = key_ty.unwrap_or(Type::Any);
    let final_val_ty = val_ty.unwrap_or(Type::Any);
    let dict_ty = Type::Dict(Box::new(final_key_ty), Box::new(final_val_ty));

    Some(HirExpr::DictLiteral {
        keys,
        values,
        ty: dict_ty,
    })
}

fn lower_tuple_literal(tuple: &ExprTuple, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_types = Vec::new();

    for elt in &tuple.elts {
        let expr = lower_expr(elt, ctx)?;
        elem_types.push(expr.ty().clone());
        elements.push(expr);
    }

    let tuple_ty = Type::Tuple(elem_types);

    Some(HirExpr::TupleLiteral {
        elements,
        ty: tuple_ty,
    })
}

fn lower_subscript(sub: &ExprSubscript, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let object = lower_expr(&sub.value, ctx)?;
    let object_ty = object.ty().clone();

    let index = lower_expr(&sub.slice, ctx)?;
    let index_ty = index.ty().clone();

    let result_ty = object_ty.index_result_type(&index_ty).unwrap_or_else(|| {
        ctx.error(format!(
            "cannot index type '{}' with '{}'",
            object_ty.display_name(),
            index_ty.display_name()
        ));
        Type::Any
    });

    Some(HirExpr::Index {
        object: Box::new(object),
        index: Box::new(index),
        ty: result_ty,
    })
}

fn lower_attribute(attr: &ExprAttribute, ctx: &mut LowerCtx) -> Option<HirExpr> {
    // This handles attribute access like x.method -- but actual method calls
    // come through as Call(Attribute(...)), so we handle them in lower_call.
    // For now, just report unsupported.
    let _object = lower_expr(&attr.value, ctx)?;
    ctx.error(format!("attribute access '.{}' is not supported as an expression; use as a method call", attr.attr));
    None
}

fn lower_method_call(attr: &ExprAttribute, call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let object = lower_expr(&attr.value, ctx)?;
    let object_ty = object.ty().clone();
    let method_name = attr.attr.to_string();

    // Lower arguments
    let mut args = Vec::new();
    for arg in &call.arguments.args {
        let expr = lower_expr(arg, ctx)?;
        args.push(expr);
    }

    // Resolve method return type based on object type and method name
    let return_ty = resolve_method_type(&object_ty, &method_name, &args, ctx)?;

    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name,
        args,
        ty: return_ty,
    })
}

/// Resolve the return type of a method call on a given type.
fn resolve_method_type(object_ty: &Type, method: &str, args: &[HirExpr], ctx: &mut LowerCtx) -> Option<Type> {
    match object_ty {
        Type::List(elem_ty) => match method {
            "append" => {
                if args.len() != 1 {
                    ctx.error(format!("list.append() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                if !args[0].ty().is_assignable_to(elem_ty) {
                    ctx.error(format!(
                        "list.append() argument type '{}' is not compatible with list element type '{}'",
                        args[0].ty().display_name(),
                        elem_ty.display_name()
                    ));
                }
                Some(Type::None)
            }
            "len" => {
                if !args.is_empty() {
                    ctx.error("list.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "pop" => {
                if !args.is_empty() {
                    ctx.error("list.pop() takes no arguments".to_string());
                    return None;
                }
                Some(*elem_ty.clone())
            }
            _ => {
                ctx.error(format!("list has no method '{}'", method));
                None
            }
        },
        Type::Dict(key_ty, val_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    ctx.error("dict.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "keys" => {
                if !args.is_empty() {
                    ctx.error("dict.keys() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(key_ty.clone()))
            }
            "values" => {
                if !args.is_empty() {
                    ctx.error("dict.values() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(val_ty.clone()))
            }
            "get" => {
                if args.len() != 1 {
                    ctx.error(format!("dict.get() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(*val_ty.clone())
            }
            _ => {
                ctx.error(format!("dict has no method '{}'", method));
                None
            }
        },
        Type::Str => match method {
            "len" => Some(Type::Int),
            "upper" | "lower" | "strip" | "lstrip" | "rstrip" => Some(Type::Str),
            "startswith" | "endswith" => {
                if args.len() != 1 {
                    ctx.error(format!("str.{}() takes exactly 1 argument, got {}", method, args.len()));
                    return None;
                }
                Some(Type::Bool)
            }
            "split" => {
                if args.len() > 1 {
                    ctx.error(format!("str.split() takes 0 or 1 arguments, got {}", args.len()));
                    return None;
                }
                Some(Type::List(Box::new(Type::Str)))
            }
            "replace" => {
                if args.len() != 2 {
                    ctx.error(format!("str.replace() takes exactly 2 arguments, got {}", args.len()));
                    return None;
                }
                Some(Type::Str)
            }
            "find" => {
                if args.len() != 1 {
                    ctx.error(format!("str.find() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::Int)
            }
            _ => {
                ctx.error(format!("str has no method '{}'", method));
                None
            }
        },
        Type::Tuple(_) => match method {
            "len" => Some(Type::Int),
            _ => {
                ctx.error(format!("tuple has no method '{}'", method));
                None
            }
        },
        _ => {
            ctx.error(format!(
                "type '{}' has no method '{}'",
                object_ty.display_name(),
                method
            ));
            None
        }
    }
}

fn lower_len_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!("len() takes exactly 1 argument, got {}", call.arguments.args.len()));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let arg_ty = arg.ty().clone();

    // len() works on str, list, dict, tuple
    match &arg_ty {
        Type::Str | Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) => {
            Some(HirExpr::MethodCall {
                object: Box::new(arg),
                method: "len".to_string(),
                args: vec![],
                ty: Type::Int,
            })
        }
        _ => {
            ctx.error(format!(
                "len() argument must be a string, list, dict, or tuple, got '{}'",
                arg_ty.display_name()
            ));
            None
        }
    }
}

fn lower_range_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let args: Vec<_> = call.arguments.args.iter().collect();

    match args.len() {
        1 => {
            // range(end) -> 0..end
            let end = lower_expr(args[0], ctx)?;
            if end.ty() != &Type::Int {
                ctx.error(format!(
                    "range() argument must be 'int', got '{}'",
                    end.ty().display_name()
                ));
                return None;
            }
            Some(HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(end),
                ty: Type::Range,
            })
        }
        2 => {
            // range(start, end) -> start..end
            let start = lower_expr(args[0], ctx)?;
            let end = lower_expr(args[1], ctx)?;
            if start.ty() != &Type::Int {
                ctx.error(format!(
                    "range() start argument must be 'int', got '{}'",
                    start.ty().display_name()
                ));
                return None;
            }
            if end.ty() != &Type::Int {
                ctx.error(format!(
                    "range() end argument must be 'int', got '{}'",
                    end.ty().display_name()
                ));
                return None;
            }
            Some(HirExpr::RangeLiteral {
                start: Box::new(start),
                end: Box::new(end),
                ty: Type::Range,
            })
        }
        _ => {
            ctx.error(format!(
                "range() takes 1 or 2 arguments, got {}",
                args.len()
            ));
            None
        }
    }
}

fn lower_if_expr(if_expr: &ExprIf, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let condition = lower_expr(&if_expr.test, ctx)?;
    let then_expr = lower_expr(&if_expr.body, ctx)?;
    let else_expr = lower_expr(&if_expr.orelse, ctx)?;

    let then_ty = then_expr.ty().clone();
    let else_ty = else_expr.ty().clone();

    if !then_ty.is_assignable_to(&else_ty) && !else_ty.is_assignable_to(&then_ty) {
        ctx.error(format!(
            "if expression branches have incompatible types: '{}' and '{}'",
            then_ty.display_name(),
            else_ty.display_name()
        ));
        return None;
    }

    Some(HirExpr::IfExpr {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
        ty: then_ty,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_python_parser::parse_module;

    fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
        let parsed = parse_module(source).expect("parse failed");
        lower_module(parsed.suite())
    }

    #[test]
    fn test_simple_function() {
        let module = lower_source(
            "def add(a: int, b: int) -> int:\n    return a + b\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "add");
        assert_eq!(module.functions[0].return_type, Type::Int);
    }

    #[test]
    fn test_type_mismatch_error() {
        let result = lower_source(
            "def main():\n    x: int = \"hello\"\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("type mismatch")));
    }

    #[test]
    fn test_undefined_variable() {
        let result = lower_source(
            "def main():\n    print(x)\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("undefined variable")));
    }

    #[test]
    fn test_use_after_move() {
        let result = lower_source(
            "def main():\n    s: str = \"hello\"\n    print(s)\n    print(s)\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("moved value")));
    }

    #[test]
    fn test_copy_type_no_move() {
        let module = lower_source(
            "def main():\n    x: int = 42\n    print(x)\n    print(x)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_while_loop() {
        let module = lower_source(
            "def main():\n    i: int = 0\n    while i < 10:\n        i = i + 1\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        // Body should contain a Let and a While
        assert!(module.functions[0].body.len() >= 2);
        assert!(matches!(module.functions[0].body[1], HirStmt::While { .. }));
    }

    #[test]
    fn test_for_range() {
        let module = lower_source(
            "def main():\n    for i in range(10):\n        print(i)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
    }

    #[test]
    fn test_for_range_start_end() {
        let module = lower_source(
            "def main():\n    for i in range(1, 5):\n        print(i)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
    }

    #[test]
    fn test_break_outside_loop() {
        let result = lower_source(
            "def main():\n    break\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("'break' outside of loop")));
    }

    #[test]
    fn test_continue_outside_loop() {
        let result = lower_source(
            "def main():\n    continue\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("'continue' outside of loop")));
    }

    #[test]
    fn test_break_inside_loop() {
        let module = lower_source(
            "def main():\n    while True:\n        break\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_nested_loops() {
        let module = lower_source(
            "def main():\n    for i in range(3):\n        for j in range(2):\n            print(i)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_fstring_basic() {
        let module = lower_source(
            "def main():\n    name: str = \"Alice\"\n    msg: str = f\"Hello, {name}!\"\n    print(msg)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        // Should have 3 statements: let name, let msg, print
        assert_eq!(module.functions[0].body.len(), 3);
    }

    #[test]
    fn test_fstring_with_expression() {
        let module = lower_source(
            "def main():\n    a: int = 2\n    b: int = 3\n    print(f\"{a} + {b} = {a + b}\")\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_tuple_unpack() {
        let module = lower_source(
            "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y = pair\n    print(x)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        // Should have: let pair, tuple_unpack, print
        assert!(module.functions[0].body.len() >= 3);
        assert!(matches!(module.functions[0].body[1], HirStmt::TupleUnpack { .. }));
    }

    #[test]
    fn test_tuple_unpack_wrong_count() {
        let result = lower_source(
            "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y, z = pair\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("expected 3 values, got 2")));
    }

    #[test]
    fn test_tuple_unpack_non_tuple() {
        let result = lower_source(
            "def main():\n    x: int = 42\n    a, b = x\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("cannot unpack non-tuple")));
    }
}
