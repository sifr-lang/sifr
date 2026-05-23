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
            | Self::Failure(_)
            | Self::TimeoutResult(_)
            | Self::Select2(_, _)
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
            Self::Failure(err) => format!("Failure[{}]", err.display_name()),
            Self::TimeoutResult(err) => format!("TimeoutResult[{}]", err.display_name()),
            Self::Select2(first, second) => {
                format!(
                    "Select2[{}, {}]",
                    first.display_name(),
                    second.display_name()
                )
            }
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
            Self::Failure(err) => format!("Failure<{}>", err.rust_type()),
            Self::TimeoutResult(err) => format!("TimeoutResult<{}>", err.rust_type()),
            Self::Select2(first, second) => {
                format!("Select2<{}, {}>", first.rust_type(), second.rust_type())
            }
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

}
