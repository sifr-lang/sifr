use crate::{RustEmitter, RustItem, RustParam, RustStmt, RustType, RustTypeParam, Visibility};
use sifr_ir::{HirClass, HirModule, RustInteropDecoratorKind};
use sifr_type_system::Type;
use std::collections::{BTreeSet, HashSet};

const STRUCTURAL: &str = "::sifr_runtime::interop::structural";

impl RustEmitter {
    pub(crate) fn emit_structural_record_impls(&mut self, class: &HirClass, module: &HirModule) {
        if !self.structural_interop_enabled || !structural_record_supported(class, module) {
            return;
        }
        let target = Self::class_impl_target(class);
        let type_params = structural_impl_type_params(class);
        let nominal_identity = nominal_identity(class, self.current_module_name.as_deref());
        let type_impl = structural_type_impl(
            class,
            module,
            &target,
            type_params.clone(),
            self.current_module_name.as_deref(),
        );
        let construct_impl =
            structural_construct_impl(class, &target, type_params.clone(), &nominal_identity, self);
        self.body_items.push(type_impl);
        self.body_items.push(construct_impl);
        self.body_items.push(structural_project_impl(
            class,
            &target,
            type_params,
            &nominal_identity,
        ));
    }

    pub(crate) fn emit_structural_enum_impls(&mut self, class: &HirClass) {
        if !self.structural_interop_enabled || !class.is_enum() {
            return;
        }
        let target = Self::class_impl_target(class);
        let nominal_identity = nominal_identity(class, self.current_module_name.as_deref());
        let shape = crate::structural_identity_codegen::enum_identity_expression(
            class,
            self.current_module_name.as_deref(),
        );
        self.body_items.extend(structural_enum_impls(
            class,
            &target,
            &nominal_identity,
            &shape,
        ));
    }
}

pub(crate) fn structural_record_supported(class: &HirClass, module: &HirModule) -> bool {
    let mut visiting = BTreeSet::from([class.name.clone()]);
    class.parent_class.is_none()
        && !class.is_error_type
        && class.newtype_inner.is_none()
        && crate::structural_identity_codegen::class_identity_inputs_supported(class)
        && !class.is_enum()
        && class.python_opaque_declaration().is_none()
        && !class
            .rust_interop
            .iter()
            .any(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)
        && class
            .fields
            .iter()
            .all(|(_, ty)| structural_record_field_supported(ty, module, &mut visiting))
}

fn structural_record_field_supported(
    ty: &Type,
    module: &HirModule,
    visiting: &mut BTreeSet<String>,
) -> bool {
    matches!(ty.resolve_alias(), Type::Bytes) || structural_type_supported(ty, module, visiting)
}

fn structural_type_supported(
    ty: &Type,
    module: &HirModule,
    visiting: &mut BTreeSet<String>,
) -> bool {
    match ty.resolve_alias() {
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::None | Type::TypeVar(_) => true,
        Type::Enum { .. } => true,
        Type::Bytes => false,
        Type::FixedInt(value) => !matches!(
            value,
            sifr_type_system::FixedIntType::ISize | sifr_type_system::FixedIntType::USize
        ),
        Type::List(value) | Type::Set(value) => structural_type_supported(value, module, visiting),
        Type::Dict(key, value) => {
            structural_type_supported(key, module, visiting)
                && structural_type_supported(value, module, visiting)
        }
        Type::Tuple(values) => {
            values.len() <= 4
                && values
                    .iter()
                    .all(|value| structural_type_supported(value, module, visiting))
        }
        Type::Union(values) => values
            .iter()
            .all(|value| structural_type_supported(value, module, visiting)),
        Type::Class { name, .. } => {
            if visiting.contains(name) {
                return true;
            }
            let Some(candidate) = module
                .classes
                .iter()
                .find(|candidate| candidate.name == *name)
            else {
                return false;
            };
            if candidate.is_enum() {
                return crate::structural_identity_codegen::class_identity_inputs_supported(
                    candidate,
                );
            }
            if candidate.parent_class.is_some()
                || candidate.is_error_type
                || candidate.newtype_inner.is_some()
                || !crate::structural_identity_codegen::class_identity_inputs_supported(candidate)
                || candidate.python_opaque_declaration().is_some()
                || candidate
                    .rust_interop
                    .iter()
                    .any(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)
            {
                return false;
            }
            visiting.insert(name.clone());
            let supported = candidate
                .fields
                .iter()
                .all(|(_, field)| structural_record_field_supported(field, module, visiting));
            visiting.remove(name);
            supported
        }
        Type::Newtype { .. } => false,
        _ => false,
    }
}

