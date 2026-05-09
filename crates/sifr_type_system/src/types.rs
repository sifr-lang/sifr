//! Core type definitions for the Sifr type system.
use crate::union::make_union;

/// Represents a type in the Sifr language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Exact source-level integer (`int` in Sifr).
    Int,
    /// Explicit fixed-width integer family for representation-sensitive values.
    FixedInt(FixedIntType),
    /// 64-bit float (`float` in Sifr, `f64` in Rust)
    Float,
    /// Boolean (`bool`)
    Bool,
    /// String (`str` in Sifr, `String` in Rust)
    Str,
    /// Immutable byte sequence (`bytes` in Sifr)
    Bytes,
    /// None type (unit type `()` in Rust)
    None,
    /// Function type with parameter types and return type
    Function(FunctionType),
    /// Async callable type. Not assignable to sync `Function`/`Callable`.
    AsyncFunction(FunctionType),
    /// Linear coroutine produced by calling an async function.
    Coroutine(Box<Type>, Box<Type>),
    /// Scoped task observer handle produced by spawning a coroutine.
    Task(Box<Type>, Box<Type>),
    /// Result of observing a task handle.
    TaskResult(Box<Type>, Box<Type>),
    /// Ordinary error result used by task timeout wrappers.
    TimeoutResult(Box<Type>),
    /// Explicit blocking-offload observer handle.
    BlockingTask(Box<Type>, Box<Type>),
    /// Structural awaitability protocol.
    Awaitable(Box<Type>),
    /// Structural async iteration protocol.
    AsyncIterator(Box<Type>, Box<Type>),
    /// User-defined async generator object.
    AsyncGenerator(Box<Type>, Box<Type>),
    /// List type (`list[T]` in Sifr, `Vec<T>` in Rust)
    List(Box<Type>),
    /// Dictionary type (`dict[K, V]` in Sifr, `HashMap<K, V>` in Rust)
    Dict(Box<Type>, Box<Type>),
    /// Set type (`set[T]` in Sifr, `HashSet<T>` in Rust)
    Set(Box<Type>),
    /// Tuple type (`tuple[A, B, ...]` in Sifr, `(A, B, ...)` in Rust)
    Tuple(Vec<Type>),
    /// Range type (maps to `std::ops::Range<i64>` in Rust)
    Range,
    /// Iterable protocol type (`Iterable[T]`)
    Iterable(Box<Type>),
    /// Iterator protocol type (`Iterator[T]`)
    Iterator(Box<Type>),
    /// Explicit opt-out of type checking
    Any,
    /// Bottom type (function that never returns)
    Never,

    // --- Advanced Type System ---
    /// Union type: value is one of several types (`int | str`)
    /// Members are normalized: flattened, deduplicated, sorted.
    Union(Vec<Type>),
    /// Intersection type: value satisfies all types (internal, for narrowing)
    Intersection(Vec<Type>),
    /// Literal integer type: a specific int value as a type (`42`)
    LiteralInt(i64),
    /// Literal string type: a specific string value as a type (`"GET"`)
    LiteralStr(String),
    /// Literal boolean type: a specific bool value as a type (`True`)
    LiteralBool(bool),
    /// Type alias reference, optionally specialized with concrete type arguments.
    /// The body holds the alias expansion when known, or `Unknown` for symbolic
    /// recursive references that must preserve alias identity without infinite expansion.
    Alias {
        name: String,
        type_args: Vec<Type>,
        body: Box<Type>,
    },
    /// Safe top type: accepts any value but must be narrowed before use.
    /// Unlike `Any` which opts out of type checking, `Unknown` forces
    /// the programmer to prove the type before operating on it.
    Unknown,

    // --- Classes ---
    /// Result type: `Result[T, E]` -> `Result<T, E>` in Rust
    Result(Box<Type>, Box<Type>),

    /// Class instance type with named fields and methods.
    /// `class Point: x: float; y: float` -> `Type::Class { name: "Point", fields: [...], methods: [...] }`
    Class {
        name: String,
        fields: Vec<(String, Type)>,
        methods: Vec<(String, FunctionType)>,
        parent_class: Option<String>,
    },

    // --- Protocols, Operators, Discriminated Unions ---
    /// Protocol type: structural interface that maps to Rust `trait`.
    /// Any class with the required methods satisfies the protocol.
    Protocol {
        name: String,
        methods: Vec<(String, FunctionType)>,
    },

    /// Newtype wrapper around a primitive type.
    /// `class Port(int)` -> `struct Port(i64)`
    Newtype { name: String, inner: Box<Type> },

    // --- Generics ---
    /// Type variable: a generic type parameter (e.g., `T` in `def first[T](items: list[T]) -> T`)
    TypeVar(String),

    /// Callable type: `Callable[[int, str], bool]` -> `fn(i64, String) -> bool`
    /// Fields: (`parameter_types`, `parameter_conventions`, `return_type`).
    Callable(Vec<Type>, Vec<ParamConvention>, Box<Type>),

    // --- Enum Types ---
    /// Enum type: `class Color(Enum): RED = 1; GREEN = 2; BLUE = 3`
    /// Maps to a Rust `#[repr(i64)] enum Color { RED = 1, GREEN = 2, BLUE = 3 }`
    Enum {
        name: String,
        variants: Vec<(String, Option<i64>)>,
    },

    // --- Integer Safety ---
    /// Arbitrary-precision integer (`bigint` in Sifr, `num_bigint::BigInt` in Rust)
    /// Unlike `int` (i64), `bigint` never overflows — it grows as needed.
    BigInt,
    /// Fixed-precision decimal (`decimal` in Sifr, `rust_decimal::Decimal` in Rust)
    Decimal,
    /// Arbitrary-precision decimal (`bigdecimal` in Sifr, `bigdecimal::BigDecimal` in Rust)
    BigDecimal,
}

/// Explicit fixed-width integer types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FixedIntType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    ISize,
    USize,
}

impl FixedIntType {
    /// Temporary scalar arithmetic policy while ordinary fixed-width promotion
    /// still depends on the current exact-int codegen surface.
    #[must_use]
    pub const fn supports_current_scalar_promotion_to_int(self) -> bool {
        !matches!(self, Self::U64 | Self::USize)
    }

    #[must_use]
    pub const fn supports_current_int_builtin_widening(self) -> bool {
        matches!(
            self,
            Self::I8 | Self::I16 | Self::I32 | Self::U8 | Self::U16 | Self::U32
        )
    }

    #[must_use]
    pub const fn source_name(self) -> &'static str {
        match self {
            Self::I8 => "int8",
            Self::I16 => "int16",
            Self::I32 => "int32",
            Self::I64 => "int64",
            Self::U8 => "uint8",
            Self::U16 => "uint16",
            Self::U32 => "uint32",
            Self::U64 => "uint64",
            Self::ISize => "isize",
            Self::USize => "usize",
        }
    }

    #[must_use]
    pub const fn rust_name(self) -> &'static str {
        match self {
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::ISize => "isize",
            Self::USize => "usize",
        }
    }

    #[must_use]
    pub const fn variant_prefix(self) -> &'static str {
        match self {
            Self::I8 => "Int8",
            Self::I16 => "Int16",
            Self::I32 => "Int32",
            Self::I64 => "Int64",
            Self::U8 => "Uint8",
            Self::U16 => "Uint16",
            Self::U32 => "Uint32",
            Self::U64 => "Uint64",
            Self::ISize => "Isize",
            Self::USize => "Usize",
        }
    }
}

/// Iterator/iterable capabilities carried through typing and lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IterationCapability {
    /// The iterator is consumed once.
    SinglePass,
    /// The iterable can be traversed repeatedly.
    MultiPass,
    /// The iterable supports reverse traversal.
    DoubleEnded,
    /// The iterable knows its exact length.
    ExactSize,
}

/// Type-level iteration metadata used by lowering/codegen decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IterationMetadata {
    pub element_type: Type,
    pub capabilities: Vec<IterationCapability>,
}

impl IterationMetadata {
    #[must_use]
    pub fn supports(&self, capability: IterationCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// Represents a function's type signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    /// Parameter names, types, and conventions
    pub params: Vec<(String, Type, ParamConvention)>,
    /// Return type
    pub return_type: Box<Type>,
}

impl FunctionType {
    /// Create a `FunctionType` where all parameters use the default convention
    /// (Borrow for Move types, Own for Copy/TypeVar types).
    pub fn new(params: Vec<(String, Type)>, return_type: Type) -> Self {
        let params = params
            .into_iter()
            .map(|(name, ty)| {
                let conv =
                    if matches!(ty, Type::TypeVar(_)) || ty.ownership() == OwnershipKind::Copy {
                        ParamConvention::own()
                    } else {
                        ParamConvention::borrow()
                    };
                (name, ty, conv)
            })
            .collect();
        FunctionType {
            params,
            return_type: Box::new(return_type),
        }
    }

