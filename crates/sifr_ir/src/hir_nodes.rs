//! HIR node definitions -- typed versions of AST nodes.

use crate::{PythonInteropDeclaration, RustInteropDeclaration};
use ruff_text_size::TextRange;
use sifr_type_system::{ParamConvention, ReceiverConvention, Type};

/// A complete HIR module (the top-level compilation unit).
#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub classes: Vec<HirClass>,
    pub imports: Vec<HirImport>,
    /// Module-level constants (name, type, value)
    pub constants: Vec<(String, Type, HirExpr)>,
    /// Generic function info: `function_name` -> `type_var_names`
    pub generic_functions: std::collections::HashMap<String, Vec<String>>,
    /// Type parameter bounds: `owner_name` (function or class) -> (`type_var_name` -> `protocol_names`)
    pub type_param_bounds:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
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
pub enum HirClassKind {
    Regular,
    Protocol,
    Enum,
    PythonOpaque(PythonInteropDeclaration),
}

/// A class definition with resolved types.
#[derive(Debug, Clone)]
pub struct HirClass {
    pub name: String,
    /// Stable declaration identity. `None` denotes the current user module.
    pub identity: Option<String>,
    pub fields: Vec<(String, Type)>,
    pub methods: Vec<HirFunction>,
    /// Whether all fields support Eq + Hash (enables derive(Eq, Hash))
    pub is_hashable: bool,
    /// Whether this class is an error type (class Foo(Error))
    pub is_error_type: bool,
    /// Class category (regular class, protocol, or enum)
    pub kind: HirClassKind,
    /// Operator overloading methods: maps dunder name to method
    /// e.g., "__add__" -> `HirFunction`, "__eq__" -> `HirFunction`
    pub operator_impls: Vec<(String, HirFunction)>,
    /// For newtype declarations: the wrapped primitive type
    /// e.g., `class Port(int)` -> `Some(Type::Int)`
    pub newtype_inner: Option<Type>,
    /// List of protocols this class implements (protocol names)
    pub implements_protocols: Vec<String>,
    /// Parent class name for single inheritance
    pub parent_class: Option<String>,
    /// Resolved parent type used by executable inheritance codegen.
    pub parent_type: Option<Type>,
    /// Generic type parameters (e.g., T, K, V from PEP 695 or `TypeVar`)
    pub type_params: Vec<String>,
    /// Enum variants: (name, `optional_value`)
    /// e.g., RED = 1 -> ("RED", Some(1))
    /// e.g., RED -> ("RED", None)
    pub enum_variants: Vec<(String, Option<i64>)>,
    /// Structured Rust interop declarations attached to this class.
    pub rust_interop: Vec<RustInteropDeclaration>,
}

impl HirClass {
    #[must_use]
    pub fn is_self_type(&self, ty: &Type) -> bool {
        let Type::Class {
            identity,
            type_args,
            name,
            ..
        } = ty.resolve_alias()
        else {
            return false;
        };
        if identity.as_ref().unwrap_or(name) != self.identity.as_ref().unwrap_or(&self.name) {
            return false;
        }
        let expected_args = self
            .type_params
            .iter()
            .cloned()
            .map(Type::TypeVar)
            .collect::<Vec<_>>();
        type_args == &expected_args
    }

    pub fn is_protocol(&self) -> bool {
        matches!(self.kind, HirClassKind::Protocol)
    }

    pub fn is_enum(&self) -> bool {
        matches!(self.kind, HirClassKind::Enum)
    }

    pub fn python_opaque_declaration(&self) -> Option<&PythonInteropDeclaration> {
        match &self.kind {
            HirClassKind::PythonOpaque(declaration) => Some(declaration),
            HirClassKind::Regular | HirClassKind::Protocol | HirClassKind::Enum => None,
        }
    }
}

/// Method kind: regular, classmethod, or staticmethod
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodKind {
    Regular,
    ClassMethod,
    StaticMethod,
}

