use super::{FunctionType, OwnershipKind, Type};
use std::collections::HashSet;

pub(super) fn parent_chain_contains(parent_class: Option<&str>, ancestor: &str) -> bool {
    parent_class.is_some_and(|chain| chain.split('|').any(|parent| parent == ancestor))
}

impl Type {
    /// Stable recursion key for a nominal class representation. The declaring
    /// identity distinguishes same-named imports, while concrete arguments
    /// distinguish specializations whose emitted trait capabilities can differ.
    #[must_use]
    pub fn class_recursion_key(&self) -> Option<(String, Vec<Self>)> {
        let Self::Class {
            identity,
            name,
            type_args,
            ..
        } = self.resolve_alias()
        else {
            return None;
        };
        Some((identity.as_ref().unwrap_or(name).clone(), type_args.clone()))
    }

    /// Whether a value transitively owns an affine resource that Rust must not
    /// clone or compare through an aggregate derive.
    #[must_use]
    pub fn contains_affine_resource(&self) -> bool {
        self.contains_affine_resource_inner(&mut HashSet::new())
    }

    fn contains_affine_resource_inner(
        &self,
        visiting_classes: &mut HashSet<(String, Vec<Self>)>,
    ) -> bool {
        match self.resolve_alias() {
            Self::PythonBuffer(_)
            | Self::PythonArrow(_)
            | Self::PythonDlpackTensor(_)
            | Self::PythonDlpackStream => true,
            Self::List(element)
            | Self::Set(element)
            | Self::Iterable(element)
            | Self::Iterator(element)
            | Self::Awaitable(element)
            | Self::Failure(element)
            | Self::TimeoutResult(element)
            | Self::Newtype { inner: element, .. } => {
                element.contains_affine_resource_inner(visiting_classes)
            }
            Self::Dict(key, value)
            | Self::Result(key, value)
            | Self::Task(key, value)
            | Self::TaskResult(key, value)
            | Self::Coroutine(key, value)
            | Self::Select2(key, value)
            | Self::BlockingTask(key, value)
            | Self::JoinSet(key, value)
            | Self::AsyncIterator(key, value)
            | Self::AsyncGenerator(key, value) => {
                key.contains_affine_resource_inner(visiting_classes)
                    || value.contains_affine_resource_inner(visiting_classes)
            }
            Self::Tuple(elements)
            | Self::Template(elements)
            | Self::Union(elements)
            | Self::Intersection(elements) => elements
                .iter()
                .any(|element| element.contains_affine_resource_inner(visiting_classes)),
            Self::Class { fields, .. } => {
                let Some(key) = self.class_recursion_key() else {
                    return false;
                };
                if !visiting_classes.insert(key.clone()) {
                    return false;
                }
                let contains = fields
                    .iter()
                    .any(|(_, field)| field.contains_affine_resource_inner(visiting_classes));
                visiting_classes.remove(&key);
                contains
            }
            _ => false,
        }
    }

    /// Whether Rust aggregate generation may derive `Clone` for this value.
    #[must_use]
    pub fn supports_derived_clone(&self) -> bool {
        self.supports_derived_clone_inner(&mut HashSet::new())
    }

