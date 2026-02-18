//! HIR node definitions -- typed versions of AST nodes.

use sifr_type_system::{Type, ParamConvention};

/// A complete HIR module (the top-level compilation unit).
#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub classes: Vec<HirClass>,
    pub imports: Vec<HirImport>,
    /// Module-level constants (name, type, value)
    pub constants: Vec<(String, Type, HirExpr)>,
}

/// An import statement.
#[derive(Debug, Clone)]
pub struct HirImport {
    /// The module to import from (e.g., "utils" for `from utils import helper`)
    pub module: String,
    /// The names to import (original names from the module)
    pub names: Vec<String>,
    /// Optional aliases: maps original name -> local alias (only present when `as` is used)
    pub aliases: Vec<(String, String)>,
}

/// A class definition with resolved types.
#[derive(Debug, Clone)]
pub struct HirClass {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub methods: Vec<HirFunction>,
    /// Whether all fields support Eq + Hash (enables derive(Eq, Hash))
    pub is_hashable: bool,
    /// Whether this class is an error type (class Foo(Error))
    pub is_error_type: bool,
    /// Whether this class is a Protocol (maps to Rust trait)
    pub is_protocol: bool,
    /// Operator overloading methods: maps dunder name to method
    /// e.g., "__add__" -> HirFunction, "__eq__" -> HirFunction
    pub operator_impls: Vec<(String, HirFunction)>,
    /// For newtype declarations: the wrapped primitive type
    /// e.g., `class Port(int)` -> Some(Type::Int)
    pub newtype_inner: Option<Type>,
    /// List of protocols this class implements (protocol names)
    pub implements_protocols: Vec<String>,
    /// Parent class name for single inheritance
    pub parent_class: Option<String>,
    /// Generic type parameters (e.g., T, K, V from PEP 695 or TypeVar)
    pub type_params: Vec<String>,
}

/// Method kind: regular, classmethod, or staticmethod
#[derive(Debug, Clone, PartialEq)]
pub enum MethodKind {
    Regular,
    ClassMethod,
    StaticMethod,
}

/// A function definition with resolved types.
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub body: Vec<HirStmt>,
    /// Method kind: Regular, ClassMethod, or StaticMethod
    pub method_kind: MethodKind,
    /// User-defined decorators (excluding classmethod/staticmethod)
    pub decorators: Vec<String>,
    /// Generic type parameters (e.g., ["T", "K", "V"] for generic functions)
    pub type_params: Vec<String>,
}

/// A function parameter with its type, convention, and optional default value.
#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub ty: Type,
    pub default: Option<HirExpr>,
    pub keyword_only: bool,
    /// How this parameter receives its value (borrow, mut borrow, or own).
    pub convention: ParamConvention,
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
    /// Assert statement: assert condition [, message]
    Assert {
        test: HirExpr,
        msg: Option<HirExpr>,
    },
    /// Raise statement: raise expr -> Err(expr)
    Raise {
        value: HirExpr,
    },
    /// Try/except: pattern matching on Result
    TryExcept {
        body: Vec<HirStmt>,
        handlers: Vec<HirExceptHandler>,
        /// Error types that can arise from the try body (collected during lowering)
        body_error_types: Vec<String>,
    },
    /// Field assignment: self.field = value (inside methods)
    FieldAssign {
        object: String,
        field: String,
        value: HirExpr,
    },
    /// Subscript assignment: list[i] = val or dict[key] = val
    SubscriptAssign {
        object: String,
        index: HirExpr,
        value: HirExpr,
        object_ty: Type,
    },
    /// Nested subscript assignment: matrix[i][j] = val
    NestedSubscriptAssign {
        object: String,
        outer_index: HirExpr,
        inner_index: HirExpr,
        value: HirExpr,
        object_ty: Type,
    },
    /// Subscript augmented assignment: list[i] += val
    SubscriptAugAssign {
        object: String,
        index: HirExpr,
        op: String,
        value: HirExpr,
        object_ty: Type,
    },
    /// Augmented assignment on attribute: self.field += val
    AttributeAugAssign {
        object: String,
        field: String,
        op: String,
        value: HirExpr,
    },
    /// Subscript assignment on an attribute: self.field[key] = val
    AttributeSubscriptAssign {
        object: String,
        field: String,
        index: HirExpr,
        value: HirExpr,
        field_ty: Type,
    },
    /// Delete statement: del d[key] or del a[i]
    Delete {
        object: HirExpr,
        index: HirExpr,
    },
    /// Yield statement: yield expr (in generator functions)
    Yield {
        value: HirExpr,
    },
    /// With statement: with expr as var: body
    /// Supports multiple context managers: with A() as a, B() as b: body
    /// Each item is (var_name, context_expr, has_context_manager_protocol)
    With {
        items: Vec<(String, HirExpr, bool)>,
        body: Vec<HirStmt>,
    },
    /// Nested function definition: def inside def
    NestedFunction {
        func: HirFunction,
    },
}