pub(crate) fn structural_union_names(
    module: &HirModule,
    unions: &std::collections::HashMap<String, Vec<Type>>,
) -> HashSet<String> {
    let mut names = HashSet::new();
    for class in &module.classes {
        if structural_record_supported(class, module) {
            for (_, field) in &class.fields {
                collect_structural_union_names(field, &mut names);
            }
        }
    }
    for (name, members) in unions {
        if members
            .iter()
            .all(|member| structural_type_supported(member, module, &mut BTreeSet::new()))
        {
            names.insert(name.clone());
        }
    }
    names
}

fn collect_structural_union_names(ty: &Type, names: &mut HashSet<String>) {
    match ty.resolve_alias() {
        Type::Union(values) if !is_optional_union(values) => {
            names.insert(ty.union_enum_name());
            for value in values {
                collect_structural_union_names(value, names);
            }
        }
        Type::Union(values) | Type::Tuple(values) => {
            for value in values {
                collect_structural_union_names(value, names);
            }
        }
        Type::List(value) | Type::Set(value) => collect_structural_union_names(value, names),
        Type::Dict(key, value) => {
            collect_structural_union_names(key, names);
            collect_structural_union_names(value, names);
        }
        Type::Class {
            type_args, fields, ..
        } => {
            for value in type_args {
                collect_structural_union_names(value, names);
            }
            for (_, value) in fields {
                collect_structural_union_names(value, names);
            }
        }
        _ => {}
    }
}

fn is_optional_union(values: &[Type]) -> bool {
    values.len() == 2
        && values
            .iter()
            .any(|value| matches!(value.resolve_alias(), Type::None))
}

fn structural_impl_type_params(class: &HirClass) -> Vec<RustTypeParam> {
    class
        .type_params
        .iter()
        .map(|name| {
            let mut bounds = RustEmitter::class_base_type_param_bounds(class, name);
            bounds.push(format!(
                "{STRUCTURAL}::StructuralConstruct + {STRUCTURAL}::StructuralProject"
            ));
            RustTypeParam {
                name: name.clone(),
                bounds,
            }
        })
        .collect()
}

fn nominal_identity(class: &HirClass, module_name: Option<&str>) -> String {
    let identity = class.identity.clone().unwrap_or_else(|| {
        module_name.map_or_else(
            || class.name.clone(),
            |module| format!("{module}.{}", class.name),
        )
    });
    format!("{identity:?}")
}

fn structural_type_impl(
    class: &HirClass,
    module: &HirModule,
    target: &str,
    type_params: Vec<RustTypeParam>,
    module_name: Option<&str>,
) -> RustItem {
    let identity =
        crate::structural_identity_codegen::class_identity_expression(class, module, module_name);
    let nominal_identity = nominal_identity(class, module_name);
    RustItem::Impl {
        target: target.to_string(),
        type_params,
        trait_: Some(format!("{STRUCTURAL}::StructuralType")),
        items: vec![
            RustItem::Fn {
                name: "shape_identity".to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: Vec::new(),
                ret: Some(RustType::Named(format!("{STRUCTURAL}::ShapeIdentity"))),
                body: vec![RustStmt::Verbatim(identity)],
                is_async: false,
            },
            RustItem::Fn {
                name: "nominal_identity".to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: Vec::new(),
                ret: Some(RustType::Named("Option<&'static str>".to_string())),
                body: vec![RustStmt::Verbatim(format!("Some({nominal_identity})"))],
                is_async: false,
            },
        ],
    }
}

