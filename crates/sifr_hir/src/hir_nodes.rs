//! HIR node definitions -- typed versions of AST nodes.

use sifr_type_system::Type;

/// A complete HIR module (the top-level compilation unit).
#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub classes: Vec<HirClass>,
}

/// A class definition with resolved types.
#[derive(Debug, Clone)]
pub struct HirClass {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub methods: Vec<HirFunction>,
    /// Whether all fields support Eq + Hash (enables derive(Eq, Hash))
    pub is_hashable: bool,
}

/// A function definition with resolved types.
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub body: Vec<HirStmt>,
}

/// A function parameter with its type and optional default value.
#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub ty: Type,
    pub default: Option<HirExpr>,
    pub keyword_only: bool,
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
    /// Augmented assignment: `x += expr`
    AugAssign {
        name: String,
        op: String,
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
        else_body: Option<Vec<HirStmt>>,
    },
    /// For loop
    For {
        target: String,
        target_ty: Type,
        iter: HirExpr,
        body: Vec<HirStmt>,
        else_body: Option<Vec<HirStmt>>,
    },
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Tuple unpacking: a, b = expr
    TupleUnpack {
        targets: Vec<(String, Type)>,
        value: HirExpr,
    },
    /// Star unpacking: first, *rest = items
    StarUnpack {
        before: Vec<(String, Type)>,
        star: (String, Type),
        after: Vec<(String, Type)>,
        value: HirExpr,
    },
    /// Pass (no-op)
    Pass,
    /// Field assignment: self.field = value (inside methods)
    FieldAssign {
        object: String,
        field: String,
        value: HirExpr,
    },
}

/// A part of an f-string.
#[derive(Debug, Clone)]
pub enum HirFStringPart {
    /// A literal string part
    Literal(String),
    /// An interpolated expression
    Expr(HirExpr),
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
    /// List literal: [1, 2, 3]
    ListLiteral {
        elements: Vec<HirExpr>,
        ty: Type,
    },
    /// Dict literal: {"a": 1, "b": 2}
    DictLiteral {
        keys: Vec<HirExpr>,
        values: Vec<HirExpr>,
        ty: Type,
    },
    /// Tuple literal: (1, "hello")
    TupleLiteral {
        elements: Vec<HirExpr>,
        ty: Type,
    },
    /// Indexing: x[0], d["key"]
    Index {
        object: Box<HirExpr>,
        index: Box<HirExpr>,
        ty: Type,
    },
    /// Method call: x.append(1), s.upper()
    MethodCall {
        object: Box<HirExpr>,
        method: String,
        args: Vec<HirExpr>,
        ty: Type,
    },
    /// Contains check: x in collection
    ContainsOp {
        element: Box<HirExpr>,
        collection: Box<HirExpr>,
        ty: Type,
    },
    /// F-string: f"Hello {name}"
    FString {
        parts: Vec<HirFStringPart>,
        ty: Type,
    },
    /// Slice: x[start:stop] or x[start:stop:step]
    Slice {
        object: Box<HirExpr>,
        start: Option<Box<HirExpr>>,
        stop: Option<Box<HirExpr>>,
        step: Option<Box<HirExpr>>,
        ty: Type,
    },
    /// Walrus (named expression): (n := expr)
    WalrusExpr {
        name: String,
        value: Box<HirExpr>,
        ty: Type,
    },
    /// Field access: obj.field
    FieldAccess {
        object: Box<HirExpr>,
        field: String,
        ty: Type,
    },
    /// Constructor call: ClassName(args)
    ConstructorCall {
        class_name: String,
        args: Vec<HirExpr>,
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
            | Self::RangeLiteral { ty, .. }
            | Self::ListLiteral { ty, .. }
            | Self::DictLiteral { ty, .. }
            | Self::TupleLiteral { ty, .. }
            | Self::Index { ty, .. }
            | Self::MethodCall { ty, .. }
            | Self::ContainsOp { ty, .. }
            | Self::FString { ty, .. }
            | Self::Slice { ty, .. }
            | Self::WalrusExpr { ty, .. }
            | Self::FieldAccess { ty, .. }
            | Self::ConstructorCall { ty, .. } => ty,
        }
    }
}
