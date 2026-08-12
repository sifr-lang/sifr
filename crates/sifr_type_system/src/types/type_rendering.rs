use super::{FixedIntType, IterationCapability, IterationMetadata, Type};
use crate::union::make_union;

impl Type {
    /// Whether two class types name the same declaration, independent of
    /// their concrete generic arguments.
    #[must_use]
    pub fn is_same_class_declaration(&self, other: &Self) -> bool {
        let (
            Type::Class {
                identity: left_identity,
                name: left_name,
                ..
            },
            Type::Class {
                identity: right_identity,
                name: right_name,
                ..
            },
        ) = (self.resolve_alias(), other.resolve_alias())
        else {
            return false;
        };
        left_identity.as_ref().unwrap_or(left_name) == right_identity.as_ref().unwrap_or(right_name)
    }

    /// Whether two class types name the same declaration and carry the same
    /// concrete generic specialization. Local import spellings may differ.
    #[must_use]
    pub fn is_same_class_specialization(&self, other: &Self) -> bool {
        fn slot_is_invariant(left: &Type, right: &Type) -> bool {
            left == right
                || matches!(left.resolve_alias(), Type::Any)
                || matches!(right.resolve_alias(), Type::Any)
                || (left.is_assignable_to(right) && right.is_assignable_to(left))
        }

        let (
            Type::Class {
                type_args: left_type_args,
                ..
            },
            Type::Class {
                type_args: right_type_args,
                ..
            },
        ) = (self.resolve_alias(), other.resolve_alias())
        else {
            return false;
        };
        if !self.is_same_class_declaration(other) || left_type_args.len() != right_type_args.len() {
            return false;
        }
        left_type_args
            .iter()
            .zip(right_type_args)
            .all(|(left, right)| slot_is_invariant(left, right))
    }