fn structural_construct_impl(
    class: &HirClass,
    target: &str,
    type_params: Vec<RustTypeParam>,
    nominal_identity: &str,
    emitter: &mut RustEmitter,
) -> RustItem {
    let edge_checks = class
        .fields
        .iter()
        .enumerate()
        .map(|(index, (name, _))| {
            format!(
                "if description.edges()[{index}].kind() != {STRUCTURAL}::StructuralEdgeKind::RecordField(\"{name}\") {{ return Err({STRUCTURAL}::StructuralContractError::MemberMismatch); }}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let field_values = class
        .fields
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| {
            let rust_name = crate::Renderer::render_identifier(name);
            if matches!(ty.resolve_alias(), Type::Bytes) {
                return format!(
                    "let {rust_name} = {STRUCTURAL}::construct_bytes_at(source, child_nodes[{index}], token)?;"
                );
            }
            let rust_type = emitter.class_struct_field_rust_type(class, name, ty);
            let rust_type = crate::Renderer::render_type_string(&rust_type);
            format!(
                "let {rust_name} = <{rust_type} as {STRUCTURAL}::StructuralConstruct>::structural_construct_at(source, child_nodes[{index}], token)?;"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut initializers = class
        .fields
        .iter()
        .map(|(name, _)| crate::Renderer::render_identifier(name))
        .collect::<Vec<_>>();
    if RustEmitter::class_needs_phantom_marker(class) {
        initializers.push("__sifr_type_marker: std::marker::PhantomData".to_string());
    }
    let body = format!(
        "let description = source.node(node)?;\nif description.kind() != {STRUCTURAL}::StructuralKind::Record {{ return Err({STRUCTURAL}::StructuralContractError::KindMismatch); }}\nif description.nominal_identity() != Some({}) {{ return Err({STRUCTURAL}::StructuralContractError::MemberMismatch); }}\nif description.edges().len() != {} {{ return Err({STRUCTURAL}::StructuralContractError::ArityMismatch); }}\n{edge_checks}\nlet child_nodes = description.edges().iter().map({STRUCTURAL}::StructuralNodeEdge::node).collect::<Vec<_>>();\n{field_values}\nOk(Self {{ {} }})",
        nominal_identity,
        class.fields.len(),
        initializers.join(", ")
    );
    RustItem::Impl {
        target: target.to_string(),
        type_params,
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
            body: vec![RustStmt::Verbatim(body)],
            is_async: false,
        }],
    }
}

fn structural_project_impl(
    class: &HirClass,
    target: &str,
    type_params: Vec<RustTypeParam>,
    nominal_identity: &str,
) -> RustItem {
    let visits = class
        .fields
        .iter()
        .map(|(name, ty)| {
            let rust_name = crate::Renderer::render_identifier(name);
            if matches!(ty.resolve_alias(), Type::Bytes) {
                return format!(
                    "visitor.edge({STRUCTURAL}::StructuralEdge::new({STRUCTURAL}::StructuralEdgeKind::RecordField(\"{name}\")))?;\n{STRUCTURAL}::project_bytes(&self.{rust_name}, visitor)?;"
                );
            }
            format!(
                "visitor.edge({STRUCTURAL}::StructuralEdge::new({STRUCTURAL}::StructuralEdgeKind::RecordField(\"{name}\")))?;\n{STRUCTURAL}::StructuralProject::structural_project(&self.{rust_name}, visitor)?;"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "let control = visitor.enter({STRUCTURAL}::StructuralEnter::new({STRUCTURAL}::StructuralKind::Record, Some({}), {}))?;\nif control == {STRUCTURAL}::VisitControl::Continue {{\n{visits}\n}}\nvisitor.exit({STRUCTURAL}::StructuralKind::Record)",
        nominal_identity,
        class.fields.len()
    );
    RustItem::Impl {
        target: target.to_string(),
        type_params,
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
            body: vec![RustStmt::Verbatim(body)],
            is_async: false,
        }],
    }
}

fn structural_enum_impls(
    class: &HirClass,
    target: &str,
    nominal_identity: &str,
    shape: &str,
) -> Vec<RustItem> {
    let mut next_value = 1_i64;
    let mut construct_arms = Vec::with_capacity(class.enum_variants.len());
    let mut project_arms = Vec::with_capacity(class.enum_variants.len());
    for (index, (name, declared)) in class.enum_variants.iter().enumerate() {
        let value = declared.unwrap_or(next_value);
        next_value = value.saturating_add(1);
        construct_arms.push(format!(
            "{STRUCTURAL}::StructuralEdgeKind::ActiveMember {{ name: \"{name}\", index: {index} }} => match <i64 as {STRUCTURAL}::StructuralConstruct>::structural_construct_at(source, child, token) {{ Ok({value}) => Ok(Self::{name}), Ok(_) => Err({STRUCTURAL}::StructuralContractError::MemberMismatch), Err(error) => Err(error), }},"
        ));
        project_arms.push(format!(
            "Self::{name} => {{ visitor.edge({STRUCTURAL}::StructuralEdge::new({STRUCTURAL}::StructuralEdgeKind::ActiveMember {{ name: \"{name}\", index: {index} }}))?; visitor.scalar({STRUCTURAL}::StructuralScalarRef::SignedInteger {{ value: i128::from({value}_i64), width: 64 }})?; }}"
        ));
    }
    let construct_body = format!(
        "let description = source.node(node)?;\nif description.kind() != {STRUCTURAL}::StructuralKind::Enum {{ return Err({STRUCTURAL}::StructuralContractError::KindMismatch); }}\nif description.nominal_identity() != Some({nominal_identity}) {{ return Err({STRUCTURAL}::StructuralContractError::MemberMismatch); }}\nlet [edge] = description.edges() else {{ return Err({STRUCTURAL}::StructuralContractError::ArityMismatch); }};\nlet edge_kind = edge.kind();\nlet child = edge.node();\nmatch edge_kind {{\n{}\n_ => Err({STRUCTURAL}::StructuralContractError::MemberMismatch),\n}}",
        construct_arms.join("\n")
    );
    let project_body = format!(
        "let control = visitor.enter({STRUCTURAL}::StructuralEnter::new({STRUCTURAL}::StructuralKind::Enum, Some({nominal_identity}), 1))?;\nif control == {STRUCTURAL}::VisitControl::Continue {{\nmatch self {{\n{}\n}}\n}}\nvisitor.exit({STRUCTURAL}::StructuralKind::Enum)",
        project_arms.join("\n")
    );
    vec![
        RustItem::Impl {
            target: target.to_string(),
            type_params: Vec::new(),
            trait_: Some(format!("{STRUCTURAL}::StructuralType")),
            items: vec![
                RustItem::Fn {
                    name: "shape_identity".to_string(),
                    visibility: Visibility::Private,
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret: Some(RustType::Named(format!("{STRUCTURAL}::ShapeIdentity"))),
                    body: vec![RustStmt::Verbatim(shape.to_string())],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "nominal_identity".to_string(),
                    visibility: Visibility::Private,
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret: Some(RustType::Named("Option<&'static str>".to_string())),
                    body: vec![RustStmt::Verbatim(format!("Some({nominal_identity})"))],
                    is_async: false,
                },
            ],
        },
        RustItem::Impl {
            target: target.to_string(),
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
            target: target.to_string(),
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