/// Stable identity assigned to a resolved source binding during lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BindingId(pub u32);

/// Source ranges retained for ownership diagnostics on source method calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodCallSource {
    pub call_range: TextRange,
    pub receiver_range: TextRange,
    pub arg_ranges: Vec<TextRange>,
}

/// Stable identity of a field projection used by ownership-place analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldIdentity {
    pub declaring_class: String,
    pub field: String,
}

/// A projection from a binding root to checked storage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlaceProjection {
    Field(FieldIdentity),
}

/// A checked source storage place.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Place {
    pub root: BindingId,
    pub projections: Vec<PlaceProjection>,
}

/// Proven target shape for a mutable method receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutableReceiverTarget {
    Place(Place),
    OwnedTemporary,
    /// Compiler-owned indexed container mutation with a separately audited
    /// lowering. The base place is retained for exclusivity checks.
    SpecializedIndexedStorage(Place),
}

/// Proven target shape for a mutable call argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutableArgumentTarget {
    Place(Place),
    OwnedTemporary,
}

/// A function definition with resolved types.
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub body: Vec<HirStmt>,
    /// Whether this function was declared with `async def`.
    pub is_async: bool,
    /// Method kind: Regular, `ClassMethod`, or `StaticMethod`
    pub method_kind: MethodKind,
    /// Receiver convention for regular instance methods.
    pub receiver: Option<ReceiverConvention>,
    /// User-defined decorators (excluding classmethod/staticmethod)
    pub decorators: Vec<String>,
    /// Structured Rust interop declarations attached to this function.
    pub rust_interop: Vec<RustInteropDeclaration>,
    /// Structured declaration-first Python interop metadata.
    pub python_interop: Vec<PythonInteropDeclaration>,
    /// Compiler-owned callable identity declared by canonical sysroot source.
    pub compiler_intrinsic: Option<CompilerIntrinsicId>,
    /// Generic type parameters (e.g., `["T", "K", "V"]` for generic functions)
    pub type_params: Vec<String>,
}

/// Typed identity for compiler-owned operations.
///
/// Callable identity is carried separately from `FunctionType`: signatures
/// remain ordinary type metadata while this enum controls the only path into
/// compiler intrinsic code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompilerIntrinsicId {
    TestAssertEqual,
    TestAssertNotEqual,
    TestAssertTrue,
    TestAssertFalse,
    TestAssertAlmostEqual,
    TestAssertGreaterThan,
    TestAssertLessThan,
    OpenBinary,
    OpenText,
    BytesFromHex,
    BytesWithSize,
    BytesFromIntegers,
    StringEncode,
    StringEncodeWithEncoding,
    BytesDecode,
    BytesDecodeWithEncoding,
    TaskCurrentContext,
    PythonFromValue,
    PythonToValue,
    PythonKwarg,
}

