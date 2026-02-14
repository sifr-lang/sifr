//! HIR node definitions -- typed versions of AST nodes.

use sifr_type_system::Type;

/// A complete HIR module (the top-level compilation unit).
#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
}

/// A function definition with resolved types.
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub body: Vec<HirStmt>,
}

/// A function parameter with its type.
#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub ty: Type,
}

/// A typed statement.
#[derive(Debug, Clone)]
pub enum HirStmt {
    /// Variable declaration: `x: int = expr` or `x = expr`
    Let {
        name: String,
        ty: Type,
        value: HirExpr,
        is_mutable: bool,
    },
    /// Assignment to existing variable: `x = expr`
    Assign {
        name: String,
        value: HirExpr,
    },
    /// Return statement
    Return {
        value: Option<HirExpr>,
    },
    /// Expression statement (e.g., function call)
    Expr {
        expr: HirExpr,
    },
    /// If/elif/else
    If {
        condition: HirExpr,
        then_body: Vec<HirStmt>,
        elif_clauses: Vec<(HirExpr, Vec<HirStmt>)>,
        else_body: Option<Vec<HirStmt>>,
    },
    /// While loop
    While {
        condition: HirExpr,
        body: Vec<HirStmt>,
    },
    /// For loop
    For {
        target: String,
        target_ty: Type,
        iter: HirExpr,
        body: Vec<HirStmt>,
    },
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Pass (no-op)
    Pass,
}

/// A typed expression.
#[derive(Debug, Clone)]
pub enum HirExpr {
    /// Integer literal
    IntLiteral(i64),
    /// Float literal
    FloatLiteral(f64),
    /// String literal
    StringLiteral(String),
    /// Boolean literal
    BoolLiteral(bool),
    /// None literal
    NoneLiteral,
    /// Variable reference
    Name {
        name: String,
        ty: Type,
    },
    /// Binary operation (a + b, a - b, etc.)
    BinOp {
        left: Box<HirExpr>,
        op: String,
        right: Box<HirExpr>,
        ty: Type,
    },
    /// Unary operation (-x, not x)
    UnaryOp {
        op: String,
        operand: Box<HirExpr>,
        ty: Type,
    },
    /// Comparison (a == b, a < b, etc.)
    Compare {
        left: Box<HirExpr>,
        ops: Vec<String>,
        comparators: Vec<HirExpr>,
        ty: Type,
    },
    /// Boolean operation (a and b, a or b)
    BoolOp {
        op: String,
        values: Vec<HirExpr>,
        ty: Type,
    },
    /// Function call
    Call {
        func: String,
        args: Vec<HirExpr>,
        ty: Type,
    },
    /// Conditional expression (x if cond else y)
    IfExpr {
        condition: Box<HirExpr>,
        then_expr: Box<HirExpr>,
        else_expr: Box<HirExpr>,
        ty: Type,
    },
    /// Range literal: range(end) or range(start, end)
    RangeLiteral {
        start: Box<HirExpr>,
        end: Box<HirExpr>,
        ty: Type,
    },
}

impl HirExpr {
    /// Get the type of this expression.
    pub fn ty(&self) -> &Type {
        match self {
            Self::IntLiteral(_) => &Type::Int,
            Self::FloatLiteral(_) => &Type::Float,
            Self::StringLiteral(_) => &Type::Str,
            Self::BoolLiteral(_) => &Type::Bool,
            Self::NoneLiteral => &Type::None,
            Self::Name { ty, .. }
            | Self::BinOp { ty, .. }
            | Self::UnaryOp { ty, .. }
            | Self::Compare { ty, .. }
            | Self::BoolOp { ty, .. }
            | Self::Call { ty, .. }
            | Self::IfExpr { ty, .. }
            | Self::RangeLiteral { ty, .. } => ty,
        }
    }
}