/// An except handler in a try/except block.
#[derive(Debug, Clone)]
pub struct HirExceptHandler {
    /// The error type to match (None = catch-all)
    pub error_type: Option<String>,
    /// The resolved type of the error (for codegen)
    pub error_resolved_type: Option<Type>,
    /// Variable name to bind the error value
    pub name: Option<String>,
    /// Handler body
    pub body: Vec<HirStmt>,
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
        step: Option<Box<HirExpr>>,
        ty: Type,
    },
    /// List literal: [1, 2, 3]
    ListLiteral {
        elements: Vec<HirExpr>,
        ty: Type,
    },
    /// Set literal: {1, 2, 3}
    SetLiteral {
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
    /// Question mark operator: expr? (early return on Err)
    QuestionMark {
        expr: Box<HirExpr>,
        ty: Type,
    },
    /// Ok wrapping: Ok(expr)
    OkWrap {
        value: Box<HirExpr>,
        ty: Type,
    },
    /// Err wrapping: Err(expr)
    ErrWrap {
        value: Box<HirExpr>,
        ty: Type,
    },
    /// Super call: super().__init__(args) -> ParentType::new(args)
    SuperCall {
        parent_class: String,
        method: String,
        args: Vec<HirExpr>,
        ty: Type,
    },
    /// Lambda expression: lambda x: x + 1
    Lambda {
        params: Vec<HirParam>,
        body: Box<HirExpr>,
        ty: Type,
    },
    /// List comprehension: [expr for var in iter] or [expr for var in iter if cond]
    /// Supports multiple generators: [expr for v1 in iter1 for v2 in iter2 ...]
    ListComp {
        expr: Box<HirExpr>,
        generators: Vec<(String, HirExpr, Option<HirExpr>)>,
        ty: Type,
    },
    /// Dict comprehension: {key: val for var in iter}
    DictComp {
        key_expr: Box<HirExpr>,
        val_expr: Box<HirExpr>,
        generators: Vec<(String, HirExpr, Option<HirExpr>)>,
        ty: Type,
    },
    /// Set comprehension: {expr for var in iter}
    SetComp {
        expr: Box<HirExpr>,
        generators: Vec<(String, HirExpr, Option<HirExpr>)>,
        ty: Type,
    },
    /// Generator expression: (expr for var in iter) -> lazy iterator
    GeneratorExpr {
        expr: Box<HirExpr>,
        var: String,
        iter: Box<HirExpr>,
        filter: Option<Box<HirExpr>>,
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
            | Self::SetLiteral { ty, .. }
            | Self::DictLiteral { ty, .. }
            | Self::TupleLiteral { ty, .. }
            | Self::Index { ty, .. }
            | Self::MethodCall { ty, .. }
            | Self::ContainsOp { ty, .. }
            | Self::FString { ty, .. }
            | Self::Slice { ty, .. }
            | Self::WalrusExpr { ty, .. }
            | Self::FieldAccess { ty, .. }
            | Self::ConstructorCall { ty, .. }
            | Self::QuestionMark { ty, .. }
            | Self::OkWrap { ty, .. }
            | Self::ErrWrap { ty, .. }
            | Self::SuperCall { ty, .. }
            | Self::Lambda { ty, .. }
            | Self::ListComp { ty, .. }
            | Self::DictComp { ty, .. }
            | Self::SetComp { ty, .. }
            | Self::GeneratorExpr { ty, .. } => ty,
        }
    }
}
