//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::TypeParameterBounds;
use super::{
    BTreeMap, Binder, HirTemplateString, ParamConvention, PythonInteropDeclaration,
    ReceiverConvention, Ref, RustInteropDeclaration, Text, TextRange, Type,
    TypedDeclarationMetadata,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirModule {
    pub functions: Vec<Ref<HirFunction>>,
    pub classes: Vec<Ref<HirClass>>,
    pub imports: Vec<Ref<HirImport>>,
    pub constants: Vec<(Ref<Text>, Ref<Type>, Ref<HirExpr>)>,
    pub generic_functions: BTreeMap<Ref<Text>, Vec<Ref<Text>>>,
    pub type_param_bounds: TypeParameterBounds,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirImport {
    pub module: Ref<Text>,
    pub names: Vec<Ref<Text>>,
    pub aliases: Vec<(Ref<Text>, Ref<Text>)>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirClassKind {
    Regular,
    Protocol,
    Enum,
    PythonOpaque(Ref<PythonInteropDeclaration>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirClass {
    pub name: Ref<Text>,
    pub identity: Option<Ref<Text>>,
    pub fields: Vec<(Ref<Text>, Ref<Type>)>,
    pub field_defaults: Vec<(u32, Ref<HirExpr>)>,
    pub field_default_identities: Vec<(u32, Ref<Text>)>,
    pub declaration_metadata: Vec<Ref<TypedDeclarationMetadata>>,
    pub methods: Vec<Ref<HirFunction>>,
    pub is_hashable: bool,
    pub is_error_type: bool,
    pub kind: Ref<HirClassKind>,
    pub operator_impls: Vec<(Ref<Text>, Ref<HirFunction>)>,
    pub newtype_inner: Option<Ref<Type>>,
    pub implements_protocols: Vec<Ref<Text>>,
    pub parent_class: Option<Ref<Text>>,
    pub parent_type: Option<Ref<Type>>,
    pub type_params: Vec<Ref<Text>>,
    pub enum_variants: Vec<(Ref<Text>, Option<i64>)>,
    pub rust_interop: Vec<Ref<RustInteropDeclaration>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MethodKind {
    Regular,
    ClassMethod,
    StaticMethod,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindingId {
    pub binder: Ref<Binder>,
    pub slot: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodCallSource {
    pub call_range: Ref<TextRange>,
    pub receiver_range: Ref<TextRange>,
    pub arg_ranges: Vec<Ref<TextRange>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldIdentity {
    pub declaring_class: Ref<Text>,
    pub field: Ref<Text>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PlaceProjection {
    Field(Ref<FieldIdentity>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Place {
    pub root: Ref<BindingId>,
    pub projections: Vec<Ref<PlaceProjection>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MutableReceiverTarget {
    Place(Ref<Place>),
    OwnedTemporary,
    SpecializedIndexedStorage(Ref<Place>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MutableArgumentTarget {
    Place(Ref<Place>),
    OwnedTemporary,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirFunction {
    pub name: Ref<Text>,
    pub params: Vec<Ref<HirParam>>,
    pub return_type: Ref<Type>,
    pub body: Vec<Ref<HirStmt>>,
    pub is_async: bool,
    pub method_kind: Ref<MethodKind>,
    pub receiver: Option<Ref<ReceiverConvention>>,
    pub decorators: Vec<Ref<Text>>,
    pub rust_interop: Vec<Ref<RustInteropDeclaration>>,
    pub python_interop: Vec<Ref<PythonInteropDeclaration>>,
    pub compiler_intrinsic: Option<Ref<CompilerIntrinsicId>>,
    pub type_params: Vec<Ref<Text>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirAsyncWithKind {
    TaskScope,
    TaskGroup {
        context: Option<Ref<HirExpr>>,
    },
    TaskTimeout {
        duration: Ref<HirExpr>,
    },
    UserDefined {
        context: Ref<HirExpr>,
        enter_value_ty: Ref<Type>,
        enter_error_ty: Ref<Type>,
        exit_error_ty: Ref<Type>,
        active_error_ty: Ref<Type>,
        body_may_raise: bool,
    },
    Python {
        context: Ref<HirExpr>,
        manager_class: Ref<Text>,
        entered_type: Ref<Type>,
        enter_error_type: Ref<Type>,
        exit_error_type: Ref<Type>,
        entered_is_opaque_borrow: bool,
        active_error_type: Ref<Type>,
        body_may_raise: bool,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirWithItemKind {
    Native {
        has_context_manager_protocol: bool,
    },
    Python {
        entered_type: Ref<Type>,
        enter_error_type: Ref<Type>,
        exit_error_type: Ref<Type>,
        entered_is_opaque_borrow: bool,
        body_may_raise: bool,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirWithItem {
    pub target: Ref<Text>,
    pub context: Ref<HirExpr>,
    pub kind: Ref<HirWithItemKind>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirParam {
    pub name: Ref<Text>,
    pub ty: Ref<Type>,
    pub default: Option<Ref<HirExpr>>,
    pub keyword_only: bool,
    pub convention: Ref<ParamConvention>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirStmt {
    Let {
        name: Ref<Text>,
        ty: Ref<Type>,
        value: Ref<HirExpr>,
        is_mutable: bool,
    },
    Assign {
        name: Ref<Text>,
        value: Ref<HirExpr>,
    },
    AugAssign {
        name: Ref<Text>,
        op: Ref<Text>,
        value: Ref<HirExpr>,
    },
    Return {
        value: Option<Ref<HirExpr>>,
    },
    Expr {
        expr: Ref<HirExpr>,
    },
    If {
        condition: Ref<HirExpr>,
        then_body: Vec<Ref<HirStmt>>,
        elif_clauses: Vec<(Ref<HirExpr>, Vec<Ref<HirStmt>>)>,
        else_body: Option<Vec<Ref<HirStmt>>>,
    },
    While {
        condition: Ref<HirExpr>,
        body: Vec<Ref<HirStmt>>,
        else_body: Option<Vec<Ref<HirStmt>>>,
    },
    For {
        target: Ref<Text>,
        target_ty: Ref<Type>,
        iter: Ref<HirExpr>,
        body: Vec<Ref<HirStmt>>,
        else_body: Option<Vec<Ref<HirStmt>>>,
    },
    AsyncFor {
        target: Ref<Text>,
        target_ty: Ref<Type>,
        iter: Ref<HirExpr>,
        iter_error_ty: Ref<Type>,
        close_error_ty: Option<Ref<Type>>,
        active_error_ty: Ref<Type>,
        body_may_raise: bool,
        body: Vec<Ref<HirStmt>>,
        else_body: Option<Vec<Ref<HirStmt>>>,
    },
    Break,
    Continue,
    TupleUnpack {
        targets: Vec<Ref<HirTupleTarget>>,
        value: Ref<HirExpr>,
    },
    StarUnpack {
        before: Vec<Ref<HirTupleTarget>>,
        star: Ref<HirTupleTarget>,
        after: Vec<Ref<HirTupleTarget>>,
        value: Ref<HirExpr>,
        failure: Option<Ref<Type>>,
    },
    Pass,
    Assert {
        test: Ref<HirExpr>,
        msg: Option<Ref<HirExpr>>,
    },
    Raise {
        value: Ref<HirExpr>,
    },
    TryExcept {
        body: Vec<Ref<HirStmt>>,
        handlers: Vec<Ref<HirExceptHandler>>,
        body_error_types: Vec<Ref<Type>>,
    },
    TryFinally {
        body: Vec<Ref<HirStmt>>,
        finalbody: Vec<Ref<HirStmt>>,
    },
    FieldAssign {
        object: Ref<Text>,
        field: Ref<Text>,
        field_ty: Ref<Type>,
        value: Ref<HirExpr>,
    },
    NestedFieldAssign {
        object: Ref<Text>,
        field: Ref<Text>,
        field_ty: Ref<Type>,
        nested_field: Ref<Text>,
        nested_field_ty: Ref<Type>,
        value: Ref<HirExpr>,
    },
    SubscriptAssign {
        object: Ref<Text>,
        index: Ref<HirExpr>,
        value: Ref<HirExpr>,
        object_ty: Ref<Type>,
        failure: Option<Ref<Type>>,
    },
    NestedSubscriptAssign {
        object: Ref<Text>,
        outer_index: Ref<HirExpr>,
        inner_index: Ref<HirExpr>,
        value: Ref<HirExpr>,
        object_ty: Ref<Type>,
        outer_failure: Option<Ref<Type>>,
        inner_failure: Option<Ref<Type>>,
        operation: Ref<HirCollectionMutation>,
    },
    AttributeNestedSubscriptAssign {
        object: Ref<Text>,
        field: Ref<Text>,
        outer_index: Ref<HirExpr>,
        inner_index: Ref<HirExpr>,
        value: Ref<HirExpr>,
        field_ty: Ref<Type>,
        outer_failure: Option<Ref<Type>>,
        inner_failure: Option<Ref<Type>>,
        operation: Ref<HirCollectionMutation>,
    },
    SubscriptAugAssign {
        object: Ref<Text>,
        index: Ref<HirExpr>,
        op: Ref<Text>,
        value: Ref<HirExpr>,
        object_ty: Ref<Type>,
        failure: Option<Ref<Type>>,
    },
    AttributeAugAssign {
        object: Ref<Text>,
        field: Ref<Text>,
        op: Ref<Text>,
        value: Ref<HirExpr>,
    },
    AttributeSubscriptAssign {
        object: Ref<Text>,
        field: Ref<Text>,
        index: Ref<HirExpr>,
        value: Ref<HirExpr>,
        field_ty: Ref<Type>,
        failure: Option<Ref<Type>>,
        operation: Ref<HirCollectionMutation>,
    },
    Delete {
        object: Ref<HirExpr>,
        index: Ref<HirExpr>,
        failure: Option<Ref<Type>>,
    },
    Yield {
        value: Ref<HirExpr>,
    },
    With {
        items: Vec<Ref<HirWithItem>>,
        body: Vec<Ref<HirStmt>>,
    },
    AsyncWith {
        kind: Ref<HirAsyncWithKind>,
        target: Option<Ref<Text>>,
        body: Vec<Ref<HirStmt>>,
    },
    NestedFunction {
        func: Ref<HirFunction>,
        move_captures: bool,
        capture_clones: Vec<Ref<Text>>,
    },
    Match {
        subject: Ref<HirExpr>,
        subject_ty: Ref<Type>,
        arms: Vec<Ref<HirMatchArm>>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirCollectionMutation {
    Assign,
    AugAssign(Ref<Text>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirMatchArm {
    pub pattern: Ref<HirPattern>,
    pub guard: Option<Ref<HirExpr>>,
    pub body: Vec<Ref<HirStmt>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirPattern {
    Wildcard,
    Capture {
        name: Ref<Text>,
        ty: Ref<Type>,
    },
    Literal {
        value: Ref<HirExpr>,
    },
    None,
    Or {
        patterns: Vec<Ref<HirPattern>>,
    },
    Class {
        class_name: Ref<Text>,
        class_type: Ref<Type>,
        fields: Vec<(Ref<Text>, Ref<HirPattern>)>,
    },
    Value {
        path: Vec<Ref<Text>>,
    },
    Tuple {
        elements: Vec<Ref<HirPattern>>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirExceptHandler {
    pub error_type: Option<Ref<Text>>,
    pub error_resolved_type: Option<Ref<Type>>,
    pub name: Option<Ref<Text>>,
    pub body: Vec<Ref<HirStmt>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirFStringPart {
    Literal(Ref<Text>),
    Expr(Ref<HirExpr>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirIteratorOp {
    Iter,
    Next,
    Reversed,
    Map,
    Filter,
    Zip,
    Enumerate,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirExpr {
    IntLiteral(i64),
    LargeIntLiteral(Ref<Text>),
    FloatLiteral(u64),
    StringLiteral(Ref<Text>),
    BoolLiteral(bool),
    NoneLiteral,
    Name {
        name: Ref<Text>,
        binding_id: Option<Ref<BindingId>>,
        ty: Ref<Type>,
    },
    BinOp {
        left: Ref<HirExpr>,
        op: Ref<Text>,
        right: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    UnaryOp {
        op: Ref<Text>,
        operand: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    Compare {
        left: Ref<HirExpr>,
        ops: Vec<Ref<Text>>,
        comparators: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    BoolOp {
        op: Ref<Text>,
        values: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    Call {
        func: Ref<Text>,
        args: Vec<Ref<HirExpr>>,
        mutable_arg_places: Vec<Option<Ref<MutableArgumentTarget>>>,
        ty: Ref<Type>,
    },
    GenericCall {
        func: Ref<Text>,
        type_args: Vec<Ref<Type>>,
        args: Vec<Ref<HirExpr>>,
        mutable_arg_places: Vec<Option<Ref<MutableArgumentTarget>>>,
        ty: Ref<Type>,
    },
    PythonCall {
        func: Ref<Text>,
        args: Vec<Ref<HirExpr>>,
        provided_arguments: Vec<bool>,
        record_expansions: Vec<Ref<PythonRecordExpansion>>,
        ty: Ref<Type>,
    },
    IntrinsicCall {
        intrinsic: Ref<CompilerIntrinsicId>,
        args: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
        call_range: Ref<TextRange>,
        arg_ranges: Vec<Ref<TextRange>>,
    },
    Await {
        value: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    IteratorCall {
        op: Ref<HirIteratorOp>,
        args: Vec<Ref<HirExpr>>,
        mutable_arg_places: Vec<Option<Ref<MutableArgumentTarget>>>,
        ty: Ref<Type>,
    },
    IfExpr {
        condition: Ref<HirExpr>,
        then_expr: Ref<HirExpr>,
        else_expr: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    RangeLiteral {
        start: Ref<HirExpr>,
        end: Ref<HirExpr>,
        step: Option<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    ListLiteral {
        elements: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    SetLiteral {
        elements: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    DictLiteral {
        keys: Vec<Ref<HirExpr>>,
        values: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    TupleLiteral {
        elements: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    Index {
        object: Ref<HirExpr>,
        index: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    MethodCall {
        object: Ref<HirExpr>,
        method: Ref<Text>,
        args: Vec<Ref<HirExpr>>,
        receiver_convention: Option<Ref<ReceiverConvention>>,
        receiver_target: Option<Ref<MutableReceiverTarget>>,
        mutable_arg_places: Vec<Option<Ref<MutableArgumentTarget>>>,
        source: Option<Ref<MethodCallSource>>,
        ty: Ref<Type>,
    },
    ContainsOp {
        element: Ref<HirExpr>,
        collection: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    FString {
        parts: Vec<Ref<HirFStringPart>>,
        ty: Ref<Type>,
    },
    TemplateString(Ref<HirTemplateString>),
    Slice {
        object: Ref<HirExpr>,
        start: Option<Ref<HirExpr>>,
        stop: Option<Ref<HirExpr>>,
        step: Option<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    WalrusExpr {
        name: Ref<Text>,
        value: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    FieldAccess {
        object: Ref<HirExpr>,
        field: Ref<Text>,
        ty: Ref<Type>,
    },
    StructuralRecordProject {
        source: Ref<HirExpr>,
        fields: Vec<Ref<Text>>,
        ty: Ref<Type>,
    },
    ConstructorCall {
        class_name: Ref<Text>,
        args: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    QuestionMark {
        expr: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    OkWrap {
        value: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    ErrWrap {
        value: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    SuperCall {
        parent_class: Ref<Text>,
        parent_type: Ref<Type>,
        method: Ref<Text>,
        args: Vec<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    Lambda {
        params: Vec<Ref<HirParam>>,
        body: Ref<HirExpr>,
        ty: Ref<Type>,
    },
    ListComp {
        expr: Ref<HirExpr>,
        generators: Vec<ComprehensionGenerator>,
        ty: Ref<Type>,
    },
    DictComp {
        key_expr: Ref<HirExpr>,
        val_expr: Ref<HirExpr>,
        generators: Vec<ComprehensionGenerator>,
        ty: Ref<Type>,
    },
    SetComp {
        expr: Ref<HirExpr>,
        generators: Vec<ComprehensionGenerator>,
        ty: Ref<Type>,
    },
    GeneratorExpr {
        expr: Ref<HirExpr>,
        var: Ref<Text>,
        iter: Ref<HirExpr>,
        filter: Option<Ref<HirExpr>>,
        ty: Ref<Type>,
    },
    EnumVariant {
        enum_name: Ref<Text>,
        variant: Ref<Text>,
        ty: Ref<Type>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirTupleTargetBinding {
    Name(Ref<Text>),
    Field { object: Ref<Text>, field: Ref<Text> },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirTupleTarget {
    pub binding: Ref<HirTupleTargetBinding>,
    pub ty: Ref<Type>,
    pub rebind_existing: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonRecordExpansion {
    pub span: Ref<TextRange>,
    pub fields: Vec<Ref<Text>>,
}

pub type ComprehensionGenerator = (Ref<Text>, Ref<HirExpr>, Option<Ref<HirExpr>>);