    fn supports_derived_clone_inner(
        &self,
        visiting_classes: &mut HashSet<(String, Vec<Self>)>,
    ) -> bool {
        match self.resolve_alias() {
            Self::Any
            | Self::Unknown
            | Self::PythonBuffer(_)
            | Self::PythonArrow(_)
            | Self::PythonDlpackTensor(_)
            | Self::PythonDlpackStream
            | Self::Protocol { .. }
            | Self::Callable(..)
            | Self::AsyncCallable(..)
            | Self::Coroutine(..)
            | Self::Task(..)
            | Self::TaskResult(..)
            | Self::Failure(_)
            | Self::TimeoutResult(_)
            | Self::Select2(..)
            | Self::BlockingTask(..)
            | Self::JoinSet(..)
            | Self::Awaitable(_)
            | Self::Iterator(_)
            | Self::AsyncIterator(..)
            | Self::AsyncGenerator(..)
            | Self::Template(_)
            | Self::Intersection(_) => false,
            Self::List(element)
            | Self::Set(element)
            | Self::Iterable(element)
            | Self::Newtype { inner: element, .. } => {
                element.supports_derived_clone_inner(visiting_classes)
            }
            Self::Dict(left, right) | Self::Result(left, right) => {
                left.supports_derived_clone_inner(visiting_classes)
                    && right.supports_derived_clone_inner(visiting_classes)
            }
            Self::Tuple(elements) | Self::Union(elements) => elements
                .iter()
                .all(|element| element.supports_derived_clone_inner(visiting_classes)),
            Self::Class {
                fields,
                parent_class,
                type_args,
                ..
            } => {
                if parent_chain_contains(parent_class.as_deref(), "NonSend") {
                    return false;
                }
                let Some(key) = self.class_recursion_key() else {
                    return false;
                };
                if !visiting_classes.insert(key.clone()) {
                    return true;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_derived_clone_inner(visiting_classes))
                    && type_args.iter().all(|argument| {
                        matches!(argument.resolve_alias(), Self::TypeVar(_))
                            || argument.supports_derived_clone_inner(visiting_classes)
                    });
                visiting_classes.remove(&key);
                supports
            }
            _ => true,
        }
    }

    /// Whether ordinary structural equality is valid for this value.
    #[must_use]
    pub fn supports_structural_equality(&self) -> bool {
        self.supports_structural_equality_inner(&mut HashSet::new())
    }

    fn supports_structural_equality_inner(
        &self,
        visiting_classes: &mut HashSet<(String, Vec<Self>)>,
    ) -> bool {
        match self.resolve_alias() {
            Self::Any
            | Self::Unknown
            | Self::Intersection(_)
            | Self::PythonBuffer(_)
            | Self::PythonArrow(_)
            | Self::PythonDlpackTensor(_)
            | Self::PythonDlpackStream
            | Self::Protocol { .. }
            | Self::Callable(..)
            | Self::AsyncCallable(..)
            | Self::Coroutine(..)
            | Self::Task(..)
            | Self::TaskResult(..)
            | Self::Failure(_)
            | Self::TimeoutResult(_)
            | Self::Select2(..)
            | Self::BlockingTask(..)
            | Self::JoinSet(..)
            | Self::Awaitable(_)
            | Self::Iterator(_)
            | Self::AsyncIterator(..)
            | Self::AsyncGenerator(..)
            | Self::Template(_) => false,
            Self::List(element) | Self::Iterable(element) => {
                element.supports_structural_equality_inner(visiting_classes)
            }
            Self::Set(element) => element.supports_hash_key_inner(visiting_classes),
            Self::Dict(key, value) => {
                key.supports_hash_key_inner(visiting_classes)
                    && value.supports_structural_equality_inner(visiting_classes)
            }
            Self::Result(ok, error) => {
                ok.supports_structural_equality_inner(visiting_classes)
                    && error.supports_structural_equality_inner(visiting_classes)
            }
            Self::Tuple(elements) | Self::Union(elements) => elements
                .iter()
                .all(|element| element.supports_structural_equality_inner(visiting_classes)),
            Self::Class {
                fields,
                methods,
                parent_class,
                type_args,
                ..
            } => {
                if methods.iter().any(|(method, _)| method == "__eq__") {
                    return true;
                }
                if parent_chain_contains(parent_class.as_deref(), "NonSend") {
                    return false;
                }
                let Some(key) = self.class_recursion_key() else {
                    return false;
                };
                if !visiting_classes.insert(key.clone()) {
                    return true;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_structural_equality_inner(visiting_classes))
                    && type_args.iter().all(|argument| {
                        matches!(argument.resolve_alias(), Self::TypeVar(_))
                            || argument.supports_structural_equality_inner(visiting_classes)
                    });
                visiting_classes.remove(&key);
                supports
            }
            Self::Newtype { inner, .. } => {
                inner.supports_structural_equality_inner(visiting_classes)
            }
            _ => true,
        }
    }

