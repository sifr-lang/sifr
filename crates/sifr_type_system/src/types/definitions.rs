/// Affine resource kind exported through the Arrow C Data Interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PythonArrowKind {
    Array,
    Schema,
    Stream,
    DeviceArray,
    DeviceStream,
}

impl PythonArrowKind {
    #[must_use]
    pub const fn source_name(self) -> &'static str {
        match self {
            Self::Array => "ArrowArray",
            Self::Schema => "ArrowSchema",
            Self::Stream => "ArrowStream",
            Self::DeviceArray => "ArrowDeviceArray",
            Self::DeviceStream => "ArrowDeviceStream",
        }
    }

    #[must_use]
    pub const fn rust_name(self) -> &'static str {
        match self {
            Self::Array => "PythonArrowArray",
            Self::Schema => "PythonArrowSchema",
            Self::Stream => "PythonArrowStream",
            Self::DeviceArray => "PythonArrowDeviceArray",
            Self::DeviceStream => "PythonArrowDeviceStream",
        }
    }
}

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
    /// Materialized task failure evidence with secondary cleanup/sibling errors.
    Failure(Box<Type>),
    /// Ordinary error result used by task timeout wrappers.
    TimeoutResult(Box<Type>),
    /// Binary first-completion result for task.select.
    Select2(Box<Type>, Box<Type>),
    /// Explicit blocking-offload observer handle.
    BlockingTask(Box<Type>, Box<Type>),
    /// Dynamically-growable homogeneous task collection.
    JoinSet(Box<Type>, Box<Type>),
    /// Structural awaitability protocol.
    Awaitable(Box<Type>),
    /// Structural async iteration protocol.
    AsyncIterator(Box<Type>, Box<Type>),
    /// User-defined async generator object.
    AsyncGenerator(Box<Type>, Box<Type>),
    /// Affine, non-send view acquired through the Python buffer protocol.
    PythonBuffer(Box<Type>),
    /// Affine, non-send capsule owner acquired through the Arrow C Data Interface.
    PythonArrow(PythonArrowKind),
    /// Affine, non-send tensor owner acquired through the `DLPack` protocol.
    PythonDlpackTensor(Box<Type>),
    /// Affine, non-send consumer stream token used for `DLPack` synchronization.
    PythonDlpackStream,
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
        /// Stable declaration identity used for nominal typing across import
        /// and re-export paths. `None` means the local declaration name is the
        /// identity (the common single-module case).
        identity: Option<String>,
        /// Concrete generic arguments in declaration order. Empty for a
        /// non-generic class.
        type_args: Vec<Type>,
        name: String,
        fields: Vec<(String, Type)>,
        methods: Vec<(String, FunctionType)>,
        parent_class: Option<String>,
    },

    // --- Protocols, Operators, Discriminated Unions ---
    /// Protocol type: structural interface that maps to Rust `trait`.
    /// Any class with the required methods satisfies the protocol.
    Protocol {
        identity: Option<String>,
        name: String,
        methods: Vec<(String, FunctionType)>,
    },

    /// Newtype wrapper around a primitive type.
    /// `class Port(int)` -> `struct Port(i64)`
    Newtype {
        identity: Option<String>,
        name: String,
        inner: Box<Type>,
    },

    // --- Generics ---
    /// Type variable: a generic type parameter (e.g., `T` in `def first[T](items: list[T]) -> T`)
    TypeVar(String),

    /// Callable type: `Callable[[int, str], bool]` -> `fn(i64, String) -> bool`
    /// Fields: (`parameter_types`, `parameter_conventions`, `return_type`).
    Callable(Vec<Type>, Vec<ParamConvention>, Box<Type>),
    /// Async callable type: `AsyncCallable[[int], str]` produces an awaitable
    /// whose terminal value is `str`. This is distinct from `AsyncFunction`,
    /// which identifies an async function item rather than a callable value.
    AsyncCallable(Vec<Type>, Vec<ParamConvention>, Box<Type>),

    // --- Enum Types ---
    /// Enum type: `class Color(Enum): RED = 1; GREEN = 2; BLUE = 3`
    /// Maps to a Rust `#[repr(i64)] enum Color { RED = 1, GREEN = 2, BLUE = 3 }`
    Enum {
        identity: Option<String>,
        name: String,
        variants: Vec<(String, Option<i64>)>,
    },

    // --- Decimal Types ---
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
    /// Whether scalar arithmetic promotes this fixed-width value to exact `int`.
    #[must_use]
    pub const fn supports_current_scalar_promotion_to_int(self) -> bool {
        true
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
    /// Instance receiver convention. Free/static/class functions have no receiver.
    pub receiver: Option<ReceiverConvention>,
    /// Parameter names, types, and conventions
    pub params: Vec<(String, Type, ParamConvention)>,
    /// Return type
    pub return_type: Box<Type>,
}