    /// Whether a union contains two incompatible concrete specializations of
    /// one nominal generic class.
    #[must_use]
    pub fn has_conflicting_class_specializations(&self) -> bool {
        let Type::Union(members) = self.resolve_alias() else {
            return false;
        };
        members.iter().enumerate().any(|(index, left)| {
            members[index + 1..].iter().any(|right| {
                let (
                    Type::Class {
                        identity: left_identity,
                        name: left_name,
                        ..
                    },
                    Type::Class {
                        identity: right_identity,
                        name: right_name,
                        ..
                    },
                ) = (left.resolve_alias(), right.resolve_alias())
                else {
                    return false;
                };
                left_identity.as_ref().unwrap_or(left_name)
                    == right_identity.as_ref().unwrap_or(right_name)
                    && !left.is_same_class_specialization(right)
            })
        })
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
            } if alias_name.starts_with("__sifr_defaultdict_") => {
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
            } if alias_name.starts_with("__sifr_defaultdict_") => {
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

    /// Whether this type transitively contains an unresolved `Unknown` or an
    /// explicitly unchecked `Any` slot.
    #[must_use]
    pub fn contains_unknown_or_any(&self) -> bool {
        self.contains_dynamic_slot(true)
    }

    fn contains_any(&self) -> bool {
        self.contains_dynamic_slot(false)
    }

    fn contains_dynamic_slot(&self, include_unknown: bool) -> bool {
        match self {
            Self::Any => true,
            Self::Unknown => include_unknown,
            Self::List(element)
            | Self::Set(element)
            | Self::Iterable(element)
            | Self::Iterator(element)
            | Self::Awaitable(element)
            | Self::Failure(element)
            | Self::TimeoutResult(element) => element.contains_dynamic_slot(include_unknown),
            Self::Dict(key, value)
            | Self::Result(key, value)
            | Self::Coroutine(key, value)
            | Self::Task(key, value)
            | Self::TaskResult(key, value)
            | Self::Select2(key, value)
            | Self::BlockingTask(key, value)
            | Self::JoinSet(key, value)
            | Self::AsyncIterator(key, value)
            | Self::AsyncGenerator(key, value) => {
                key.contains_dynamic_slot(include_unknown)
                    || value.contains_dynamic_slot(include_unknown)
            }
            Self::Tuple(elements) | Self::Union(elements) | Self::Intersection(elements) => {
                elements
                    .iter()
                    .any(|element| element.contains_dynamic_slot(include_unknown))
            }
            Self::Callable(params, _, result) | Self::AsyncCallable(params, _, result) => {
                params
                    .iter()
                    .any(|param| param.contains_dynamic_slot(include_unknown))
                    || result.contains_dynamic_slot(include_unknown)
            }
            Self::Alias { body, .. } => body.contains_dynamic_slot(include_unknown),
            Self::Function(function) | Self::AsyncFunction(function) => {
                function
                    .params
                    .iter()
                    .any(|(_, ty, _)| ty.contains_dynamic_slot(include_unknown))
                    || function.return_type.contains_dynamic_slot(include_unknown)
            }
            Self::Class {
                fields, methods, ..
            } => {
                fields
                    .iter()
                    .any(|(_, ty)| ty.contains_dynamic_slot(include_unknown))
                    || methods.iter().any(|(_, function)| {
                        function
                            .params
                            .iter()
                            .any(|(_, ty, _)| ty.contains_dynamic_slot(include_unknown))
                            || function.return_type.contains_dynamic_slot(include_unknown)
                    })
            }
            _ => false,
        }
    }

    /// Check if a value of type `self` can be assigned to a target of type `target`.
    pub fn is_assignable_to(&self, target: &Type) -> bool {
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

        fn invariant_slot_compatible(left: &Type, right: &Type) -> bool {
            left == right
                || same_alias_identity(left, right)
                || left.contains_any()
                || right.contains_any()
                || (left.is_assignable_to(right) && right.is_assignable_to(left))
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
            (Self::List(a), Self::List(b)) => invariant_slot_compatible(a, b),
            (Self::Set(a), Self::Set(b)) => invariant_slot_compatible(a, b),
            (Self::Dict(ak, av), Self::Dict(bk, bv)) => {
                invariant_slot_compatible(ak, bk) && invariant_slot_compatible(av, bv)
            }
            (Self::Tuple(a), Self::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.is_assignable_to(y))
            }
            // Class types: nominal typing with inheritance support
            (
                Self::Class {
                    name: a,
                    identity: identity_a,
                    parent_class: ref parent_a,
                    ..
                },
                Self::Class {
                    name: b,
                    identity: identity_b,
                    ..
                },
            ) => {
                let identity_a = identity_a.as_ref().unwrap_or(a);
                let identity_b = identity_b.as_ref().unwrap_or(b);
                if identity_a == identity_b {
                    return source.is_same_class_specialization(target_resolved);
                }
                // `parent_class` stores the inheritance chain as `Parent|Grandparent|...`.
                if let Some(ref chain) = parent_a {
                    if chain.split('|').any(|ancestor| ancestor == identity_b) {
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
            | (Self::Select2(ok_a, err_a), Self::Select2(ok_b, err_b))
            | (Self::BlockingTask(ok_a, err_a), Self::BlockingTask(ok_b, err_b))
            | (Self::JoinSet(ok_a, err_a), Self::JoinSet(ok_b, err_b))
            | (Self::AsyncIterator(ok_a, err_a), Self::AsyncIterator(ok_b, err_b))
            | (Self::AsyncGenerator(ok_a, err_a), Self::AsyncGenerator(ok_b, err_b)) => {
                ok_a.is_assignable_to(ok_b) && err_a.is_assignable_to(err_b)
            }
            (Self::TimeoutResult(err_a), Self::TimeoutResult(err_b)) => {
                err_a.is_assignable_to(err_b)
            }
            (Self::Failure(err_a), Self::Failure(err_b)) => err_a.is_assignable_to(err_b),
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
            (
                Self::AsyncCallable(params_a, conventions_a, ret_a),
                Self::AsyncCallable(params_b, conventions_b, ret_b),
            ) => {
                params_a.len() == params_b.len()
                    && conventions_a == conventions_b
                    && params_a
                        .iter()
                        .zip(params_b.iter())
                        .all(|(a, b)| a.is_assignable_to(b))
                    && ret_a.is_assignable_to(ret_b)
            }
            (Self::AsyncFunction(ft), Self::AsyncCallable(params, conventions, ret)) => {
                ft.params.len() == params.len()
                    && ft
                        .params
                        .iter()
                        .zip(params.iter().zip(conventions.iter()))
                        .all(|((_, pt, source_convention), (ct, target_convention))| {
                            source_convention == target_convention && pt.is_assignable_to(ct)
                        })
                    && ft.return_type.is_assignable_to(ret)
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
                        && match (cft.receiver, pft.receiver) {
                            (Some(implementation), Some(protocol)) => {
                                implementation.satisfies_protocol(protocol)
                            }
                            (None, None) => true,
                            _ => false,
                        }
                        && cft.params.len() == pft.params.len()
                        && cft
                            .params
                            .iter()
                            .zip(pft.params.iter())
                            .all(|((_, ct, _), (_, pt, _))| ct.is_assignable_to(pt))
                        && cft.return_type.is_assignable_to(&pft.return_type)
                })
            }),
            // Nominal types compare their canonical declaration identity when available.
            (
                Self::Protocol {
                    identity: a_identity,
                    name: a,
                    ..
                },
                Self::Protocol {
                    identity: b_identity,
                    name: b,
                    ..
                },
            ) => a_identity.as_deref().unwrap_or(a) == b_identity.as_deref().unwrap_or(b),
            (
                Self::Newtype {
                    identity: a_identity,
                    name: a,
                    ..
                },
                Self::Newtype {
                    identity: b_identity,
                    name: b,
                    ..
                },
            ) => a_identity.as_deref().unwrap_or(a) == b_identity.as_deref().unwrap_or(b),
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
            (
                Self::Enum {
                    identity: a_identity,
                    name: a,
                    ..
                },
                Self::Enum {
                    identity: b_identity,
                    name: b,
                    ..
                },
            ) => a_identity.as_deref().unwrap_or(a) == b_identity.as_deref().unwrap_or(b),
            // BigInt: only assignable to BigInt
            (Self::BigInt, Self::BigInt) => true,
            _ => false,
        }
    }
}
