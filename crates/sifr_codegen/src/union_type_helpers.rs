use crate::{
    RustEmitter, RustEnumVariant, RustExpr, RustItem, RustLiteral, RustMatchArm, RustParam,
    RustStmt, RustType, RustTypeParam, Visibility, sifr_type_to_rust_type,
};
use sifr_ir::HirModule;
use sifr_type_system::ParamConvention;
use sifr_type_system::Type;

impl RustEmitter {
    fn project_nominal_path(&self, identity: Option<&str>, name: &str) -> Option<&str> {
        if self.project_nominal_type_paths.is_empty() {
            return None;
        }
        let builtin_identity = identity
            .is_none()
            .then(|| crate::builtin_error_identity(name))
            .flatten();
        let key = identity.or(builtin_identity.as_deref()).unwrap_or(name);
        if let Some(path) = self.project_nominal_type_paths.get(key) {
            return Some(path);
        }
        if identity.is_some_and(sifr_type_system::is_crate_root_rust_nominal_identity) {
            return None;
        }
        panic!("missing crate-root path for project union nominal identity '{key}'");
    }

    fn project_union_member_rust_type(&self, ty: &Type) -> RustType {
        let resolved = crate::resolve_alias_type_for_plain_call(ty);
        if let Some(member) = resolved.optional_member_type() {
            return RustType::Option(Box::new(self.project_union_member_rust_type(&member)));
        }
        match resolved {
            class @ Type::Class {
                identity,
                type_args,
                name,
                ..
            } => {
                if class.is_python_object_contract() || class.is_python_resource_identity_contract()
                {
                    return sifr_type_to_rust_type(resolved);
                }
                let Some(path) = self.project_nominal_path(identity.as_deref(), name) else {
                    return sifr_type_to_rust_type(resolved);
                };
                if type_args.is_empty() {
                    RustType::Named(path.to_string())
                } else {
                    RustType::Generic {
                        base: path.to_string(),
                        params: type_args
                            .iter()
                            .map(|arg| self.project_union_member_rust_type(arg))
                            .collect(),
                    }
                }
            }
            Type::Protocol { identity, name, .. } => self
                .project_nominal_path(identity.as_deref(), name)
                .map_or_else(
                    || sifr_type_to_rust_type(resolved),
                    |path| {
                        RustType::Boxed(Box::new(RustType::DynTrait {
                            trait_: crate::RustTrait::Named {
                                name: path.to_string(),
                                params: Vec::new(),
                                associated_types: Vec::new(),
                            },
                            auto_traits: Vec::new(),
                        }))
                    },
                ),
            Type::Newtype { identity, name, .. } | Type::Enum { identity, name, .. } => self
                .project_nominal_path(identity.as_deref(), name)
                .map_or_else(
                    || sifr_type_to_rust_type(resolved),
                    |path| RustType::Named(path.to_string()),
                ),
            Type::List(inner) | Type::Iterable(inner) => {
                RustType::Vec(Box::new(self.project_union_member_rust_type(inner)))
            }
            Type::Dict(key, value) => RustType::HashMap(
                Box::new(self.project_union_member_rust_type(key)),
                Box::new(self.project_union_member_rust_type(value)),
            ),
            Type::Set(inner) => {
                RustType::HashSet(Box::new(self.project_union_member_rust_type(inner)))
            }
            Type::Tuple(items) => RustType::Tuple(
                items
                    .iter()
                    .map(|item| self.project_union_member_rust_type(item))
                    .collect(),
            ),
            Type::Result(ok, error) => RustType::Result(
                Box::new(self.project_union_member_rust_type(ok)),
                Box::new(self.project_union_member_rust_type(error)),
            ),
            _ => sifr_type_to_rust_type(resolved),
        }
    }