    /// Create a `FunctionType` where all parameters borrow (for built-in functions).
    pub fn all_borrow(params: Vec<(String, Type)>, return_type: Type) -> Self {
        let params = params
            .into_iter()
            .map(|(name, ty)| (name, ty, ParamConvention::borrow()))
            .collect();
        FunctionType {
            params,
            return_type: Box::new(return_type),
        }
    }
}

/// Parameter ownership mode in the type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ParamOwnership {
    /// Borrow the argument from the caller.
    #[default]
    Borrow,
    /// Transfer ownership into the callee.
    Own,
}

/// Parameter local mutability mode in the type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ParamMutability {
    /// The local binding is immutable.
    #[default]
    Immutable,
    /// The local binding is mutable.
    Mutable,
}

/// How a function parameter receives its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ParamConvention {
    ownership: ParamOwnership,
    mutability: ParamMutability,
}

impl ParamConvention {
    #[must_use]
    pub const fn new(ownership: ParamOwnership, mutability: ParamMutability) -> Self {
        Self {
            ownership,
            mutability,
        }
    }

    #[must_use]
    pub const fn borrow() -> Self {
        Self::new(ParamOwnership::Borrow, ParamMutability::Immutable)
    }

    #[must_use]
    pub const fn mut_borrow() -> Self {
        Self::new(ParamOwnership::Borrow, ParamMutability::Mutable)
    }

    #[must_use]
    pub const fn own() -> Self {
        Self::new(ParamOwnership::Own, ParamMutability::Immutable)
    }

    #[must_use]
    pub const fn own_mut() -> Self {
        Self::new(ParamOwnership::Own, ParamMutability::Mutable)
    }

    #[must_use]
    pub const fn ownership(self) -> ParamOwnership {
        self.ownership
    }

    #[must_use]
    pub const fn mutability(self) -> ParamMutability {
        self.mutability
    }

    #[must_use]
    pub const fn is_owned(self) -> bool {
        matches!(self.ownership, ParamOwnership::Own)
    }

    #[must_use]
    pub const fn is_borrowed(self) -> bool {
        matches!(self.ownership, ParamOwnership::Borrow)
    }

    #[must_use]
    pub const fn is_mutable(self) -> bool {
        matches!(self.mutability, ParamMutability::Mutable)
    }

    #[must_use]
    pub const fn is_shared_borrow(self) -> bool {
        self.is_borrowed() && !self.is_mutable()
    }

    #[must_use]
    pub const fn is_mut_borrow(self) -> bool {
        self.is_borrowed() && self.is_mutable()
    }
}

/// Describes how a type behaves with respect to ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipKind {
    /// Value is copied on assignment (primitives: int, float, bool)
    Copy,
    /// Value is moved on assignment (str, compound types, classes)
    Move,
}

impl Type {
    #[must_use]
    pub fn reversible(element_type: Type) -> Self {
        Self::Alias {
            name: "Reversible".to_string(),
            type_args: vec![element_type.clone()],
            body: Box::new(Self::Iterable(Box::new(element_type))),
        }
    }

    fn reversible_alias_element_type(ty: &Type) -> Option<Type> {
        let Type::Alias {
            name,
            type_args,
            body,
        } = ty
        else {
            return None;
        };
        if name != "Reversible" {
            return None;
        }
        if let Some(elem) = type_args.first() {
            return Some(elem.clone());
        }
        let Type::Iterable(elem) = body.resolve_alias() else {
            return None;
        };
        Some(*elem.clone())
    }

    fn homogeneous_tuple_iter_element_type(elems: &[Type]) -> Option<Type> {
        let first = elems.first()?.clone();
        if elems.iter().all(|elem| elem == &first) {
            Some(first)
        } else {
            None
        }
    }