impl CompilerIntrinsicId {
    #[must_use]
    pub const fn declaration_name(self) -> &'static str {
        match self {
            Self::TestAssertEqual => "test_assert_equal",
            Self::TestAssertNotEqual => "test_assert_not_equal",
            Self::TestAssertTrue => "test_assert_true",
            Self::TestAssertFalse => "test_assert_false",
            Self::TestAssertAlmostEqual => "test_assert_almost_equal",
            Self::TestAssertGreaterThan => "test_assert_greater_than",
            Self::TestAssertLessThan => "test_assert_less_than",
            Self::OpenBinary => "open_binary",
            Self::OpenText => "open_text",
            Self::BytesFromHex => "bytes_from_hex",
            Self::BytesWithSize => "bytes_with_size",
            Self::BytesFromIntegers => "bytes_from_integers",
            Self::StringEncode => "string_encode",
            Self::StringEncodeWithEncoding => "string_encode_with_encoding",
            Self::BytesDecode => "bytes_decode",
            Self::BytesDecodeWithEncoding => "bytes_decode_with_encoding",
            Self::TaskCurrentContext => "task_current_context",
            Self::PythonFromValue => "python_from_value",
            Self::PythonToValue => "python_to_value",
            Self::PythonKwarg => "python_kwarg",
        }
    }

    #[must_use]
    pub fn from_declaration_name(name: &str) -> Option<Self> {
        Some(match name {
            "test_assert_equal" => Self::TestAssertEqual,
            "test_assert_not_equal" => Self::TestAssertNotEqual,
            "test_assert_true" => Self::TestAssertTrue,
            "test_assert_false" => Self::TestAssertFalse,
            "test_assert_almost_equal" => Self::TestAssertAlmostEqual,
            "test_assert_greater_than" => Self::TestAssertGreaterThan,
            "test_assert_less_than" => Self::TestAssertLessThan,
            "open_binary" => Self::OpenBinary,
            "open_text" => Self::OpenText,
            "bytes_from_hex" => Self::BytesFromHex,
            "bytes_with_size" => Self::BytesWithSize,
            "bytes_from_integers" => Self::BytesFromIntegers,
            "string_encode" => Self::StringEncode,
            "string_encode_with_encoding" => Self::StringEncodeWithEncoding,
            "bytes_decode" => Self::BytesDecode,
            "bytes_decode_with_encoding" => Self::BytesDecodeWithEncoding,
            "task_current_context" => Self::TaskCurrentContext,
            "python_from_value" => Self::PythonFromValue,
            "python_to_value" => Self::PythonToValue,
            "python_kwarg" => Self::PythonKwarg,
            _ => return None,
        })
    }
}

/// Async-with forms recognized by the compiler.
#[derive(Debug, Clone)]
pub enum HirAsyncWithKind {
    TaskScope,
    TaskGroup {
        context: Option<HirExpr>,
    },
    TaskTimeout {
        duration: HirExpr,
    },
    UserDefined {
        context: HirExpr,
        enter_value_ty: Type,
        enter_error_ty: Type,
        exit_error_ty: Type,
    },
    /// Dedicated declaration-first Python async context protocol, distinct
    /// from native user-defined async context semantics.
    Python {
        context: HirExpr,
        manager_class: String,
        entered_type: Type,
        enter_error_type: Type,
        exit_error_type: Type,
        entered_is_opaque_borrow: bool,
        active_error_type: Type,
    },
}

/// Synchronous with-item protocol selected during type-directed lowering.
#[derive(Debug, Clone)]
pub enum HirWithItemKind {
    Native {
        has_context_manager_protocol: bool,
    },
    Python {
        entered_type: Type,
        enter_error_type: Type,
        exit_error_type: Type,
        entered_is_opaque_borrow: bool,
    },
}