    /// Collect all union types from the module that need enum definitions,
    /// and build a map of function signatures for call-site wrapping.
    pub(crate) fn collect_union_types(&mut self, module: &HirModule) {
        for func in &module.functions {
            // Record function signature with conventions
            let param_info: Vec<(Type, ParamConvention)> = func
                .params
                .iter()
                .map(|p| (p.ty.clone(), p.convention))
                .collect();
            self.func_signatures
                .insert(func.name.clone(), (param_info, func.return_type.clone()));

            // Track generator functions (contain yield statements)
            if crate::body_contains_yield(&func.body) {
                self.generator_functions.insert(func.name.clone());
            }

            // Check params
            for param in &func.params {
                self.register_union_type(&param.ty);
            }
            // Check return type
            self.register_union_type(&func.return_type);
            for ty in crate::hir_analysis::queries::collect_let_declared_types(&func.body) {
                self.register_union_type(&ty);
            }
            self.register_expression_union_types(&func.body);
            for ty in crate::hir_analysis::queries::collect_try_error_carriers(&func.body) {
                self.register_try_error_carrier(&ty);
            }
        }
        // Also scan class method bodies and register their signatures
        for class in &module.classes {
            for (_, field_type) in &class.fields {
                self.register_union_type(field_type);
            }
            let mut has_constructor = false;
            for method in &class.methods {
                // Register method signature under ClassName::method_name
                let param_info: Vec<(Type, ParamConvention)> = method
                    .params
                    .iter()
                    .map(|p| {
                        let conv = if method.name == "new" {
                            ParamConvention::own()
                        } else {
                            p.convention
                        };
                        (p.ty.clone(), conv)
                    })
                    .collect();
                self.func_signatures.insert(
                    format!("{}::{}", class.name, method.name),
                    (param_info, method.return_type.clone()),
                );
                if method.name == "new" {
                    has_constructor = true;
                }

                for param in &method.params {
                    self.register_union_type(&param.ty);
                }
                self.register_union_type(&method.return_type);
                for ty in crate::hir_analysis::queries::collect_let_declared_types(&method.body) {
                    self.register_union_type(&ty);
                }
                self.register_expression_union_types(&method.body);
                for ty in crate::hir_analysis::queries::collect_try_error_carriers(&method.body) {
                    self.register_try_error_carrier(&ty);
                }
            }
            if !has_constructor {
                // Classes without an explicit `new` still get an auto-generated constructor.
                // Register it so call sites can apply ownership conventions correctly.
                let ctor_params = class
                    .fields
                    .iter()
                    .map(|(_, ty)| (ty.clone(), ParamConvention::own()))
                    .collect::<Vec<_>>();
                self.func_signatures.insert(
                    format!("{}::new", class.name),
                    (
                        ctor_params,
                        Type::Class {
                            identity: None,
                            type_args: Vec::new(),
                            name: class.name.clone(),
                            fields: class.fields.clone(),
                            methods: Vec::new(),
                            parent_class: class.semantic_parent_chain(),
                        },
                    ),
                );
            }
        }
    }

    fn register_expression_union_types(&mut self, body: &[crate::HirStmt]) {
        let mut types = Vec::new();
        crate::hir_analysis::traversal::walk_stmts(
            body,
            crate::hir_analysis::traversal::TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
            &mut |_| {},
            &mut |expr| types.push(expr.ty().clone()),
        );
        for ty in types {
            self.register_union_type(&ty);
        }
    }

    pub(crate) fn register_union_type(&mut self, ty: &Type) {
        self.register_union_type_with_usage(ty, true);
    }