impl FunctionType {
    /// Attach an instance-receiver convention to this method signature.
    #[must_use]
    pub fn with_receiver(mut self, receiver: ReceiverConvention) -> Self {
        self.receiver = Some(receiver);
        self
    }

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
            receiver: None,
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
            receiver: None,
            params,
            return_type: Box::new(return_type),
        }
    }
}

/// How an instance method receives its receiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReceiverConvention {
    /// Borrow the receiver immutably.
    SharedBorrow,
    /// Borrow the receiver mutably.
    MutableBorrow,
    /// Consume the receiver without granting a mutable local binding.
    Owned,
    /// Consume the receiver and grant a mutable local binding.
    OwnedMutable,
}

impl ReceiverConvention {
    #[must_use]
    pub const fn is_owned(self) -> bool {
        matches!(self, Self::Owned | Self::OwnedMutable)
    }

    #[must_use]
    pub const fn is_mutable(self) -> bool {
        matches!(self, Self::MutableBorrow | Self::OwnedMutable)
    }

    /// Whether an implementation receiver can satisfy a protocol declaration.
    ///
    /// A shared implementation is valid for a mutable protocol because it
    /// requires less access. Consuming receivers remain invariant.
    #[must_use]
    pub const fn satisfies_protocol(self, protocol: Self) -> bool {
        matches!(
            (self, protocol),
            (Self::SharedBorrow, Self::SharedBorrow | Self::MutableBorrow)
                | (Self::MutableBorrow, Self::MutableBorrow)
                | (Self::Owned, Self::Owned)
                | (Self::OwnedMutable, Self::OwnedMutable)
        )
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

#[cfg(test)]
mod receiver_tests {
    use super::*;

    #[test]
    fn function_receiver_is_separate_from_ordinary_parameters() {
        let signature = FunctionType::new(vec![("value".to_string(), Type::Int)], Type::None)
            .with_receiver(ReceiverConvention::MutableBorrow);

        assert_eq!(signature.receiver, Some(ReceiverConvention::MutableBorrow));
        assert_eq!(signature.params.len(), 1);
        assert_eq!(signature.params[0].0, "value");
    }

    #[test]
    fn free_function_constructors_do_not_invent_receivers() {
        assert_eq!(
            FunctionType::all_borrow(Vec::new(), Type::None).receiver,
            None
        );
        assert_eq!(FunctionType::new(Vec::new(), Type::None).receiver, None);
    }

    #[test]
    fn protocol_receiver_variance_allows_only_no_stronger_access() {
        assert!(
            ReceiverConvention::SharedBorrow.satisfies_protocol(ReceiverConvention::SharedBorrow)
        );
        assert!(
            ReceiverConvention::SharedBorrow.satisfies_protocol(ReceiverConvention::MutableBorrow)
        );
        assert!(
            ReceiverConvention::MutableBorrow.satisfies_protocol(ReceiverConvention::MutableBorrow)
        );
        assert!(
            !ReceiverConvention::MutableBorrow.satisfies_protocol(ReceiverConvention::SharedBorrow)
        );
        assert!(ReceiverConvention::Owned.satisfies_protocol(ReceiverConvention::Owned));
        assert!(
            ReceiverConvention::OwnedMutable.satisfies_protocol(ReceiverConvention::OwnedMutable)
        );
        assert!(!ReceiverConvention::Owned.satisfies_protocol(ReceiverConvention::OwnedMutable));
        assert!(!ReceiverConvention::OwnedMutable.satisfies_protocol(ReceiverConvention::Owned));
    }
}