    fn class_method<'a>(
        methods: &'a [(String, FunctionType)],
        method_name: &str,
    ) -> Option<&'a FunctionType> {
        methods.iter().find_map(
            |(name, ft)| {
                if name == method_name {
                    Some(ft)
                } else {
                    None
                }
            },
        )
    }

    fn option_like_member_type(ty: &Type) -> Option<Type> {
        let Type::Union(members) = ty.resolve_alias() else {
            return None;
        };
        let has_none = members
            .iter()
            .any(|member| matches!(member.resolve_alias(), Type::None));
        let non_none: Vec<Type> = members
            .iter()
            .filter(|member| !matches!(member.resolve_alias(), Type::None))
            .cloned()
            .collect();
        if has_none && non_none.len() == 1 {
            non_none.first().cloned()
        } else {
            None
        }
    }

    fn class_next_element_type(
        class_name: &str,
        methods: &[(String, FunctionType)],
    ) -> Option<Type> {
        let next_ft = Self::class_method(methods, "__next__")?;
        if !next_ft.params.is_empty() {
            return None;
        }
        let elem = Self::option_like_member_type(next_ft.return_type.as_ref())?;
        if matches!(elem.resolve_alias(), Type::Class { name, .. } if name == class_name) {
            return None;
        }
        Some(elem)
    }

    fn class_iter_element_type(
        class_name: &str,
        methods: &[(String, FunctionType)],
    ) -> Option<Type> {
        let iter_ft = Self::class_method(methods, "__iter__")?;
        if !iter_ft.params.is_empty() {
            return None;
        }
        match iter_ft.return_type.resolve_alias() {
            Type::Iterator(elem) | Type::Iterable(elem) => Some(*elem.clone()),
            Type::Class {
                name,
                methods: ret_methods,
                ..
            } => {
                if name == class_name {
                    Self::class_next_element_type(class_name, methods)
                } else {
                    Self::class_iter_element_type(name, ret_methods)
                        .or_else(|| Self::class_next_element_type(name, ret_methods))
                }
            }
            _ => None,
        }
    }

    fn class_reversed_element_type(
        class_name: &str,
        methods: &[(String, FunctionType)],
    ) -> Option<Type> {
        let reversed_ft = Self::class_method(methods, "__reversed__")?;
        if !reversed_ft.params.is_empty() {
            return None;
        }
        match reversed_ft.return_type.resolve_alias() {
            Type::Iterator(elem) | Type::Iterable(elem) => Some(*elem.clone()),
            Type::Class {
                name,
                methods: ret_methods,
                ..
            } => {
                if name == class_name {
                    Self::class_next_element_type(class_name, methods)
                } else {
                    Self::class_iter_element_type(name, ret_methods)
                        .or_else(|| Self::class_next_element_type(name, ret_methods))
                }
            }
            _ => None,
        }
    }

    /// Construct a non-generic type alias wrapper.
    pub fn alias(name: impl Into<String>, body: Type) -> Self {
        Self::Alias {
            name: name.into(),
            type_args: Vec::new(),
            body: Box::new(body),
        }
    }

    /// Returns the ownership kind for this type.
    ///
    /// - Primitives (`Int`, `Float`, `Bool`) are `Copy`.
    /// - `Str` and compound types are `Move`.
    /// - `None` is `Copy` (it's a zero-sized type).
    /// - `Any` is `Move` (conservative).
    /// - `Never` is `Copy` (unreachable).
    /// - `Function` is `Copy` (function pointers).
    pub fn ownership(&self) -> OwnershipKind {
        match self {
            Self::Int
            | Self::FixedInt(_)
            | Self::Float
            | Self::Bool
            | Self::None
            | Self::Never
            | Self::Range
            | Self::Decimal => OwnershipKind::Copy,
            Self::LiteralInt(_) | Self::LiteralBool(_) => OwnershipKind::Copy,
            Self::Function(_) | Self::AsyncFunction(_) => OwnershipKind::Copy,
            Self::Str
            | Self::Bytes
            | Self::Any
            | Self::Coroutine(_, _)
            | Self::Task(_, _)
            | Self::TaskResult(_, _)
            | Self::TimeoutResult(_)
            | Self::BlockingTask(_, _)
            | Self::Awaitable(_)
            | Self::AsyncIterator(_, _)
            | Self::AsyncGenerator(_, _)
            | Self::List(_)
            | Self::Dict(_, _)
            | Self::Set(_)
            | Self::Iterable(_)
            | Self::Iterator(_) => OwnershipKind::Move,
            Self::Tuple(elems) => {
                if elems
                    .iter()
                    .all(|elem| elem.ownership() == OwnershipKind::Copy)
                {
                    OwnershipKind::Copy
                } else {
                    OwnershipKind::Move
                }
            }
            Self::LiteralStr(_) => OwnershipKind::Move,
            Self::Unknown => OwnershipKind::Move,
            Self::Class { .. } => OwnershipKind::Move,
            Self::Result(_, _) => OwnershipKind::Move,
            Self::Protocol { .. } => OwnershipKind::Move,
            Self::Newtype { inner, .. } => inner.ownership(),
            Self::TypeVar(_) => OwnershipKind::Move, // conservative: treat as Move
            Self::Callable(..) => OwnershipKind::Copy, // function pointers are Copy
            Self::Enum { .. } => OwnershipKind::Copy, // enums are Copy (repr(i64))
            Self::BigInt => OwnershipKind::Move,     // heap-allocated, not Copy
            Self::BigDecimal => OwnershipKind::Move,
            // Union/Intersection: Move if any member is Move
            Self::Union(members) | Self::Intersection(members) => {
                if members.iter().any(|m| m.ownership() == OwnershipKind::Move) {
                    OwnershipKind::Move
                } else {
                    OwnershipKind::Copy
                }
            }
            // Alias: delegate to the underlying type
            Self::Alias { body, .. } => body.ownership(),
        }
    }

    /// Returns the Sifr source name for this type.
    pub fn display_name(&self) -> String {
        match self {
            Self::Int => "int".to_string(),
            Self::FixedInt(fixed) => fixed.source_name().to_string(),
            Self::Float => "float".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Str => "str".to_string(),
            Self::Bytes => "bytes".to_string(),
            Self::None => "None".to_string(),
            Self::Function(_) => "function".to_string(),
            Self::List(elem) => format!("list[{}]", elem.display_name()),
            Self::Dict(key, val) => format!("dict[{}, {}]", key.display_name(), val.display_name()),
            Self::Set(elem) => format!("set[{}]", elem.display_name()),
            Self::Tuple(elems) => {
                let parts: Vec<String> = elems.iter().map(Self::display_name).collect();
                format!("tuple[{}]", parts.join(", "))
            }
            Self::Range => "range".to_string(),
            Self::Iterable(elem) => format!("Iterable[{}]", elem.display_name()),
            Self::Iterator(elem) => format!("Iterator[{}]", elem.display_name()),
            Self::Any => "Any".to_string(),
            Self::Never => "Never".to_string(),
            Self::Union(members) => {
                let parts: Vec<String> = members.iter().map(Self::display_name).collect();
                parts.join(" | ")
            }
            Self::Intersection(members) => {
                let parts: Vec<String> = members.iter().map(Self::display_name).collect();
                parts.join(" & ")
            }
            Self::LiteralInt(v) => format!("{v}"),
            Self::LiteralStr(v) => format!("\"{v}\""),
            Self::LiteralBool(v) => {
                if *v {
                    "True".to_string()
                } else {
                    "False".to_string()
                }
            }
            Self::Alias {
                name, type_args, ..
            } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    let args = type_args
                        .iter()
                        .map(Self::display_name)
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{name}[{args}]")
                }
            }
            Self::Unknown => "Unknown".to_string(),
            Self::Class { name, .. } => name.clone(),
            Self::Result(ok, err) => {
                format!("Result[{}, {}]", ok.display_name(), err.display_name())
            }
            Self::AsyncFunction(ft) => {
                let params = ft
                    .params
                    .iter()
                    .map(|(_, ty, _)| ty.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "AsyncFunction[[{}], {}]",
                    params,
                    ft.return_type.display_name()
                )
            }
            Self::Coroutine(ok, err) => {
                format!("Coroutine[{}, {}]", ok.display_name(), err.display_name())
            }
            Self::Task(ok, err) => format!("Task[{}, {}]", ok.display_name(), err.display_name()),
            Self::TaskResult(ok, err) => {
                format!("TaskResult[{}, {}]", ok.display_name(), err.display_name())
            }
            Self::TimeoutResult(err) => format!("TimeoutResult[{}]", err.display_name()),
            Self::BlockingTask(ok, err) => {
                format!(
                    "BlockingTask[{}, {}]",
                    ok.display_name(),
                    err.display_name()
                )
            }
            Self::Awaitable(result) => format!("Awaitable[{}]", result.display_name()),
            Self::AsyncIterator(item, err) => {
                format!(
                    "AsyncIterator[{}, {}]",
                    item.display_name(),
                    err.display_name()
                )
            }
            Self::AsyncGenerator(item, err) => {
                format!(
                    "AsyncGenerator[{}, {}]",
                    item.display_name(),
                    err.display_name()
                )
            }
            Self::Protocol { name, .. } => name.clone(),
            Self::Newtype { name, .. } => name.clone(),
            Self::TypeVar(name) => name.clone(),
            Self::Callable(params, _, ret) => {
                let parts: Vec<String> = params.iter().map(Self::display_name).collect();
                format!("Callable[[{}], {}]", parts.join(", "), ret.display_name())
            }
            Self::Enum { name, .. } => name.clone(),
            Self::BigInt => "bigint".to_string(),
            Self::Decimal => "decimal".to_string(),
            Self::BigDecimal => "bigdecimal".to_string(),
        }
    }

    /// Returns the Rust type name for code generation.
    ///
    /// For union types, this returns a generated enum name.
    /// The actual enum definition is emitted during code generation.
    pub fn rust_type(&self) -> String {
        match self {
            Self::Int => "i64".to_string(),
            Self::FixedInt(fixed) => fixed.rust_name().to_string(),
            Self::Float => "f64".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Str => "String".to_string(),
            Self::Bytes => "Vec<u8>".to_string(),
            Self::None => "()".to_string(),
            Self::List(elem) => format!("Vec<{}>", elem.rust_type()),
            Self::Dict(key, val) => format!("HashMap<{}, {}>", key.rust_type(), val.rust_type()),
            Self::Set(elem) => format!("HashSet<{}>", elem.rust_type()),
            Self::Tuple(elems) => {
                let parts: Vec<String> = elems.iter().map(Self::rust_type).collect();
                format!("({})", parts.join(", "))
            }
            Self::Range => "std::ops::Range<i64>".to_string(),
            Self::Iterable(elem) => format!("Vec<{}>", elem.rust_type()),
            Self::Iterator(elem) => format!("Box<dyn Iterator<Item = {}>>", elem.rust_type()),
            Self::Any => "Box<dyn std::any::Any>".to_string(),
            Self::Never => "!".to_string(),
            Self::Function(ft) => {
                let params: Vec<String> = ft.params.iter().map(|(_, t, _)| t.rust_type()).collect();
                let ret = ft.return_type.rust_type();
                format!("fn({}) -> {}", params.join(", "), ret)
            }
            // Literal types map to their base Rust type
            Self::LiteralInt(_) => "i64".to_string(),
            Self::LiteralStr(_) => "String".to_string(),
            Self::LiteralBool(_) => "bool".to_string(),
            // Union: special case for T | None -> Option<T>
            Self::Union(members) => {
                let non_none: Vec<&Type> = members
                    .iter()
                    .filter(|m| !matches!(m, Type::None))
                    .collect();
                let has_none = members.iter().any(|m| matches!(m, Type::None));
                if has_none && non_none.len() == 1 {
                    // T | None -> Option<T>
                    format!("Option<{}>", non_none[0].rust_type())
                } else {
                    // General union -> generated enum name
                    self.union_enum_name()
                }
            }
            Self::Intersection(_) => "Box<dyn std::any::Any>".to_string(),
            Self::Alias { body, .. } => body.rust_type(),
            Self::Unknown => "Box<dyn std::any::Any>".to_string(),
            Self::Class { name, .. } => name.clone(),
            Self::Result(ok, err) => format!("Result<{}, {}>", ok.rust_type(), err.rust_type()),
            Self::AsyncFunction(ft) => {
                let params: Vec<String> = ft.params.iter().map(|(_, t, _)| t.rust_type()).collect();
                let ret = ft.return_type.rust_type();
                format!("fn({}) -> {}", params.join(", "), ret)
            }
            Self::Coroutine(ok, err) => {
                format!(
                    "std::pin::Pin<Box<dyn std::future::Future<Output = Result<{}, {}>>>>",
                    ok.rust_type(),
                    err.rust_type()
                )
            }
            Self::Task(ok, err) => format!("Task<{}, {}>", ok.rust_type(), err.rust_type()),
            Self::TaskResult(ok, err) => {
                format!("TaskResult<{}, {}>", ok.rust_type(), err.rust_type())
            }
            Self::TimeoutResult(err) => format!("TimeoutResult<{}>", err.rust_type()),
            Self::BlockingTask(ok, err) => {
                format!("BlockingTask<{}, {}>", ok.rust_type(), err.rust_type())
            }
            Self::Awaitable(result) => format!(
                "std::pin::Pin<Box<dyn std::future::Future<Output = {}>>>",
                result.rust_type()
            ),
            Self::AsyncIterator(item, err) => {
                format!("AsyncIterator<{}, {}>", item.rust_type(), err.rust_type())
            }
            Self::AsyncGenerator(item, err) => {
                format!("AsyncGenerator<{}, {}>", item.rust_type(), err.rust_type())
            }
            Self::Protocol { name, .. } => format!("Box<dyn {name}>"),
            Self::Newtype { name, .. } => name.clone(),
            Self::TypeVar(name) => name.clone(), // Generic type parameter name (e.g., T)
            Self::Enum { name, .. } => name.clone(), // Enum type maps to its Rust enum name
            Self::BigInt => "BigInt".to_string(),
            Self::Decimal => "Decimal".to_string(),
            Self::BigDecimal => "BigDecimal".to_string(),
            Self::Callable(params, conventions, ret) => {
                let param_types: Vec<String> = params
                    .iter()
                    .zip(conventions.iter())
                    .map(|(t, conv)| {
                        let rust_ty = t.rust_type();
                        match conv {
                            convention
                                if convention.is_shared_borrow()
                                    && t.ownership() == OwnershipKind::Move =>
                            {
                                format!("&{rust_ty}")
                            }
                            convention
                                if convention.is_mut_borrow()
                                    && t.ownership() == OwnershipKind::Move =>
                            {
                                format!("&mut {rust_ty}")
                            }
                            _ => rust_ty,
                        }
                    })
                    .collect();
                let ret_type = ret.rust_type();
                if ret_type == "()" {
                    format!("impl Fn({})", param_types.join(", "))
                } else {
                    format!("impl Fn({}) -> {}", param_types.join(", "), ret_type)
                }
            }
        }
    }

    /// Generate the Rust type for use in struct fields.
    /// For most types this is the same as `rust_type()`, but `Callable` types
    /// emit `Box<dyn Fn(...)>` instead of `impl Fn(...)` because `impl Trait`
    /// is not allowed in struct field positions in Rust.
    pub fn rust_type_for_struct_field(&self) -> String {
        match self {
            Self::Callable(params, conventions, ret) => {
                let param_types: Vec<String> = params
                    .iter()
                    .zip(conventions.iter())
                    .map(|(t, conv)| {
                        let rust_ty = t.rust_type();
                        match conv {
                            convention
                                if convention.is_shared_borrow()
                                    && t.ownership() == OwnershipKind::Move =>
                            {
                                format!("&{rust_ty}")
                            }
                            convention
                                if convention.is_mut_borrow()
                                    && t.ownership() == OwnershipKind::Move =>
                            {
                                format!("&mut {rust_ty}")
                            }
                            _ => rust_ty,
                        }
                    })
                    .collect();
                let ret_type = ret.rust_type();
                if ret_type == "()" {
                    format!("Box<dyn Fn({})>", param_types.join(", "))
                } else {
                    format!("Box<dyn Fn({}) -> {}>", param_types.join(", "), ret_type)
                }
            }
            _ => self.rust_type(),
        }
    }

    /// Generate a Rust enum name for a union type.
    ///
    /// E.g., `int | str` -> `IntOrStr`, `int | str | bool` -> `IntOrStrOrBool`
    pub fn union_enum_name(&self) -> String {
        match self {
            Self::Union(members) => {
                let parts: Vec<String> = members
                    .iter()
                    .map(Self::type_to_enum_variant_prefix)
                    .collect();
                parts.join("Or")
            }
            _ => self.rust_type(),
        }
    }

    /// Get the enum variant name for a type when it appears in a union enum.
    pub fn union_variant_name(&self) -> String {
        Self::type_to_enum_variant_prefix(self)
    }

    /// Helper: map a type to a `PascalCase` name for enum variant/name generation.
    fn type_to_enum_variant_prefix(ty: &Type) -> String {
        match ty {
            Type::Int => "Int".to_string(),
            Type::FixedInt(fixed) => fixed.variant_prefix().to_string(),
            Type::Float => "Float".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Str => "Str".to_string(),
            Type::Bytes => "Bytes".to_string(),
            Type::None => "None".to_string(),
            Type::LiteralInt(v) => format!("LitInt{v}"),
            Type::LiteralStr(v) => format!("Lit{}", capitalize(v)),
            Type::LiteralBool(v) => format!("Lit{}", if *v { "True" } else { "False" }),
            Type::List(_) => "List".to_string(),
            Type::Dict(_, _) => "Dict".to_string(),
            Type::Set(_) => "Set".to_string(),
            Type::Tuple(_) => "Tuple".to_string(),
            Type::Range => "Range".to_string(),
            Type::Iterable(_) => "Iterable".to_string(),
            Type::Iterator(_) => "Iterator".to_string(),
            Type::Function(_) => "Fn".to_string(),
            Type::AsyncFunction(_) => "AsyncFn".to_string(),
            Type::Coroutine(_, _) => "Coroutine".to_string(),
            Type::Task(_, _) => "Task".to_string(),
            Type::TaskResult(_, _) => "TaskResult".to_string(),
            Type::TimeoutResult(_) => "TimeoutResult".to_string(),
            Type::BlockingTask(_, _) => "BlockingTask".to_string(),
            Type::Awaitable(_) => "Awaitable".to_string(),
            Type::AsyncIterator(_, _) => "AsyncIterator".to_string(),
            Type::AsyncGenerator(_, _) => "AsyncGenerator".to_string(),
            Type::Unknown => "Unknown".to_string(),
            Type::Any => "Any".to_string(),
            Type::Never => "Never".to_string(),
            Type::Union(_) => "Union".to_string(),
            Type::Intersection(_) => "Intersection".to_string(),
            Type::Alias { name, .. } => capitalize(name),
            Type::Class { name, .. } => name.clone(),
            Type::Result(_, _) => "Result".to_string(),
            Type::Protocol { name, .. } => name.clone(),
            Type::Newtype { name, .. } => name.clone(),
            Type::TypeVar(name) => name.clone(),
            Type::Callable(..) => "Fn".to_string(),
            Type::Enum { name, .. } => name.clone(),
            Type::BigInt => "BigInt".to_string(),
            Type::Decimal => "Decimal".to_string(),
            Type::BigDecimal => "BigDecimal".to_string(),
        }
    }

    /// Check if this type is a numeric type.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Int
                | Self::Float
                | Self::LiteralInt(_)
                | Self::BigInt
                | Self::Decimal
                | Self::BigDecimal
        )
    }

    /// Check if this type is a union type.
    pub fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }

    /// Check if this type is a literal type.
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::LiteralInt(_) | Self::LiteralStr(_) | Self::LiteralBool(_)
        )
    }

    /// Check if this type is the Unknown type.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Get the members of a union type, or a single-element vec for non-unions.
    pub fn union_members(&self) -> Vec<Type> {
        match self {
            Self::Union(members) => members.clone(),
            other => vec![other.clone()],
        }
    }

    /// Resolve an alias to its underlying type.
    pub fn resolve_alias(&self) -> &Type {
        match self {
            Self::Alias { body, .. } => body.resolve_alias(),
            other => other,
        }
    }

    /// Returns the element type if this type is iterable, or None otherwise.
    pub fn iterable_element_type(&self) -> Option<Type> {
        if let Some(elem) = Self::reversible_alias_element_type(self) {
            return Some(elem);
        }
        match self {
            Self::Range => Some(Type::Int),
            Self::List(elem) => Some(*elem.clone()),
            Self::Set(elem) => Some(*elem.clone()),
            Self::Tuple(elems) => Self::homogeneous_tuple_iter_element_type(elems),
            Self::Str => Some(Type::Str),
            Self::Bytes => Some(Type::FixedInt(FixedIntType::U8)),
            Self::Dict(key, _) => Some(*key.clone()),
            Self::Iterable(elem) => Some(*elem.clone()),
            Self::Iterator(elem) => Some(*elem.clone()),
            Self::Class { name, methods, .. } => Self::class_iter_element_type(name, methods)
                .or_else(|| Self::class_next_element_type(name, methods)),
            Self::Alias { body, .. } => body.iterable_element_type(),
            _ => None,
        }
    }

    /// Returns the element type when this type participates in the iterator protocol.
    pub fn iterator_element_type(&self) -> Option<Type> {
        match self.resolve_alias() {
            Self::Iterator(elem) => Some(*elem.clone()),
            Self::Class { name, methods, .. } => Self::class_next_element_type(name, methods),
            _ => None,
        }
    }

    /// Returns iteration element/capability metadata when this type participates
    /// in the iterable protocol.
    pub fn iteration_metadata(&self) -> Option<IterationMetadata> {
        if let Some(elem) = Self::reversible_alias_element_type(self) {
            return Some(IterationMetadata {
                element_type: elem,
                capabilities: vec![
                    IterationCapability::MultiPass,
                    IterationCapability::DoubleEnded,
                ],
            });
        }
        match self.resolve_alias() {
            Self::Iterator(elem) => Some(IterationMetadata {
                element_type: *elem.clone(),
                capabilities: vec![IterationCapability::SinglePass],
            }),
            Self::List(elem) => Some(IterationMetadata {
                element_type: *elem.clone(),
                capabilities: vec![
                    IterationCapability::MultiPass,
                    IterationCapability::DoubleEnded,
                    IterationCapability::ExactSize,
                ],
            }),
            Self::Tuple(elems) => Some(IterationMetadata {
                element_type: Self::homogeneous_tuple_iter_element_type(elems)?,
                capabilities: vec![
                    IterationCapability::MultiPass,
                    IterationCapability::DoubleEnded,
                    IterationCapability::ExactSize,
                ],
            }),
            Self::Range => Some(IterationMetadata {
                element_type: Type::Int,
                capabilities: vec![
                    IterationCapability::MultiPass,
                    IterationCapability::DoubleEnded,
                    IterationCapability::ExactSize,
                ],
            }),
            Self::Str => Some(IterationMetadata {
                element_type: Type::Str,
                capabilities: vec![
                    IterationCapability::MultiPass,
                    IterationCapability::DoubleEnded,
                    IterationCapability::ExactSize,
                ],
            }),
            Self::Bytes => Some(IterationMetadata {
                element_type: Type::FixedInt(FixedIntType::U8),
                capabilities: vec![
                    IterationCapability::MultiPass,
                    IterationCapability::DoubleEnded,
                    IterationCapability::ExactSize,
                ],
            }),
            Self::Dict(key, _) => Some(IterationMetadata {
                element_type: *key.clone(),
                capabilities: vec![
                    IterationCapability::MultiPass,
                    IterationCapability::ExactSize,
                ],
            }),
            Self::Set(elem) => Some(IterationMetadata {
                element_type: *elem.clone(),
                capabilities: vec![
                    IterationCapability::MultiPass,
                    IterationCapability::ExactSize,
                ],
            }),
            Self::Iterable(elem) => Some(IterationMetadata {
                element_type: *elem.clone(),
                capabilities: vec![IterationCapability::MultiPass],
            }),
            Self::Class { name, methods, .. } => {
                let element_type = Self::class_iter_element_type(name, methods)
                    .or_else(|| Self::class_next_element_type(name, methods))?;
                let mut capabilities = if Self::class_next_element_type(name, methods).is_some() {
                    vec![IterationCapability::SinglePass]
                } else {
                    vec![IterationCapability::MultiPass]
                };
                if Self::class_reversed_element_type(name, methods).is_some() {
                    capabilities.push(IterationCapability::DoubleEnded);
                }
                Some(IterationMetadata {
                    element_type,
                    capabilities,
                })
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn supports_iteration_capability(&self, capability: IterationCapability) -> bool {
        self.iteration_metadata()
            .is_some_and(|metadata| metadata.supports(capability))
    }

    #[must_use]
    pub fn is_reversible_iterable(&self) -> bool {
        self.supports_iteration_capability(IterationCapability::DoubleEnded)
    }

    /// Returns the result type of indexing this type with the given index type.
    /// For list, dict, and str: returns Option[T] (T | None) for safe indexing.
    /// For tuple with literal index: returns the exact element type (no Option).
    pub fn index_result_type(&self, index_ty: &Type) -> Option<Type> {
        match self {
            Self::Alias {
                name: alias_name,
                body,
                ..
            } if alias_name.starts_with("__compat_defaultdict_") => {
                let Self::Dict(key, value) = body.resolve_alias() else {
                    return None;
                };
                if matches!(key.as_ref(), Type::Any | Type::Unknown)
                    || index_ty.is_assignable_to(key)
                    || key.is_assignable_to(index_ty)
                {
                    Some(*value.clone())
                } else {
                    None
                }
            }
            Self::Alias { body, .. } => body.index_result_type(index_ty),
            Self::List(elem) => {
                if index_ty == &Type::Int {
                    // Safe indexing: returns Option[T] = T | None
                    Some(Type::Union(vec![*elem.clone(), Type::None]))
                } else {
                    None
                }
            }
            Self::Dict(key, val) => {
                if matches!(key.as_ref(), Type::Any | Type::Unknown)
                    || index_ty.is_assignable_to(key)
                    || key.is_assignable_to(index_ty)
                {
                    // Safe indexing: returns Option[V] = V | None
                    Some(Type::Union(vec![*val.clone(), Type::None]))
                } else {
                    None
                }
            }
            Self::Tuple(elems) => {
                // Lowering resolves literal tuple indices precisely.
                // For non-literal int indices, conservatively return the union of element types.
                if index_ty == &Type::Int && !elems.is_empty() {
                    Some(make_union(elems.clone()))
                } else {
                    None
                }
            }
            Self::Str => {
                if index_ty == &Type::Int {
                    // Safe indexing: returns Option[str] = str | None
                    Some(Type::Union(vec![Type::Str, Type::None]))
                } else {
                    None
                }
            }
            Self::Bytes => {
                if index_ty == &Type::Int {
                    // Safe indexing: returns Option[uint8] = uint8 | None
                    Some(Type::Union(vec![
                        Type::FixedInt(FixedIntType::U8),
                        Type::None,
                    ]))
                } else {
                    None
                }
            }
            Self::Class { methods, .. } | Self::Protocol { methods, .. } => {
                let (_, getitem_ft) = methods.iter().find(|(name, _)| name == "__getitem__")?;
                if getitem_ft.params.len() != 1 {
                    return None;
                }
                let param_ty = &getitem_ft.params[0].1;
                if matches!(param_ty.resolve_alias(), Type::TypeVar(_))
                    || index_ty.is_assignable_to(param_ty)
                    || param_ty.is_assignable_to(index_ty)
                {
                    Some((*getitem_ft.return_type).clone())
                } else {
                    None
                }
            }
            // Union type: if T|None where T is indexable, unwrap and delegate
            Self::Union(members) => {
                let non_none: Vec<&Type> = members
                    .iter()
                    .filter(|m| !matches!(m, Type::None))
                    .collect();
                if non_none.len() == 1 {
                    non_none[0].index_result_type(index_ty)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Returns the result type of the `in` operator for this collection type.
    pub fn contains_element_type(&self) -> Option<Type> {
        match self {
            Self::Alias {
                name: alias_name,
                body,
                ..
            } if alias_name.starts_with("__compat_defaultdict_") => {
                let Self::Dict(key, _) = body.resolve_alias() else {
                    return None;
                };
                Some(*key.clone())
            }
            Self::Alias { body, .. } => body.contains_element_type(),
            Self::List(elem) => Some(*elem.clone()),
            Self::Set(elem) => Some(*elem.clone()),
            Self::Dict(key, _) => Some(*key.clone()),
            Self::Range => Some(Type::Int),
            Self::Str => Some(Type::Str),
            Self::Bytes => Some(Type::FixedInt(FixedIntType::U8)),
            Self::Union(members) => {
                let non_none: Vec<&Type> = members
                    .iter()
                    .filter(|member| !matches!(member, Type::None))
                    .collect();
                if non_none.len() == 1 {
                    non_none[0].contains_element_type()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if a value of type `self` can be assigned to a target of type `target`.
    pub fn is_assignable_to(&self, target: &Type) -> bool {
        fn contains_any(ty: &Type) -> bool {
            match ty {
                Type::Any => true,
                Type::List(elem)
                | Type::Set(elem)
                | Type::Iterable(elem)
                | Type::Iterator(elem) => contains_any(elem),
                Type::Dict(key, value) => contains_any(key) || contains_any(value),
                Type::Tuple(elems) | Type::Union(elems) | Type::Intersection(elems) => {
                    elems.iter().any(contains_any)
                }
                Type::Callable(params, _, ret) => {
                    params.iter().any(contains_any) || contains_any(ret)
                }
                Type::Result(ok, err)
                | Type::Coroutine(ok, err)
                | Type::Task(ok, err)
                | Type::TaskResult(ok, err)
                | Type::BlockingTask(ok, err)
                | Type::AsyncIterator(ok, err)
                | Type::AsyncGenerator(ok, err) => contains_any(ok) || contains_any(err),
                Type::TimeoutResult(err) => contains_any(err),
                Type::Awaitable(result) => contains_any(result),
                Type::Alias { body, .. } => contains_any(body),
                Type::Function(ft) | Type::AsyncFunction(ft) => {
                    ft.params.iter().any(|(_, ty, _)| contains_any(ty))
                        || contains_any(&ft.return_type)
                }
                Type::Class {
                    fields, methods, ..
                } => {
                    fields.iter().any(|(_, ty)| contains_any(ty))
                        || methods.iter().any(|(_, ft)| {
                            ft.params.iter().any(|(_, ty, _)| contains_any(ty))
                                || contains_any(&ft.return_type)
                        })
                }
                _ => false,
            }
        }

        fn same_alias_identity(left: &Type, right: &Type) -> bool {
            match (left, right) {
                (
                    Type::Alias {
                        name: left_name,
                        type_args: left_args,
                        ..
                    },
                    Type::Alias {
                        name: right_name,
                        type_args: right_args,
                        ..
                    },
                ) => left_name == right_name && left_args == right_args,
                _ => false,
            }
        }

        if same_alias_identity(self, target) {
            return true;
        }

        if let Some(target_elem) = Self::reversible_alias_element_type(target) {
            let Some(source_metadata) = self.iteration_metadata() else {
                return false;
            };
            return source_metadata.supports(IterationCapability::DoubleEnded)
                && source_metadata.element_type.is_assignable_to(&target_elem);
        }

        // Resolve aliases
        let source = self.resolve_alias();
        let target_resolved = target.resolve_alias();

        // Same-type nominal assignability, including Decimal/BigDecimal exact numeric types.
        if source == target_resolved {
            return true;
        }
        // Any is compatible with everything
        if matches!(source, Self::Any) || matches!(target_resolved, Self::Any) {
            return true;
        }
        // Never is assignable to everything
        if matches!(source, Self::Never) {
            return true;
        }
        // Unknown accepts any value (but operations on it are restricted)
        if matches!(target_resolved, Self::Unknown) {
            return true;
        }
        // Literal types are assignable to their base types
        match (source, target_resolved) {
            (Self::LiteralInt(_), Self::Int) => return true,
            (Self::LiteralStr(_), Self::Str) => return true,
            (Self::LiteralBool(_), Self::Bool) => return true,
            (Self::Int | Self::LiteralInt(_), Self::Float) => return true,
            _ => {}
        }
        // Source is assignable to a union target if assignable to any member
        if let Self::Union(target_members) = target_resolved {
            if target_members.iter().any(|m| source.is_assignable_to(m)) {
                return true;
            }
        }
        // Union source is assignable to target if ALL members are assignable
        if let Self::Union(source_members) = source {
            if source_members
                .iter()
                .all(|m| m.is_assignable_to(target_resolved))
            {
                return true;
            }
        }
        // Iterable/Iterator protocol assignability.
        match (source, target_resolved) {
            (Self::Iterator(src), Self::Iterator(dst) | Self::Iterable(dst))
            | (Self::Iterable(src), Self::Iterable(dst)) => return src.is_assignable_to(dst),
            (Self::List(src) | Self::Set(src), Self::Iterable(dst)) => {
                return src.is_assignable_to(dst);
            }
            (Self::Range, Self::Iterable(dst)) => return Type::Int.is_assignable_to(dst),
            (Self::Str, Self::Iterable(dst)) => return Type::Str.is_assignable_to(dst),
            (Self::Bytes, Self::Iterable(dst)) => {
                return Type::FixedInt(FixedIntType::U8).is_assignable_to(dst);
            }
            (Self::Dict(key, _), Self::Iterable(dst)) => return key.is_assignable_to(dst),
            (Self::Class { name, methods, .. }, Self::Iterator(dst)) => {
                return Self::class_next_element_type(name, methods)
                    .is_some_and(|source_elem| source_elem.is_assignable_to(dst));
            }
            (Self::Class { name, methods, .. }, Self::Iterable(dst)) => {
                return Self::class_iter_element_type(name, methods)
                    .or_else(|| Self::class_next_element_type(name, methods))
                    .is_some_and(|source_elem| source_elem.is_assignable_to(dst));
            }
            (Self::Tuple(items), Self::Iterable(dst)) => {
                let Some(elem) = Self::homogeneous_tuple_iter_element_type(items) else {
                    return false;
                };
                return elem.is_assignable_to(dst);
            }
            _ => {}
        }
        // Mutable collections are invariant in their element/key/value types.
        // Explicit `Any` inside the collection type remains an escape hatch.
        match (source, target_resolved) {
            (Self::List(a), Self::List(b)) => {
                a == b || same_alias_identity(a, b) || contains_any(a) || contains_any(b)
            }
            (Self::Set(a), Self::Set(b)) => {
                a == b || same_alias_identity(a, b) || contains_any(a) || contains_any(b)
            }
            (Self::Dict(ak, av), Self::Dict(bk, bv)) => {
                (ak == bk || same_alias_identity(ak, bk) || contains_any(ak) || contains_any(bk))
                    && (av == bv
                        || same_alias_identity(av, bv)
                        || contains_any(av)
                        || contains_any(bv))
            }
            (Self::Tuple(a), Self::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.is_assignable_to(y))
            }
            // Class types: nominal typing with inheritance support
            (
                Self::Class {
                    name: a,
                    parent_class: ref parent_a,
                    ..
                },
                Self::Class { name: b, .. },
            ) => {
                if a == b {
                    return true;
                }
                // `parent_class` stores the inheritance chain as `Parent|Grandparent|...`.
                if let Some(ref chain) = parent_a {
                    if chain.split('|').any(|ancestor| ancestor == b) {
                        return true;
                    }
                }
                false
            }
            // Result types: covariant in both T and E
            (Self::Result(ok_a, err_a), Self::Result(ok_b, err_b)) => {
                ok_a.is_assignable_to(ok_b) && err_a.is_assignable_to(err_b)
            }
            (Self::Coroutine(ok_a, err_a), Self::Coroutine(ok_b, err_b))
            | (Self::Task(ok_a, err_a), Self::Task(ok_b, err_b))
            | (Self::TaskResult(ok_a, err_a), Self::TaskResult(ok_b, err_b))
            | (Self::BlockingTask(ok_a, err_a), Self::BlockingTask(ok_b, err_b))
            | (Self::AsyncIterator(ok_a, err_a), Self::AsyncIterator(ok_b, err_b))
            | (Self::AsyncGenerator(ok_a, err_a), Self::AsyncGenerator(ok_b, err_b)) => {
                ok_a.is_assignable_to(ok_b) && err_a.is_assignable_to(err_b)
            }
            (Self::TimeoutResult(err_a), Self::TimeoutResult(err_b)) => {
                err_a.is_assignable_to(err_b)
            }
            (Self::Awaitable(a), Self::Awaitable(b)) => a.is_assignable_to(b),
            (Self::Coroutine(ok, err), Self::Awaitable(result))
                if matches!(err.resolve_alias(), Type::Never) =>
            {
                ok.is_assignable_to(result)
            }
            (Self::Coroutine(ok, err), Self::Awaitable(result)) => {
                Type::Result(ok.clone(), err.clone()).is_assignable_to(result)
            }
            (Self::Task(ok, err), Self::Awaitable(result)) => {
                Type::TaskResult(ok.clone(), err.clone()).is_assignable_to(result)
            }
            (Self::AsyncFunction(a), Self::AsyncFunction(b)) => {
                a.params.len() == b.params.len()
                    && a.params
                        .iter()
                        .zip(b.params.iter())
                        .all(|((_, at, _), (_, bt, _))| at.is_assignable_to(bt))
                    && a.return_type.is_assignable_to(&b.return_type)
            }
            // Protocol: a class satisfies a protocol if it has all required methods
            (
                Self::Class {
                    methods: class_methods,
                    ..
                },
                Self::Protocol {
                    methods: proto_methods,
                    ..
                },
            ) => proto_methods.iter().all(|(pname, pft)| {
                class_methods.iter().any(|(cname, cft)| {
                    cname == pname
                        && cft.params.len() == pft.params.len()
                        && cft
                            .params
                            .iter()
                            .zip(pft.params.iter())
                            .all(|((_, ct, _), (_, pt, _))| ct.is_assignable_to(pt))
                        && cft.return_type.is_assignable_to(&pft.return_type)
                })
            }),
            // Protocol types: same name means same protocol
            (Self::Protocol { name: a, .. }, Self::Protocol { name: b, .. }) => a == b,
            // Newtype: same name means same newtype (nominal)
            (Self::Newtype { name: a, .. }, Self::Newtype { name: b, .. }) => a == b,
            // TypeVar: only assignable to the same type parameter name.
            (Self::TypeVar(a), Self::TypeVar(b)) => a == b,
            // Callable: compatible if param and return types match
            (Self::Callable(params_a, _, ret_a), Self::Callable(params_b, _, ret_b)) => {
                params_a.len() == params_b.len()
                    && params_a
                        .iter()
                        .zip(params_b.iter())
                        .all(|(a, b)| a.is_assignable_to(b))
                    && ret_a.is_assignable_to(ret_b)
            }
            // A Function type is assignable to a Callable if signatures match
            (Self::Function(ft), Self::Callable(params, _, ret)) => {
                ft.params.len() == params.len()
                    && ft
                        .params
                        .iter()
                        .zip(params.iter())
                        .all(|((_, pt, _), ct)| pt.is_assignable_to(ct))
                    && ft.return_type.is_assignable_to(ret)
            }
            // Enum: nominal typing - same name means same enum
            (Self::Enum { name: a, .. }, Self::Enum { name: b, .. }) => a == b,
            // BigInt: only assignable to BigInt
            (Self::BigInt, Self::BigInt) => true,
            _ => false,
        }
    }
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_primitives_are_copy() {
        assert_eq!(Type::Int.ownership(), OwnershipKind::Copy);
        assert_eq!(
            Type::FixedInt(FixedIntType::U8).ownership(),
            OwnershipKind::Copy
        );
        assert_eq!(Type::Float.ownership(), OwnershipKind::Copy);
        assert_eq!(Type::Bool.ownership(), OwnershipKind::Copy);
        assert_eq!(Type::None.ownership(), OwnershipKind::Copy);
    }

    #[test]
    fn test_ownership_str_is_move() {
        assert_eq!(Type::Str.ownership(), OwnershipKind::Move);
    }

    #[test]
    fn test_rust_type_mapping() {
        assert_eq!(Type::Int.rust_type(), "i64");
        assert_eq!(Type::FixedInt(FixedIntType::I8).rust_type(), "i8");
        assert_eq!(Type::FixedInt(FixedIntType::I16).rust_type(), "i16");
        assert_eq!(Type::FixedInt(FixedIntType::I32).rust_type(), "i32");
        assert_eq!(Type::FixedInt(FixedIntType::I64).rust_type(), "i64");
        assert_eq!(Type::FixedInt(FixedIntType::U8).rust_type(), "u8");
        assert_eq!(Type::FixedInt(FixedIntType::U16).rust_type(), "u16");
        assert_eq!(Type::FixedInt(FixedIntType::U32).rust_type(), "u32");
        assert_eq!(Type::FixedInt(FixedIntType::U64).rust_type(), "u64");
        assert_eq!(Type::FixedInt(FixedIntType::ISize).rust_type(), "isize");
        assert_eq!(Type::FixedInt(FixedIntType::USize).rust_type(), "usize");
        assert_eq!(Type::Float.rust_type(), "f64");
        assert_eq!(Type::Bool.rust_type(), "bool");
        assert_eq!(Type::Str.rust_type(), "String");
        assert_eq!(Type::None.rust_type(), "()");
    }

    #[test]
    fn test_fixed_width_type_names_and_union_variants() {
        let fixed = Type::FixedInt(FixedIntType::U32);
        assert_eq!(fixed.display_name(), "uint32");
        assert_eq!(fixed.union_variant_name(), "Uint32");
    }

    #[test]
    fn test_fixed_width_current_int_builtin_widening_policy() {
        for fixed in [
            FixedIntType::I8,
            FixedIntType::I16,
            FixedIntType::I32,
            FixedIntType::U8,
            FixedIntType::U16,
            FixedIntType::U32,
        ] {
            assert!(fixed.supports_current_int_builtin_widening());
        }

        for fixed in [
            FixedIntType::I64,
            FixedIntType::U64,
            FixedIntType::ISize,
            FixedIntType::USize,
        ] {
            assert!(!fixed.supports_current_int_builtin_widening());
        }
    }

    #[test]
    fn test_fixed_width_current_scalar_promotion_policy() {
        for fixed in [
            FixedIntType::I8,
            FixedIntType::I16,
            FixedIntType::I32,
            FixedIntType::I64,
            FixedIntType::U8,
            FixedIntType::U16,
            FixedIntType::U32,
            FixedIntType::ISize,
        ] {
            assert!(fixed.supports_current_scalar_promotion_to_int());
        }

        for fixed in [FixedIntType::U64, FixedIntType::USize] {
            assert!(!fixed.supports_current_scalar_promotion_to_int());
        }
    }

    #[test]
    fn test_assignability() {
        assert!(Type::Int.is_assignable_to(&Type::Int));
        assert!(!Type::Int.is_assignable_to(&Type::Str));
        assert!(Type::Int.is_assignable_to(&Type::Float));
        assert!(Type::LiteralInt(42).is_assignable_to(&Type::Float));
        assert!(Type::Any.is_assignable_to(&Type::Int));
        assert!(Type::Int.is_assignable_to(&Type::Any));
        assert!(Type::Never.is_assignable_to(&Type::Int));
    }

    #[test]
    fn test_decimal_assignability_contract() {
        assert!(Type::Decimal.is_assignable_to(&Type::Decimal));
        assert!(Type::BigDecimal.is_assignable_to(&Type::BigDecimal));
        assert!(!Type::Decimal.is_assignable_to(&Type::BigDecimal));
        assert!(!Type::BigDecimal.is_assignable_to(&Type::Decimal));
    }

    #[test]
    fn test_typevar_assignability_is_strict() {
        assert!(Type::TypeVar("T".to_string()).is_assignable_to(&Type::TypeVar("T".to_string())));
        assert!(!Type::TypeVar("T".to_string()).is_assignable_to(&Type::TypeVar("U".to_string())));
        assert!(!Type::TypeVar("T".to_string()).is_assignable_to(&Type::Int));
        assert!(!Type::Int.is_assignable_to(&Type::TypeVar("T".to_string())));
    }

    #[test]
    fn test_list_type() {
        let list_int = Type::List(Box::new(Type::Int));
        assert_eq!(list_int.ownership(), OwnershipKind::Move);
        assert_eq!(list_int.display_name(), "list[int]");
        assert_eq!(list_int.rust_type(), "Vec<i64>");
        assert_eq!(list_int.iterable_element_type(), Some(Type::Int));
    }

    #[test]
    fn test_iterator_and_iterable_type_contract() {
        let iter_int = Type::Iterator(Box::new(Type::Int));
        let iterable_int = Type::Iterable(Box::new(Type::Int));
        let list_int = Type::List(Box::new(Type::Int));

        assert_eq!(iter_int.display_name(), "Iterator[int]");
        assert_eq!(iterable_int.display_name(), "Iterable[int]");
        assert_eq!(iter_int.iterable_element_type(), Some(Type::Int));
        assert_eq!(iterable_int.iterable_element_type(), Some(Type::Int));
        assert!(iter_int.is_assignable_to(&iterable_int));
        assert!(list_int.is_assignable_to(&iterable_int));
    }

    #[test]
    fn test_bytes_iterable_and_index_contract_uses_uint8() {
        let uint8 = Type::FixedInt(FixedIntType::U8);

        assert_eq!(Type::Bytes.iterable_element_type(), Some(uint8.clone()));
        assert_eq!(
            Type::Bytes
                .iteration_metadata()
                .map(|metadata| metadata.element_type),
            Some(uint8.clone())
        );
        assert_eq!(
            Type::Bytes.index_result_type(&Type::Int),
            Some(Type::Union(vec![uint8.clone(), Type::None]))
        );
        assert!(Type::Bytes.is_assignable_to(&Type::Iterable(Box::new(uint8))));
        assert!(!Type::Bytes.is_assignable_to(&Type::Iterable(Box::new(Type::Int))));
    }

    #[test]
    fn test_reversible_alias_contract() {
        let reversible_int = Type::reversible(Type::Int);
        let iterable_int = Type::Iterable(Box::new(Type::Int));
        let list_int = Type::List(Box::new(Type::Int));
        let set_int = Type::Set(Box::new(Type::Int));
        let iterator_int = Type::Iterator(Box::new(Type::Int));

        assert!(list_int.is_assignable_to(&reversible_int));
        assert!(!set_int.is_assignable_to(&reversible_int));
        assert!(!iterator_int.is_assignable_to(&reversible_int));
        assert!(reversible_int.is_assignable_to(&iterable_int));
        assert!(reversible_int.supports_iteration_capability(IterationCapability::DoubleEnded));
    }

    #[test]
    fn test_tuple_iterability_requires_homogeneous_elements() {
        let homogeneous = Type::Tuple(vec![Type::Int, Type::Int, Type::Int]);
        let heterogeneous = Type::Tuple(vec![Type::Int, Type::Str]);

        assert_eq!(homogeneous.iterable_element_type(), Some(Type::Int));
        assert_eq!(heterogeneous.iterable_element_type(), None);
        assert!(homogeneous.is_assignable_to(&Type::Iterable(Box::new(Type::Int))));
        assert!(!heterogeneous.is_assignable_to(&Type::Iterable(Box::new(Type::Any))));
    }

    #[test]
    fn test_class_with_iter_method_is_iterable() {
        let iterable_class = Type::Class {
            name: "Counter".to_string(),
            fields: vec![],
            methods: vec![(
                "__iter__".to_string(),
                FunctionType::new(vec![], Type::Iterator(Box::new(Type::Int))),
            )],
            parent_class: None,
        };

        assert_eq!(iterable_class.iterable_element_type(), Some(Type::Int));
        assert!(iterable_class.is_assignable_to(&Type::Iterable(Box::new(Type::Int))));
    }

    #[test]
    fn test_class_with_next_method_is_iterator_protocol() {
        let self_iter_type = Type::Class {
            name: "CounterIter".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let iterator_class = Type::Class {
            name: "CounterIter".to_string(),
            fields: vec![],
            methods: vec![
                (
                    "__iter__".to_string(),
                    FunctionType::new(vec![], self_iter_type),
                ),
                (
                    "__next__".to_string(),
                    FunctionType::new(vec![], Type::Union(vec![Type::Int, Type::None])),
                ),
            ],
            parent_class: None,
        };

        assert_eq!(iterator_class.iterator_element_type(), Some(Type::Int));
        assert!(iterator_class.is_assignable_to(&Type::Iterator(Box::new(Type::Int))));
        assert!(iterator_class.is_assignable_to(&Type::Iterable(Box::new(Type::Int))));
    }

    #[test]
    fn test_class_with_reversed_method_is_reversible_iterable() {
        let reversible_class = Type::Class {
            name: "Deck".to_string(),
            fields: vec![],
            methods: vec![
                (
                    "__iter__".to_string(),
                    FunctionType::new(vec![], Type::Iterator(Box::new(Type::Int))),
                ),
                (
                    "__reversed__".to_string(),
                    FunctionType::new(vec![], Type::Iterator(Box::new(Type::Int))),
                ),
            ],
            parent_class: None,
        };

        assert!(reversible_class.is_reversible_iterable());
        assert!(reversible_class.is_assignable_to(&Type::reversible(Type::Int)));
    }

    #[test]
    fn test_dict_type() {
        let dict_str_int = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
        assert_eq!(dict_str_int.ownership(), OwnershipKind::Move);
        assert_eq!(dict_str_int.display_name(), "dict[str, int]");
        assert_eq!(dict_str_int.rust_type(), "HashMap<String, i64>");
    }

    #[test]
    fn test_tuple_type() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Str]);
        assert_eq!(tuple.ownership(), OwnershipKind::Move);
        assert_eq!(tuple.display_name(), "tuple[int, str]");
        assert_eq!(tuple.rust_type(), "(i64, String)");
    }

    #[test]
    fn test_tuple_ownership_all_copy_is_copy() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Float]);
        assert_eq!(tuple.ownership(), OwnershipKind::Copy);
    }

    #[test]
    fn test_tuple_ownership_with_move_is_move() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Str]);
        assert_eq!(tuple.ownership(), OwnershipKind::Move);
    }

    #[test]
    fn test_collection_assignability() {
        let list_int = Type::List(Box::new(Type::Int));
        let list_int2 = Type::List(Box::new(Type::Int));
        let list_str = Type::List(Box::new(Type::Str));
        assert!(list_int.is_assignable_to(&list_int2));
        assert!(!list_int.is_assignable_to(&list_str));

        // Mutable collections are invariant.
        let list_int_or_str = Type::List(Box::new(Type::Union(vec![Type::Int, Type::Str])));
        assert!(!list_int.is_assignable_to(&list_int_or_str));

        let dict_int_int = Type::Dict(Box::new(Type::Int), Box::new(Type::Int));
        let dict_int_union = Type::Dict(
            Box::new(Type::Int),
            Box::new(Type::Union(vec![Type::Int, Type::Str])),
        );
        assert!(!dict_int_int.is_assignable_to(&dict_int_union));
    }

    #[test]
    fn test_class_assignability_supports_transitive_inheritance_chain() {
        let base = Type::Class {
            name: "Base".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let mid = Type::Class {
            name: "Mid".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("Base".to_string()),
        };
        let leaf = Type::Class {
            name: "Leaf".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("Mid|Base".to_string()),
        };

        assert!(leaf.is_assignable_to(&mid));
        assert!(leaf.is_assignable_to(&base));
    }

    #[test]
    fn test_error_assignability_requires_actual_error_ancestry() {
        let error = Type::Class {
            name: "Error".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let non_error_child = Type::Class {
            name: "Widget".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("BaseThing".to_string()),
        };
        let real_error_child = Type::Class {
            name: "ValueError".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };

        assert!(!non_error_child.is_assignable_to(&error));
        assert!(real_error_child.is_assignable_to(&error));
    }

    #[test]
    fn test_index_result_type() {
        let list_int = Type::List(Box::new(Type::Int));
        // Safe indexing returns Option[T] = T | None
        assert_eq!(
            list_int.index_result_type(&Type::Int),
            Some(Type::Union(vec![Type::Int, Type::None]))
        );
        assert_eq!(list_int.index_result_type(&Type::Str), None);

        let dict_any_int = Type::Dict(Box::new(Type::Any), Box::new(Type::Int));
        assert_eq!(
            dict_any_int.index_result_type(&Type::Str),
            Some(Type::Union(vec![Type::Int, Type::None]))
        );
    }

    #[test]
    fn test_contains_element_type_range_and_compat_defaultdict() {
        assert_eq!(Type::Range.contains_element_type(), Some(Type::Int));

        let compat_defaultdict = Type::Alias {
            name: "__compat_defaultdict_list".to_string(),
            type_args: Vec::new(),
            body: Box::new(Type::Dict(
                Box::new(Type::Str),
                Box::new(Type::List(Box::new(Type::Int))),
            )),
        };
        assert_eq!(compat_defaultdict.contains_element_type(), Some(Type::Str));
    }

    // --- Union type tests ---

    #[test]
    fn test_union_display_name() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert_eq!(u.display_name(), "int | str");
    }

    #[test]
    fn test_literal_display_name() {
        assert_eq!(Type::LiteralInt(42).display_name(), "42");
        assert_eq!(
            Type::LiteralStr("GET".to_string()).display_name(),
            "\"GET\""
        );
        assert_eq!(Type::LiteralBool(true).display_name(), "True");
        assert_eq!(Type::LiteralBool(false).display_name(), "False");
    }

    #[test]
    fn test_unknown_display_name() {
        assert_eq!(Type::Unknown.display_name(), "Unknown");
    }

    #[test]
    fn test_literal_assignable_to_base() {
        assert!(Type::LiteralInt(42).is_assignable_to(&Type::Int));
        assert!(Type::LiteralStr("GET".to_string()).is_assignable_to(&Type::Str));
        assert!(Type::LiteralBool(true).is_assignable_to(&Type::Bool));
    }

    #[test]
    fn test_literal_not_assignable_to_wrong_base() {
        assert!(!Type::LiteralInt(42).is_assignable_to(&Type::Str));
        assert!(!Type::LiteralStr("GET".to_string()).is_assignable_to(&Type::Int));
    }

    #[test]
    fn test_assignable_to_union() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert!(Type::Int.is_assignable_to(&u));
        assert!(Type::Str.is_assignable_to(&u));
        assert!(!Type::Bool.is_assignable_to(&u));
    }

    #[test]
    fn test_union_assignable_to_target() {
        // Union is assignable to target only if ALL members are assignable
        let u = Type::Union(vec![Type::Int, Type::Int]);
        assert!(u.is_assignable_to(&Type::Int));

        let u2 = Type::Union(vec![Type::Int, Type::Str]);
        assert!(!u2.is_assignable_to(&Type::Int));
    }

    #[test]
    fn test_anything_assignable_to_unknown() {
        assert!(Type::Int.is_assignable_to(&Type::Unknown));
        assert!(Type::Str.is_assignable_to(&Type::Unknown));
        assert!(Type::Bool.is_assignable_to(&Type::Unknown));
    }

    #[test]
    fn test_union_rust_type_option() {
        let optional_str = Type::Union(vec![Type::None, Type::Str]);
        assert_eq!(optional_str.rust_type(), "Option<String>");
    }

    #[test]
    fn test_union_rust_type_enum() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert_eq!(u.rust_type(), "IntOrStr");
    }

    #[test]
    fn test_union_ownership() {
        // Union with Move member -> Move
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert_eq!(u.ownership(), OwnershipKind::Move);
        // Union with only Copy members -> Copy
        let u2 = Type::Union(vec![Type::Int, Type::Bool]);
        assert_eq!(u2.ownership(), OwnershipKind::Copy);
    }

    #[test]
    fn test_alias_resolves() {
        let alias = Type::Alias {
            name: "UserId".to_string(),
            type_args: Vec::new(),
            body: Box::new(Type::Int),
        };
        assert_eq!(alias.display_name(), "UserId");
        assert_eq!(alias.rust_type(), "i64");
        assert!(alias.is_assignable_to(&Type::Int));
    }

    #[test]
    fn test_literal_is_numeric() {
        assert!(Type::LiteralInt(42).is_numeric());
        assert!(!Type::LiteralStr("x".to_string()).is_numeric());
    }

    #[test]
    fn test_is_union() {
        assert!(Type::Union(vec![Type::Int, Type::Str]).is_union());
        assert!(!Type::Int.is_union());
    }

    #[test]
    fn test_is_literal() {
        assert!(Type::LiteralInt(42).is_literal());
        assert!(Type::LiteralStr("x".to_string()).is_literal());
        assert!(Type::LiteralBool(true).is_literal());
        assert!(!Type::Int.is_literal());
    }

    #[test]
    fn test_never_assignable_to_union() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert!(Type::Never.is_assignable_to(&u));
    }
}