    fn register_union_type_with_usage(&mut self, ty: &Type, ordinary_value: bool) {
        let resolved = crate::resolve_alias_type_for_plain_call(ty);
        // A raw `T | None` is an Option wrapper whose payload grouping is
        // significant. Only canonicalize unions that are represented by an
        // enum; the payload union is registered during the recursive walk.
        if resolved.optional_member_type().is_none() {
            if let Type::Union(members) = resolved {
                let canonical = sifr_type_system::make_union(members.clone());
                if canonical != *resolved {
                    self.register_union_type_with_usage(&canonical, ordinary_value);
                    return;
                }
            }
        }
        match resolved {
            Type::StructuralRecord(record) => {
                let name = crate::structural_identity_codegen::structural_record_rust_name(record);
                self.structural_record_types
                    .entry(name)
                    .or_insert_with(|| record.clone());
                for field in record.fields() {
                    self.register_union_type_with_usage(field.ty(), ordinary_value);
                }
            }
            Type::Union(members) => {
                let is_option = resolved.optional_member_type().is_some();
                if !is_option {
                    let enum_name = resolved.union_enum_name();
                    if ordinary_value {
                        self.ordinary_union_enums.insert(enum_name.clone());
                    }
                    self.union_enums
                        .entry(enum_name)
                        .or_insert_with(|| members.clone());
                }
                for member in members {
                    self.register_union_type_with_usage(member, ordinary_value);
                }
            }
            Type::List(inner)
            | Type::Set(inner)
            | Type::Iterable(inner)
            | Type::Iterator(inner)
            | Type::Awaitable(inner)
            | Type::Newtype { inner, .. }
            | Type::Failure(inner)
            | Type::TimeoutResult(inner)
            | Type::PythonBuffer(inner)
            | Type::PythonDlpackTensor(inner) => {
                self.register_union_type_with_usage(inner, ordinary_value);
            }
            Type::Dict(left, right)
            | Type::Result(left, right)
            | Type::Coroutine(left, right)
            | Type::Task(left, right)
            | Type::TaskResult(left, right)
            | Type::Select2(left, right)
            | Type::BlockingTask(left, right)
            | Type::JoinSet(left, right)
            | Type::AsyncIterator(left, right)
            | Type::AsyncGenerator(left, right) => {
                self.register_union_type_with_usage(left, ordinary_value);
                self.register_union_type_with_usage(right, ordinary_value);
            }
            Type::Tuple(items) | Type::Intersection(items) => {
                for item in items {
                    self.register_union_type_with_usage(item, ordinary_value);
                }
            }
            Type::Function(signature) | Type::AsyncFunction(signature) => {
                for (_, parameter, _) in &signature.params {
                    self.register_union_type_with_usage(parameter, ordinary_value);
                }
                self.register_union_type_with_usage(&signature.return_type, ordinary_value);
            }
            Type::Callable(parameters, _, result) | Type::AsyncCallable(parameters, _, result) => {
                for parameter in parameters {
                    self.register_union_type_with_usage(parameter, ordinary_value);
                }
                self.register_union_type_with_usage(result, ordinary_value);
            }
            _ => {}
        }
    }

    fn register_try_error_carrier(&mut self, ty: &Type) {
        if let Type::Union(members) = ty.resolve_alias() {
            let canonical = sifr_type_system::make_union(members.clone());
            if matches!(canonical, Type::Union(_)) {
                self.try_error_carrier_enums
                    .insert(canonical.union_enum_name());
            }
        }
        self.register_union_type_with_usage(ty, false);
    }

