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
    Let {
        mutable: bool,
        name: String,
        ty: Option<RustType>,
        value: RustExpr,
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
    Ident(String),
    Path(Vec<String>),
    MethodCall {
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

#[derive(Debug, Clone, PartialEq)]
pub enum RustLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
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
    Unit,
    Vec(Box<RustType>),
    HashMap(Box<RustType>, Box<RustType>),
    HashSet(Box<RustType>),
    VecDeque(Box<RustType>),
    Option(Box<RustType>),
    Result(Box<RustType>, Box<RustType>),
    Tuple(Vec<RustType>),
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
    DynTrait(String),
    Impl(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustParam {
    SelfParam { mutable: bool },
    SelfValue,
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
mod tests {
    use super::*;

    #[test]
    fn constructs_struct_with_derives() {
        let item = RustItem::Struct {
            name: "Point".to_string(),
            visibility: Visibility::Pub,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields: vec![
                ("x".to_string(), RustType::I64),
                ("y".to_string(), RustType::I64),
            ],
        };

        match item {
            RustItem::Struct {
                name,
                derives,
                fields,
                ..
            } => {
                assert_eq!(name, "Point");
                assert_eq!(derives, vec!["Debug".to_string(), "Clone".to_string()]);
                assert_eq!(fields.len(), 2);
            }
            _ => unreachable!("constructed as Struct"),
        }
    }

    #[test]
    fn constructs_impl_block_with_method() {
        let method = RustItem::Fn {
            name: "sum".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::SelfParam { mutable: false }],
            ret: Some(RustType::I64),
            body: vec![RustStmt::Return(Some(RustExpr::BinOp {
                left: Box::new(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "x".to_string(),
                }),
                op: "+".to_string(),
                right: Box::new(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: "y".to_string(),
                }),
            }))],
            is_async: false,
        };
        let item = RustItem::Impl {
            target: "Point".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![method],
        };

        match item {
            RustItem::Impl { target, items, .. } => {
                assert_eq!(target, "Point");
                assert_eq!(items.len(), 1);
            }
            _ => unreachable!("constructed as Impl"),
        }
    }

    #[test]
    fn constructs_function_with_let_if_return() {
        let func = RustItem::Fn {
            name: "abs_like".to_string(),
            visibility: Visibility::Pub,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "v".to_string(),
                ty: RustType::I64,
            }],
            ret: Some(RustType::I64),
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "is_negative".to_string(),
                    ty: Some(RustType::Bool),
                    value: RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("v".to_string())),
                        op: "<".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    },
                },
                RustStmt::If {
                    cond: RustExpr::Ident("is_negative".to_string()),
                    then_body: vec![RustStmt::Return(Some(RustExpr::UnaryOp {
                        op: "-".to_string(),
                        operand: Box::new(RustExpr::Ident("v".to_string())),
                    }))],
                    else_body: Some(vec![RustStmt::Return(Some(RustExpr::Ident(
                        "v".to_string(),
                    )))]),
                },
            ],
            is_async: false,
        };

        match func {
            RustItem::Fn { body, .. } => {
                assert_eq!(body.len(), 2);
                assert!(matches!(body[0], RustStmt::Let { .. }));
                assert!(matches!(body[1], RustStmt::If { .. }));
            }
            _ => unreachable!("constructed as Fn"),
        }
    }

    #[test]
    fn constructs_match_expression() {
        let expr = RustExpr::Match {
            expr: Box::new(RustExpr::Ident("value".to_string())),
            arms: vec![
                RustMatchArm {
                    pattern: "0".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Str(
                        "zero".to_string(),
                    ))))],
                },
                RustMatchArm {
                    pattern: "_".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Str(
                        "many".to_string(),
                    ))))],
                },
            ],
        };

        match expr {
            RustExpr::Match { arms, .. } => assert_eq!(arms.len(), 2),
            _ => unreachable!("constructed as Match"),
        }
    }

    #[test]
    fn constructs_closure_expression() {
        let expr = RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "x".to_string(),
                ty: RustType::I64,
            }],
            body: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("x".to_string())),
                op: "+".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
            }),
            is_move: false,
        };

        match expr {
            RustExpr::Closure { params, .. } => assert_eq!(params.len(), 1),
            _ => unreachable!("constructed as Closure"),
        }
    }

    #[test]
    fn constructs_parenthesized_expression() {
        let expr = RustExpr::Paren(Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
            op: "+".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
        }));

        match expr {
            RustExpr::Paren(inner) => assert!(matches!(*inner, RustExpr::BinOp { .. })),
            _ => unreachable!("constructed as Paren"),
        }
    }
}