/// One manager in a synchronous `with` statement.
#[derive(Debug, Clone)]
pub struct HirWithItem {
    pub target: String,
    pub context: HirExpr,
    pub kind: HirWithItemKind,
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
    Assign { name: String, value: HirExpr },
    /// Augmented assignment: `x += expr`
    AugAssign {
        name: String,
        op: String,
        value: HirExpr,
    },
    /// Return statement
    Return { value: Option<HirExpr> },
    /// Expression statement (e.g., function call)
    Expr { expr: HirExpr },
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
    /// Async for loop over `AsyncIterator[T, E]`.
    AsyncFor {
        target: String,
        target_ty: Type,
        iter: HirExpr,
        iter_error_ty: Type,
        close_error_ty: Option<Type>,
        body: Vec<HirStmt>,
        else_body: Option<Vec<HirStmt>>,
    },
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Tuple unpacking: a, b = expr
    TupleUnpack {
        targets: Vec<HirTupleTarget>,
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
    Assert { test: HirExpr, msg: Option<HirExpr> },
    /// Raise statement: raise expr -> Err(expr)
    Raise { value: HirExpr },
    /// Try/except: pattern matching on Result
    /// Note: HIR intentionally has no dedicated `else_body` for try/except.
    /// Python-style `try ... except ... else` can be represented using explicit
    /// control-flow statements inside `body`/handlers during lowering.
    TryExcept {
        body: Vec<HirStmt>,
        handlers: Vec<HirExceptHandler>,
        /// Error types that can arise from the try body (collected during lowering)
        body_error_types: Vec<Type>,
    },
    /// Try/finally: finalbody runs before normal completion, return, or error propagation.
    TryFinally {
        body: Vec<HirStmt>,
        finalbody: Vec<HirStmt>,
    },
    /// Field assignment: self.field = value (inside methods)
    FieldAssign {
        object: String,
        field: String,
        field_ty: Type,
        value: HirExpr,
    },
    /// Nested field assignment: obj.field.inner = value
    NestedFieldAssign {
        object: String,
        field: String,
        field_ty: Type,
        nested_field: String,
        nested_field_ty: Type,
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
    /// Nested subscript assignment on an attribute: self.field[key][i] = val
    AttributeNestedSubscriptAssign {
        object: String,
        field: String,
        outer_index: HirExpr,
        inner_index: HirExpr,
        value: HirExpr,
        field_ty: Type,
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
    Delete { object: HirExpr, index: HirExpr },
    /// Yield statement: yield expr (in generator functions)
    Yield { value: HirExpr },
    /// With statement: with expr as var: body
    /// Supports multiple context managers: with `A()` as a, `B()` as b: body
    /// Python context items retain their dedicated protocol and scoped-borrow metadata.
    With {
        items: Vec<HirWithItem>,
        body: Vec<HirStmt>,
    },
    /// Built-in async with forms: `async with task.scope()` and `async with task.timeout(...)`.
    AsyncWith {
        kind: HirAsyncWithKind,
        target: Option<String>,
        body: Vec<HirStmt>,
    },
    /// Nested function definition: def inside def
    NestedFunction {
        func: HirFunction,
        /// Whether the closure must own its captured environment.
        move_captures: bool,
        /// Captures cloned in an isolated construction block before a
        /// retained closure takes ownership.
        capture_clones: Vec<String>,
    },
    /// Match/case statement (Python 3.10 structural pattern matching)
    Match {
        subject: HirExpr,
        subject_ty: Type,
        arms: Vec<HirMatchArm>,
    },
}

/// A single arm in a match statement.
#[derive(Debug, Clone)]
pub struct HirMatchArm {
    /// The pattern to match against
    pub pattern: HirPattern,
    /// Optional guard condition
    pub guard: Option<HirExpr>,
    /// Body to execute when pattern matches
    pub body: Vec<HirStmt>,
}

/// Pattern types for match/case statements.
#[derive(Debug, Clone)]
pub enum HirPattern {
    /// `case _:` — matches anything, no binding
    Wildcard,
    /// `case x:` — captures the value into variable `x`
    Capture { name: String, ty: Type },
    /// `case 42:` / `case "hello":` / `case True:` — literal value match
    Literal { value: HirExpr },
    /// `case None:` — matches None in T | None
    None,
    /// `case "GET" | "POST":` — OR pattern
    Or { patterns: Vec<HirPattern> },
    /// `case Circle(radius=r):` — class pattern with field bindings
    Class {
        class_name: String,
        class_type: Type,
        fields: Vec<(String, HirPattern)>,
    },
    /// `case Color.RED:` — attribute value pattern (enum-like)
    Value { path: Vec<String> },
    /// `case (x, y):` — tuple destructuring pattern
    Tuple { elements: Vec<HirPattern> },
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

/// Canonical iterator operations lowered as dedicated HIR nodes instead of
/// generic builtin-call strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirIteratorOp {
    Iter,
    Next,
    Reversed,
    Map,
    Filter,
    Zip,
    Enumerate,
}

/// A typed expression.
#[derive(Debug, Clone)]
pub enum HirExpr {
    /// Integer literal
    IntLiteral(i64),
    /// Canonical decimal integer literal that does not fit in the historical small-literal `i64`
    /// slot.
    LargeIntLiteral(String),
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
        binding_id: Option<BindingId>,
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
        mutable_arg_places: Vec<Option<MutableArgumentTarget>>,
        ty: Type,
    },
    /// A typed call to a declaration-first Python wrapper.
    PythonCall {
        func: String,
        args: Vec<HirExpr>,
        provided_arguments: Vec<bool>,
        record_expansions: Vec<PythonRecordExpansion>,
        ty: Type,
    },
    /// Compiler-owned operation selected by typed lowering metadata.
    IntrinsicCall {
        intrinsic: CompilerIntrinsicId,
        args: Vec<HirExpr>,
        ty: Type,
        call_range: TextRange,
        arg_ranges: Vec<TextRange>,
    },
    /// Await expression. The operand must have an awaitable type.
    Await { value: Box<HirExpr>, ty: Type },
    /// Canonical iterator operation call.
    IteratorCall {
        op: HirIteratorOp,
        args: Vec<HirExpr>,
        mutable_arg_places: Vec<Option<MutableArgumentTarget>>,
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
    ListLiteral { elements: Vec<HirExpr>, ty: Type },
    /// Set literal: {1, 2, 3}
    SetLiteral { elements: Vec<HirExpr>, ty: Type },
    /// Dict literal: {"a": 1, "b": 2}
    DictLiteral {
        keys: Vec<HirExpr>,
        values: Vec<HirExpr>,
        ty: Type,
    },
    /// Tuple literal: (1, "hello")
    TupleLiteral { elements: Vec<HirExpr>, ty: Type },
    /// Indexing: `x[0]`, `d["key"]`
    Index {
        object: Box<HirExpr>,
        index: Box<HirExpr>,
        ty: Type,
    },
    /// Method call: `x.append(1)`, `s.upper()`
    MethodCall {
        object: Box<HirExpr>,
        method: String,
        args: Vec<HirExpr>,
        receiver_convention: Option<ReceiverConvention>,
        receiver_target: Option<MutableReceiverTarget>,
        mutable_arg_places: Vec<Option<MutableArgumentTarget>>,
        source: Option<MethodCallSource>,
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
    QuestionMark { expr: Box<HirExpr>, ty: Type },
    /// Ok wrapping: Ok(expr)
    OkWrap { value: Box<HirExpr>, ty: Type },
    /// Err wrapping: Err(expr)
    ErrWrap { value: Box<HirExpr>, ty: Type },
    /// Super call: `super().__init__(args)` -> `ParentType::new(args)`
    SuperCall {
        parent_class: String,
        parent_type: Type,
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
    /// Enum variant access: `Color.RED` -> `Color::RED`
    EnumVariant {
        enum_name: String,
        variant: String,
        ty: Type,
    },
}

/// A tuple-unpack target binding destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirTupleTargetBinding {
    /// Local name binding target.
    Name(String),
    /// Attribute target such as `obj.field`.
    Field { object: String, field: String },
}

/// A tuple-unpack target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTupleTarget {
    /// The assignment destination.
    pub binding: HirTupleTargetBinding,
    /// The inferred/declared element type.
    pub ty: Type,
    /// Whether this target rebinds an existing local/nonlocal binding.
    pub rebind_existing: bool,
}

impl HirExpr {
    /// Get the type of this expression.
    pub fn ty(&self) -> &Type {
        match self {
            Self::IntLiteral(_) | Self::LargeIntLiteral(_) => &Type::Int,
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
            | Self::PythonCall { ty, .. }
            | Self::IntrinsicCall { ty, .. }
            | Self::Await { ty, .. }
            | Self::IteratorCall { ty, .. }
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
            | Self::GeneratorExpr { ty, .. }
            | Self::EnumVariant { ty, .. } => ty,
        }
    }
}

/// Closed-record kwargs metadata retained from explicit `**record` syntax.
#[derive(Debug, Clone)]
pub struct PythonRecordExpansion {
    pub span: TextRange,
    pub fields: Vec<String>,
}