    /// Generate Rust enum definitions for all collected union types.
    pub(crate) fn generate_enum_definitions(&mut self) {
        // Sort enum names for deterministic output
        let mut enums: Vec<(String, Vec<Type>)> = self.union_enums.clone().into_iter().collect();
        enums.sort_by(|a, b| a.0.cmp(&b.0));

        self.enum_items.clear();
        for (enum_name, members) in enums {
            if self.suppressed_union_enum_definitions.contains(&enum_name) {
                continue;
            }
            let is_try_error_carrier = self.try_error_carrier_enums.contains(&enum_name);
            let supports_ordinary_value_traits =
                !is_try_error_carrier || self.ordinary_union_enums.contains(&enum_name);
            let variants = members
                .iter()
                .map(|member| RustEnumVariant {
                    name: member.union_variant_name(),
                    tuple_fields: vec![self.project_union_member_rust_type(member)],
                    fields: Vec::new(),
                    value: None,
                })
                .collect();
            let mut derives = Vec::new();
            if members.iter().all(Type::supports_debug_formatting) {
                derives.push("Debug".to_string());
            }
            if members.iter().all(Type::supports_derived_clone) {
                derives.push("Clone".to_string());
            }
            if supports_ordinary_value_traits
                && members.iter().all(Type::supports_structural_equality)
            {
                derives.push("PartialEq".to_string());
            }
            if supports_ordinary_value_traits && members.iter().all(Type::supports_hash_key) {
                derives.push("Eq".to_string());
                derives.push("Hash".to_string());
            }
            self.enum_items.push(RustItem::Enum {
                name: enum_name.clone(),
                visibility: Visibility::Private,
                derives,
                repr: None,
                variants,
            });
            if self.structural_union_enums.contains(&enum_name) {
                self.enum_items
                    .extend(self.structural_union_impls(&enum_name, &members));
            }
            if is_try_error_carrier {
                let conversion_items = members
                    .iter()
                    .map(|member| {
                        let member_type = self.project_union_member_rust_type(member);
                        let variant = member.union_variant_name();
                        RustItem::Impl {
                            target: enum_name.clone(),
                            type_params: Vec::new(),
                            trait_: Some(format!("From<{}>", crate::render_type(&member_type))),
                            items: vec![RustItem::Fn {
                                name: "from".to_string(),
                                visibility: Visibility::Private,
                                type_params: Vec::new(),
                                params: vec![RustParam::Named {
                                    name: "value".to_string(),
                                    ty: member_type,
                                }],
                                ret: Some(RustType::Named("Self".to_string())),
                                body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        enum_name.clone(),
                                        variant,
                                    ])),
                                    args: vec![RustExpr::Ident("value".to_string())],
                                }))],
                                is_async: false,
                            }],
                        }
                    })
                    .collect::<Vec<_>>();
                self.enum_items.extend(conversion_items);
            }

            let supports_display = members.iter().all(|member| {
                member.supports_display_formatting() || member.supports_debug_formatting()
            });
            if !supports_display {
                continue;
            }
            let arms: Vec<RustMatchArm> = members
                .iter()
                .map(|member| {
                    let variant = member.union_variant_name();
                    let fmt_spec = if member.supports_display_formatting() {
                        "{}"
                    } else {
                        "{:?}"
                    };
                    RustMatchArm {
                        pattern: format!("{enum_name}::{variant}(v)"),
                        bindings: Vec::new(),
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::MacroCall {
                            name: "write".to_string(),
                            args: vec![
                                RustExpr::Ident("f".to_string()),
                                RustExpr::Literal(RustLiteral::Str(fmt_spec.to_string())),
                                RustExpr::Ident("v".to_string()),
                            ],
                        }))],
                    }
                })
                .collect();

            self.enum_items.push(RustItem::Impl {
                target: enum_name,
                type_params: Vec::new(),
                trait_: Some("std::fmt::Display".to_string()),
                items: vec![RustItem::Fn {
                    name: "fmt".to_string(),
                    visibility: Visibility::Private,
                    type_params: Vec::new(),
                    params: vec![
                        RustParam::SelfParam { mutable: false },
                        RustParam::Named {
                            name: "f".to_string(),
                            ty: RustType::Ref {
                                mutable: true,
                                inner: Box::new(RustType::Named(
                                    "std::fmt::Formatter<'_>".to_string(),
                                )),
                            },
                        },
                    ],
                    ret: Some(RustType::Named("std::fmt::Result".to_string())),
                    body: vec![RustStmt::Match {
                        expr: RustExpr::Ident("self".to_string()),
                        arms,
                    }],
                    is_async: false,
                }],
            });
        }
    }

    fn structural_union_impls(&self, enum_name: &str, members: &[Type]) -> Vec<RustItem> {
        const STRUCTURAL: &str = "::sifr_runtime::interop::structural";
        let member_types = members
            .iter()
            .map(|member| {
                crate::Renderer::render_type_string(&self.project_union_member_rust_type(member))
            })
            .collect::<Vec<_>>();
        let shape_members = member_types
            .iter()
            .map(|member| format!("<{member} as {STRUCTURAL}::StructuralType>::shape_identity()"))
            .collect::<Vec<_>>()
            .join(", ");
        let construct_arms = members
            .iter()
            .zip(&member_types)
            .enumerate()
            .map(|(index, (member, member_type))| {
                let variant = member.union_variant_name();
                format!(
                    "{STRUCTURAL}::StructuralEdgeKind::ActiveMember {{ name: \"member\", index: {index} }} => <{member_type} as {STRUCTURAL}::StructuralConstruct>::structural_construct_at(source, child, token).map(Self::{variant}),"
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let project_arms = members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let variant = member.union_variant_name();
                format!(
                    "Self::{variant}(value) => {{ visitor.edge({STRUCTURAL}::StructuralEdge::new({STRUCTURAL}::StructuralEdgeKind::ActiveMember {{ name: \"member\", index: {index} }}))?; {STRUCTURAL}::StructuralProject::structural_project(value, visitor)?; }}"
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let construct_body = format!(
            "let description = source.node(node)?;\nif description.kind() != {STRUCTURAL}::StructuralKind::Union {{ return Err({STRUCTURAL}::StructuralContractError::KindMismatch); }}\nif description.nominal_identity().is_some() {{ return Err({STRUCTURAL}::StructuralContractError::MemberMismatch); }}\nlet [edge] = description.edges() else {{ return Err({STRUCTURAL}::StructuralContractError::ArityMismatch); }};\nlet edge_kind = edge.kind();\nlet child = edge.node();\nmatch edge_kind {{\n{construct_arms}\n_ => Err({STRUCTURAL}::StructuralContractError::MemberMismatch),\n}}"
        );
        let project_body = format!(
            "let control = visitor.enter({STRUCTURAL}::StructuralEnter::new({STRUCTURAL}::StructuralKind::Union, None, 1))?;\nif control == {STRUCTURAL}::VisitControl::Continue {{\nmatch self {{\n{project_arms}\n}}\n}}\nvisitor.exit({STRUCTURAL}::StructuralKind::Union)"
        );
        vec![
            RustItem::Impl {
                target: enum_name.to_string(),
                type_params: Vec::new(),
                trait_: Some(format!("{STRUCTURAL}::StructuralType")),
                items: vec![RustItem::Fn {
                    name: "shape_identity".to_string(),
                    visibility: Visibility::Private,
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret: Some(RustType::Named(format!("{STRUCTURAL}::ShapeIdentity"))),
                    body: vec![RustStmt::Verbatim(format!(
                        "{STRUCTURAL}::union(&[{shape_members}])"
                    ))],
                    is_async: false,
                }],
            },
            RustItem::Impl {
                target: enum_name.to_string(),
                type_params: Vec::new(),
                trait_: Some(format!("{STRUCTURAL}::StructuralConstruct")),
                items: vec![RustItem::Fn {
                    name: "structural_construct_at".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![RustTypeParam {
                        name: "S".to_string(),
                        bounds: vec![format!("{STRUCTURAL}::StructuralSource")],
                    }],
                    params: vec![
                        RustParam::Named {
                            name: "source".to_string(),
                            ty: RustType::Ref {
                                mutable: true,
                                inner: Box::new(RustType::Named("S".to_string())),
                            },
                        },
                        RustParam::Named {
                            name: "node".to_string(),
                            ty: RustType::Named(format!("{STRUCTURAL}::NodeId")),
                        },
                        RustParam::Named {
                            name: "token".to_string(),
                            ty: RustType::Named(format!("{STRUCTURAL}::ConstructToken")),
                        },
                    ],
                    ret: Some(RustType::Named(format!(
                        "Result<Self, {STRUCTURAL}::StructuralContractError>"
                    ))),
                    body: vec![RustStmt::Verbatim(construct_body)],
                    is_async: false,
                }],
            },
            RustItem::Impl {
                target: enum_name.to_string(),
                type_params: Vec::new(),
                trait_: Some(format!("{STRUCTURAL}::StructuralProject")),
                items: vec![RustItem::Fn {
                    name: "structural_project".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![
                        RustTypeParam {
                            name: "'value".to_string(),
                            bounds: Vec::new(),
                        },
                        RustTypeParam {
                            name: "V".to_string(),
                            bounds: vec![format!("{STRUCTURAL}::StructuralVisitor<'value>")],
                        },
                    ],
                    params: vec![
                        RustParam::SelfParamWithLifetime {
                            mutable: false,
                            lifetime: "'value".to_string(),
                        },
                        RustParam::Named {
                            name: "visitor".to_string(),
                            ty: RustType::Ref {
                                mutable: true,
                                inner: Box::new(RustType::Named("V".to_string())),
                            },
                        },
                    ],
                    ret: Some(RustType::Named("Result<(), V::Error>".to_string())),
                    body: vec![RustStmt::Verbatim(project_body)],
                    is_async: false,
                }],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affine_union_uses_debug_without_clone_or_display_bounds() {
        let buffer =
            Type::PythonBuffer(Box::new(Type::FixedInt(sifr_type_system::FixedIntType::U8)));
        let members = vec![Type::None, Type::Int, buffer.clone()];
        let mut emitter = RustEmitter::new();
        emitter.register_union_type(&Type::Union(members));
        emitter.generate_enum_definitions();

        let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
            panic!("first generated item should be the union enum");
        };
        assert_eq!(derives, &["Debug"]);
        let rendered = crate::render::render_items(&emitter.enum_items);
        assert!(rendered.contains(&format!("{}(v) =>", buffer.union_variant_name())));
        assert!(rendered.contains("write!(f, \"{:?}\", v)"));
        let none_arm = rendered
            .split(&format!("{}(v) =>", Type::None.union_variant_name()))
            .nth(1)
            .expect("None union arm should be rendered");
        assert!(none_arm[..none_arm.len().min(120)].contains("write!(f, \"{:?}\", v)"));
    }

    #[test]
    fn equality_capable_union_derives_the_required_rust_traits() {
        let union = Type::Union(vec![Type::Int, Type::Str]);
        let mut emitter = RustEmitter::new();
        emitter.register_union_type(&union);
        emitter.generate_enum_definitions();

        let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
            panic!("first generated item should be the union enum");
        };
        assert_eq!(derives, &["Debug", "Clone", "PartialEq", "Eq", "Hash"]);
        let rendered = crate::render::render_items(&emitter.enum_items);
        assert!(rendered.contains(&format!(
            "impl ::std::fmt::Display for {}",
            union.union_enum_name()
        )));
    }

    #[test]
    fn try_carrier_reuse_keeps_ordinary_union_traits() {
        let first_error = Type::Class {
            identity: Some("tests.first.Error".to_string()),
            type_args: Vec::new(),
            name: "Error".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let second_error = Type::Class {
            identity: Some("tests.second.Error".to_string()),
            type_args: Vec::new(),
            name: "Error".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let union = Type::Union(vec![first_error, second_error]);
        let mut emitter = RustEmitter::new();
        emitter.register_union_type(&union);
        emitter.register_try_error_carrier(&union);
        emitter.generate_enum_definitions();

        let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
            panic!("first generated item should be the union enum");
        };
        assert_eq!(derives, &["Debug", "Clone", "PartialEq", "Eq", "Hash"]);
    }

    #[test]
    fn try_carrier_and_ordinary_union_registration_keep_one_member_order() {
        let error = |name: &str| Type::Class {
            identity: Some(format!("a.{name}")),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let os_error = error("OSError");
        let zero_division = error("ZeroDivisionError");
        let ordinary = sifr_type_system::make_union(vec![os_error.clone(), zero_division.clone()]);
        let raw = Type::Union(vec![zero_division.clone(), os_error.clone()]);
        let carrier = crate::try_error_carrier::exact_try_error_carrier(&[zero_division, os_error])
            .expect("carrier should contain both errors");

        let render = |first: &Type| {
            let mut emitter = RustEmitter::new();
            emitter.register_union_type(first);
            emitter.register_try_error_carrier(&carrier);
            emitter.register_union_type(&ordinary);
            emitter.generate_enum_definitions();
            crate::render::render_items(&emitter.enum_items)
        };

        assert_eq!(render(&raw), render(&ordinary));
        assert_eq!(render(&carrier), render(&ordinary));
    }

    #[test]
    fn carrier_only_union_omits_unneeded_value_traits() {
        let union = Type::Union(vec![Type::Int, Type::Str]);
        let mut emitter = RustEmitter::new();
        emitter.register_try_error_carrier(&union);
        emitter.generate_enum_definitions();

        let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
            panic!("first generated item should be the union enum");
        };
        assert_eq!(derives, &["Debug", "Clone"]);
    }

    #[test]
    fn collapsed_try_carrier_does_not_record_a_plain_type_as_an_enum() {
        let class = Type::Class {
            identity: Some("tests.Error".to_string()),
            type_args: Vec::new(),
            name: "Error".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let snapshot = Type::Class {
            identity: Some("tests.Error".to_string()),
            type_args: Vec::new(),
            name: "Error".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let mut emitter = RustEmitter::new();

        emitter.register_try_error_carrier(&Type::Union(vec![class, snapshot]));

        assert!(emitter.try_error_carrier_enums.is_empty());
    }

    #[test]
    fn nested_optional_union_registers_its_grouped_payload_enum() {
        let inner = sifr_type_system::make_union(vec![Type::Int, Type::Str]);
        let raw = Type::Union(vec![inner.clone(), Type::None]);
        let expected_name = inner.union_enum_name();
        let mut emitter = RustEmitter::new();

        emitter.register_union_type(&raw);

        assert_eq!(
            crate::render_type(&crate::sifr_type_to_rust_type(&raw)),
            format!("Option<{expected_name}>")
        );
        assert!(emitter.union_enums.contains_key(&expected_name));
        assert_eq!(emitter.union_enums.len(), 1);
    }

    #[test]
    fn callable_bearing_union_requires_no_unavailable_formatting_trait() {
        let holder = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "CallbackHolder".to_string(),
            fields: vec![(
                "callback".to_string(),
                Type::Callable(
                    vec![Type::Int],
                    vec![ParamConvention::own()],
                    Box::new(Type::Int),
                ),
            )],
            methods: vec![],
            parent_class: None,
        };
        let mut emitter = RustEmitter::new();
        emitter.register_union_type(&Type::Union(vec![Type::Int, holder]));
        emitter.generate_enum_definitions();

        let RustItem::Enum { derives, .. } = &emitter.enum_items[0] else {
            panic!("first generated item should be the union enum");
        };
        assert!(derives.is_empty());
        assert_eq!(emitter.enum_items.len(), 1);
    }

    #[test]
    fn nested_result_error_union_is_registered() {
        let handler_error = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "HandlerError".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let python_error = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "PythonError".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let error = Type::Union(vec![handler_error, python_error]);
        let mut emitter = RustEmitter::new();

        emitter.register_union_type(&Type::Result(Box::new(Type::None), Box::new(error.clone())));

        assert_eq!(
            emitter.union_enums.get(&error.union_enum_name()),
            match error {
                Type::Union(ref members) => Some(members),
                _ => None,
            }
        );
    }
}
