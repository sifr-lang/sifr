//! Structured Rust IR used by code generation.

#[derive(Debug, Clone, PartialEq)]
pub struct RustFile {
    pub items: Vec<RustItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustItem {
    Use(Vec<String>),
    UseAlias {
        path: Vec<String>,
        alias: String,
    },
    Struct {
        name: String,
        visibility: Visibility,
        derives: Vec<String>,
        fields: Vec<(String, RustType)>,
    },
    TupleStruct {
        name: String,
        visibility: Visibility,
        derives: Vec<String>,
        inner: RustType,
    },
    Enum {
        name: String,
        visibility: Visibility,
        derives: Vec<String>,
        repr: Option<String>,
        variants: Vec<RustEnumVariant>,
    },
    Trait {
        name: String,
        visibility: Visibility,
        supertraits: Vec<String>,
        methods: Vec<RustItem>,
    },
    Impl {
        target: String,
        type_params: Vec<RustTypeParam>,
        trait_: Option<String>,
        items: Vec<RustItem>,
    },
    Fn {
        name: String,
        visibility: Visibility,
        type_params: Vec<RustTypeParam>,
        params: Vec<RustParam>,
        ret: Option<RustType>,
        body: Vec<RustStmt>,
        is_async: bool,
    },
    TraitMethodSig {
        name: String,
        params: Vec<RustParam>,
        ret: Option<RustType>,
        is_async: bool,
    },
    TypeAlias {
        name: String,
        ty: RustType,
    },
    Const {
        name: String,
        visibility: Visibility,
        ty: RustType,
        value: RustExpr,
    },
    Static {
        name: String,
        visibility: Visibility,
        ty: RustType,
        value: RustExpr,
    },
    Attr(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustStmt {
    /// A validated compiler-owned Rust fragment used only where the structured
    /// IR cannot yet represent the generated control-flow form. Keeping this
    /// distinct from `Ident` prevents raw syntax from masquerading as a name.
    Verbatim(String),
    Let {
        mutable: bool,
        name: String,
        ty: Option<RustType>,
        value: RustExpr,
    },
    LetDecl {
        mutable: bool,
        name: String,
        ty: RustType,
    },
    LetPattern {
        pattern: String,
        value: RustExpr,
    },
    LetElse {
        pattern: String,
        value: RustExpr,
        else_body: Vec<RustStmt>,
    },
    Assign {
        target: RustExpr,
        value: RustExpr,
    },
    AugAssign {
        target: RustExpr,
        op: String,
        value: RustExpr,
    },
    Expr(RustExpr),
    /// Final expression of an expression-valued block, rendered without a
    /// semicolon so the surrounding block evaluates to this value.
    TailExpr(RustExpr),
    Assert {
        cond: RustExpr,
        msg: Option<RustExpr>,
    },
    Return(Option<RustExpr>),
    If {
        cond: RustExpr,
        then_body: Vec<RustStmt>,
        else_body: Option<Vec<RustStmt>>,
    },
    IfLet {
        pattern: String,
        expr: RustExpr,
        then_body: Vec<RustStmt>,
        else_body: Option<Vec<RustStmt>>,
    },
    Match {
        expr: RustExpr,
        arms: Vec<RustMatchArm>,
    },
    For {
        var: String,
        iter: RustExpr,
        body: Vec<RustStmt>,
    },
    With {
        items: Vec<RustWithItem>,
        body: Vec<RustStmt>,
    },
    While {
        cond: RustExpr,
        body: Vec<RustStmt>,
    },
    Loop {
        body: Vec<RustStmt>,
    },
    LocalFn {
        name: String,
        params: Vec<RustParam>,
        ret: Option<RustType>,
        body: Vec<RustStmt>,
        is_async: bool,
    },
    Break,
    Continue,
    Block(Vec<RustStmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustWithItem {
    pub binding: String,
    pub value: RustExpr,
    pub mutable: bool,
    pub has_cm: bool,
    pub class_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustExpr {
    Literal(RustLiteral),
    /// A syntax-validated compiler-owned Rust expression for forms that the
    /// structured IR does not yet model. Never use `Ident` for raw syntax.
    Verbatim(String),
    Ident(String),
    Path(Vec<String>),
    MethodCall {
        receiver: Box<RustExpr>,
        method: String,
        args: Vec<RustExpr>,
    },
    /// A call whose method name is resolved from a Sifr source expression.
    /// Source methods may legally be named `unwrap` or `expect`.
    SourceMethodCall {
        receiver: Box<RustExpr>,
        method: String,
        args: Vec<RustExpr>,
    },
    FnCall {
        func: Box<RustExpr>,
        args: Vec<RustExpr>,
    },
    MacroCall {
        name: String,
        args: Vec<RustExpr>,
    },
    FormatMacro {
        name: String,
        format_str: String,
        args: Vec<RustExpr>,
    },
    BinOp {
        left: Box<RustExpr>,
        op: String,
        right: Box<RustExpr>,
    },
    UnaryOp {
        op: String,
        operand: Box<RustExpr>,
    },
    Field {
        expr: Box<RustExpr>,
        field: String,
    },
    Index {
        expr: Box<RustExpr>,
        index: Box<RustExpr>,
    },
    Slice {
        expr: Box<RustExpr>,
        start: Option<Box<RustExpr>>,
        stop: Option<Box<RustExpr>>,
    },
    Ref {
        mutable: bool,
        expr: Box<RustExpr>,
    },
    Deref(Box<RustExpr>),
    Clone(Box<RustExpr>),
    Cast {
        expr: Box<RustExpr>,
        ty: RustType,
    },
    Block {
        stmts: Vec<RustStmt>,
        expr: Option<Box<RustExpr>>,
    },
    If {
        cond: Box<RustExpr>,
        then_expr: Box<RustExpr>,
        else_expr: Option<Box<RustExpr>>,
    },
    Match {
        expr: Box<RustExpr>,
        arms: Vec<RustMatchArm>,
    },
    Closure {
        params: Vec<RustParam>,
        body: Box<RustExpr>,
        is_move: bool,
    },
    ClosureBlock {
        params: Vec<RustParam>,
        body: Vec<RustStmt>,
        is_move: bool,
        is_async: bool,
    },
    AsyncBlock {
        body: Vec<RustStmt>,
        is_move: bool,
    },
    StructInit {
        name: String,
        fields: Vec<(String, RustExpr)>,
    },
    Tuple(Vec<RustExpr>),
    Array(Vec<RustExpr>),
    Vec(Vec<RustExpr>),
    TimeoutAwait {
        duration: Box<RustExpr>,
        future: Box<RustExpr>,
        error: Box<RustExpr>,
    },
    Try(Box<RustExpr>),
    Await(Box<RustExpr>),
    Paren(Box<RustExpr>),
    Range {
        start: Box<RustExpr>,
        end: Box<RustExpr>,
    },
}

impl RustExpr {
    /// Keep source-resolved method names distinct from compiler-owned Rust calls.
    pub(crate) fn from_source_method(
        receiver: RustExpr,
        method: String,
        args: Vec<RustExpr>,
    ) -> Self {
        if matches!(method.as_str(), "unwrap" | "expect") {
            Self::SourceMethodCall {
                receiver: Box::new(receiver),
                method,
                args,
            }
        } else {
            Self::MethodCall {
                receiver: Box::new(receiver),
                method,
                args,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
    StaticStr(String),
    Str(String),
    Char(char),
    Unit,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustType {
    I64,
    F64,
    Bool,
    String_,
    Str,
    Unit,
    Vec(Box<RustType>),
    Slice(Box<RustType>),
    HashMap(Box<RustType>, Box<RustType>),
    HashSet(Box<RustType>),
    VecDeque(Box<RustType>),
    Option(Box<RustType>),
    Result(Box<RustType>, Box<RustType>),
    Tuple(Vec<RustType>),
    Array {
        element: Box<RustType>,
        len: usize,
    },
    Boxed(Box<RustType>),
    Never,
    Ref {
        mutable: bool,
        inner: Box<RustType>,
    },
    Named(String),
    Generic {
        base: String,
        params: Vec<RustType>,
    },
    Fn {
        params: Vec<RustType>,
        ret: Box<RustType>,
    },
    DynTrait {
        trait_: RustTrait,
        auto_traits: Vec<String>,
    },
    ImplTrait {
        trait_: RustTrait,
        auto_traits: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustTrait {
    Named {
        name: String,
        params: Vec<RustType>,
        associated_types: Vec<(String, RustType)>,
    },
    Callable {
        name: String,
        params: Vec<RustType>,
        ret: Option<Box<RustType>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustParam {
    SelfParam { mutable: bool },
    SelfParamWithLifetime { mutable: bool, lifetime: String },
    SelfValue,
    MutableSelfValue,
    Named { name: String, ty: RustType },
    NamedMut { name: String, ty: RustType },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustMatchArm {
    pub pattern: String,
    pub bindings: Vec<String>,
    pub guard: Option<RustExpr>,
    pub body: Vec<RustStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustEnumVariant {
    pub name: String,
    pub tuple_fields: Vec<RustType>,
    pub fields: Vec<(String, RustType)>,
    pub value: Option<RustExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustTypeParam {
    pub name: String,
    pub bounds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Private,
    Pub,
}

#[cfg(test)]
#[path = "../tests/unit/rust_ir/tests.rs"]
mod tests;
