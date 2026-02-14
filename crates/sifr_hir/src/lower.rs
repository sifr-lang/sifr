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
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    fn error(&mut self, message: String) {
        self.errors.push(LoweringError {
            message,
            line: None,
            col: None,
        });
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
        Stmt::Pass(_) => Some(HirStmt::Pass),
        _ => {
            ctx.error(format!("unsupported statement type"));
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
    let func_name = match call.func.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("only simple function calls are supported".to_string());
            return None;
        }
    };

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
}