    #[must_use]
    pub fn reversible(element_type: Type) -> Self {
        Self::Alias {
            name: "Reversible".to_string(),
            type_args: vec![element_type.clone()],
            body: Box::new(Self::Iterable(Box::new(element_type))),
        }
    }

    pub(super) fn reversible_alias_element_type(ty: &Type) -> Option<Type> {
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

    pub(super) fn homogeneous_tuple_iter_element_type(elems: &[Type]) -> Option<Type> {
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
                if name == method_name { Some(ft) } else { None }
            },
        )
    }

    /// Return the payload of a raw `T | None` wrapper without flattening `T`.
    #[must_use]
    pub fn optional_member_type(&self) -> Option<Type> {
        let Type::Union(members) = self.resolve_alias() else {
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
            non_none.into_iter().next()
        } else {
            None
        }
    }

    pub(super) fn class_next_element_type(
        class_name: &str,
        methods: &[(String, FunctionType)],
    ) -> Option<Type> {
        let next_ft = Self::class_method(methods, "__next__")?;
        if !next_ft.params.is_empty() {
            return None;
        }
        let elem = next_ft.return_type.optional_member_type()?;
        if matches!(elem.resolve_alias(), Type::Class { name, .. } if name == class_name) {
            return None;
        }
        Some(elem)
    }

    pub(super) fn class_iter_element_type(
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

    pub(super) fn class_reversed_element_type(
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
            | Self::JoinSet(_, _)
            | Self::Awaitable(_)
            | Self::AsyncIterator(_, _)
            | Self::AsyncGenerator(_, _)
            | Self::PythonBuffer(_)
            | Self::PythonArrow(_)
            | Self::PythonDlpackTensor(_)
            | Self::PythonDlpackStream
            | Self::List(_)
            | Self::Dict(_, _)
            | Self::Set(_)
            | Self::Template(_)
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
            Self::Callable(..) | Self::AsyncCallable(..) => OwnershipKind::Copy,
            Self::Enum { .. } => OwnershipKind::Copy, // enums are Copy (repr(i64))
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
            Self::Template(elems) if elems.is_empty() => "Template".to_string(),
            Self::Template(elems) => {
                let parts: Vec<String> = elems.iter().map(Self::display_name).collect();
                format!("Template[{}]", parts.join(", "))
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
            Self::Class { name, .. } if name == "JoinItemId" => "JoinItemId".to_string(),
            Self::Class { name, .. } if name == "CancelOutcome" => "CancelOutcome".to_string(),
            Self::Class {
                name, type_args, ..
            } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    format!(
                        "{}[{}]",
                        name,
                        type_args
                            .iter()
                            .map(Self::display_name)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
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
            Self::JoinSet(ok, err) => {
                format!("JoinSet[{}, {}]", ok.display_name(), err.display_name())
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
            Self::PythonBuffer(element) => {
                format!("python.Buffer[{}]", element.display_name())
            }
            Self::PythonArrow(kind) => format!("python.{}", kind.source_name()),
            Self::PythonDlpackTensor(element) => {
                format!("python.DlpackTensor[{}]", element.display_name())
            }
            Self::PythonDlpackStream => "python.DlpackStream".to_string(),
            Self::Protocol { name, .. } => name.clone(),
            Self::Newtype { name, .. } => name.clone(),
            Self::TypeVar(name) => name.clone(),
            Self::Callable(params, _, ret) => {
                let parts: Vec<String> = params.iter().map(Self::display_name).collect();
                format!("Callable[[{}], {}]", parts.join(", "), ret.display_name())
            }
            Self::AsyncCallable(params, _, ret) => {
                let parts: Vec<String> = params.iter().map(Self::display_name).collect();
                format!(
                    "AsyncCallable[[{}], {}]",
                    parts.join(", "),
                    ret.display_name()
                )
            }
            Self::Enum { name, .. } => name.clone(),
            Self::Decimal => "decimal".to_string(),
            Self::BigDecimal => "bigdecimal".to_string(),
        }
    }
}
