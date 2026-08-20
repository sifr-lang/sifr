use crate::{RustEmitter, RustItem, RustParam, RustStmt, RustType, RustTypeParam, Visibility};
use sifr_ir::{HirClass, HirModule, RustInteropDecoratorKind};
use sifr_type_system::Type;
use std::collections::{BTreeSet, HashSet};

#[cfg(test)]
mod generic_representation_tests;

use crate::structural_record_fields::structural_record_fields;

mod stdlib_implementations;

const STRUCTURAL: &str = "::sifr_runtime::interop::structural";

impl RustEmitter {
    pub(crate) fn emit_imported_stdlib_structural_impls(
        &mut self,
        module: &HirModule,
        stdlib_code: &crate::StdlibCode,
    ) {
        if !self.structural_interop_enabled {
            return;
        }
        let mut emitted_targets = HashSet::new();
        for import in &module.imports {
            let Some(templates) = stdlib_code.module_class_templates.get(&import.module) else {
                continue;
            };
            let support_module = HirModule {
                functions: Vec::new(),
                classes: templates.values().cloned().collect(),
                imports: Vec::new(),
                constants: Vec::new(),
                generic_functions: std::collections::HashMap::new(),
                type_param_bounds: std::collections::HashMap::new(),
            };
            for name in &import.names {
                let Some(class) = templates.get(name) else {
                    continue;
                };
                let target = stdlib_implementations::target(class);
                if !emitted_targets.insert(target.clone()) {
                    continue;
                }
                self.emit_structural_record_impls_for_target(
                    class,
                    &support_module,
                    &target,
                    StructuralRecordOrigin::Stdlib,
                );
                self.emit_structural_enum_impls_for_target(class, &target);
            }
        }
    }

    pub(crate) fn emit_structural_record_impls(&mut self, class: &HirClass, module: &HirModule) {
        let target = Self::class_impl_target(class);
        self.emit_structural_record_impls_for_target(
            class,
            module,
            &target,
            StructuralRecordOrigin::Project,
        );
    }

    fn emit_structural_record_impls_for_target(
        &mut self,
        class: &HirClass,
        module: &HirModule,
        target: &str,
        origin: StructuralRecordOrigin,
    ) {
        let supported = self
            .project_structural_record_identities
            .as_ref()
            .map_or_else(
                || structural_record_supported(class, module),
                |identities| {
                    let identity =
                        structural_record_identity(class, self.current_module_name.as_deref());
                    identities.contains(&identity)
                        || (origin == StructuralRecordOrigin::Stdlib
                            && structural_record_supported(class, module))
                },
            );
        if !self.structural_interop_enabled || !supported {
            return;
        }
        let type_params = structural_impl_type_params(class);
        let structural_module_name = self.structural_identity_module_name.as_deref();
        let nominal_identity = nominal_identity(class, structural_module_name);
        let identity_key = structural_record_identity(class, self.current_module_name.as_deref());
        let shape = self
            .project_structural_identity_expressions
            .as_ref()
            .and_then(|expressions| expressions.get(&identity_key))
            .cloned()
            .unwrap_or_else(|| {
                crate::structural_identity_codegen::class_identity_expression(
                    class,
                    module,
                    structural_module_name,
                )
            });
        let type_impl = structural_type_impl(
            class,
            target,
            type_params.clone(),
            structural_module_name,
            shape,
        );
        let construct_impl =
            structural_construct_impl(class, target, type_params.clone(), &nominal_identity, self);
        self.body_items.push(type_impl);
        self.body_items.push(construct_impl);
        self.body_items.push(structural_project_impl(
            class,
            target,
            type_params,
            &nominal_identity,
        ));
    }

    pub(crate) fn emit_structural_enum_impls(&mut self, class: &HirClass) {
        let target = Self::class_impl_target(class);
        self.emit_structural_enum_impls_for_target(class, &target);
    }

    fn emit_structural_enum_impls_for_target(&mut self, class: &HirClass, target: &str) {
        if !self.structural_interop_enabled || !class.is_enum() || !structural_enum_supported(class)
        {
            return;
        }
        let structural_module_name = self.structural_identity_module_name.as_deref();
        let nominal_identity = nominal_identity(class, structural_module_name);
        let identity_key = structural_record_identity(class, self.current_module_name.as_deref());
        let shape = self
            .project_structural_identity_expressions
            .as_ref()
            .and_then(|expressions| expressions.get(&identity_key))
            .cloned()
            .unwrap_or_else(|| {
                crate::structural_identity_codegen::enum_identity_expression(
                    class,
                    structural_module_name,
                )
            });
        self.body_items.extend(structural_enum_impls(
            class,
            target,
            &nominal_identity,
            &shape,
        ));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StructuralRecordOrigin {
    Project,
    Stdlib,
}

pub(crate) fn structural_record_supported(class: &HirClass, module: &HirModule) -> bool {
    let modules = [("", module)];
    structural_record_supported_in(class, &modules)
}

pub(crate) fn structural_record_supported_for_project(
    class: &HirClass,
    modules: &[(&str, &HirModule)],
) -> bool {
    structural_record_supported_in(class, modules)
}

pub(crate) fn structural_record_identities_for_project(
    modules: &[(&str, &HirModule)],
) -> HashSet<String> {
    modules
        .iter()
        .flat_map(|(module_name, module)| {
            module
                .classes
                .iter()
                .filter(|class| structural_record_supported_in(class, modules))
                .map(|class| structural_record_identity(class, Some(module_name)))
        })
        .collect()
}

fn structural_record_identity(class: &HirClass, module_name: Option<&str>) -> String {
    class.identity.clone().unwrap_or_else(|| {
        module_name
            .filter(|module_name| !module_name.is_empty())
            .map_or_else(
                || class.name.clone(),
                |module_name| format!("{module_name}.{}", class.name),
            )
    })
}

fn structural_record_supported_in(class: &HirClass, modules: &[(&str, &HirModule)]) -> bool {
    let mut visiting =
        BTreeSet::from([class.identity.clone().unwrap_or_else(|| class.name.clone())]);
    structural_record_supported_with_visiting(class, modules, &mut visiting)
}

fn structural_record_supported_with_visiting(
    class: &HirClass,
    modules: &[(&str, &HirModule)],
    visiting: &mut BTreeSet<String>,
) -> bool {
    structural_parent_supported(class, modules, visiting)
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
            .all(|(_, ty)| structural_record_field_supported(ty, modules, visiting))
}

fn structural_parent_supported(
    class: &HirClass,
    modules: &[(&str, &HirModule)],
    visiting: &mut BTreeSet<String>,
) -> bool {
    let Some(parent_type) = class.parent_type.as_ref() else {
        return class.parent_class.is_none();
    };
    let Type::Class {
        identity,
        type_args,
        name,
        fields,
        ..
    } = parent_type.resolve_alias()
    else {
        return false;
    };
    let Some(parent) = structural_class_candidate(identity.as_deref(), name, modules) else {
        return false;
    };
    generic_arguments_preserve_union_structure(parent, type_args, modules)
        && parent.parent_class.is_none()
        && !fields
            .iter()
            .any(|(_, ty)| crate::helpers::type_references_class(ty, name))
        && !parent.methods.iter().any(|method| method.name == "new")
        && !parent.is_error_type
        && parent.newtype_inner.is_none()
        && crate::structural_identity_codegen::class_identity_inputs_supported(parent)
        && parent.python_opaque_declaration().is_none()
        && !parent
            .rust_interop
            .iter()
            .any(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)
        && fields
            .iter()
            .all(|(_, ty)| structural_record_field_supported(ty, modules, visiting))
}

pub(crate) fn structural_mapped_opaque_supported(class: &HirClass) -> bool {
    class.parent_class.is_none()
        && class.type_params.is_empty()
        && class.fields.is_empty()
        && !class.is_error_type
        && sifr_ir::rust_opaque_type_path(&class.rust_interop).is_some()
        && sifr_ir::rust_opaque_structural_mapping(&class.rust_interop).is_some()
}

fn structural_record_field_supported(
    ty: &Type,
    modules: &[(&str, &HirModule)],
    visiting: &mut BTreeSet<String>,
) -> bool {
    matches!(ty.resolve_alias(), Type::Bytes) || structural_type_supported(ty, modules, visiting)
}

fn structural_type_supported(
    ty: &Type,
    modules: &[(&str, &HirModule)],
    visiting: &mut BTreeSet<String>,
) -> bool {
    match ty.resolve_alias() {
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::None | Type::TypeVar(_) => true,
        Type::Enum {
            identity,
            name,
            variants,
        } => structural_class_candidate(identity.as_deref(), name, modules).map_or_else(
            || sifr_ir::structural_identity_enum_variants_supported(variants),
            structural_enum_supported,
        ),
        Type::Bytes => false,
        Type::FixedInt(value) => !matches!(
            value,
            sifr_type_system::FixedIntType::ISize | sifr_type_system::FixedIntType::USize
        ),
        Type::List(value) | Type::Set(value) => structural_type_supported(value, modules, visiting),
        Type::Dict(key, value) => {
            structural_type_supported(key, modules, visiting)
                && structural_type_supported(value, modules, visiting)
        }
        Type::Tuple(values) => {
            values.len() <= 4
                && values
                    .iter()
                    .all(|value| structural_type_supported(value, modules, visiting))
        }
        Type::Union(values) => values
            .iter()
            .all(|value| structural_type_supported(value, modules, visiting)),
        Type::Class {
            identity,
            type_args,
            name,
            ..
        } => {
            let key = identity.as_deref().unwrap_or(name);
            if visiting.contains(key) {
                return true;
            }
            let Some(candidate) = structural_class_candidate(identity.as_deref(), name, modules)
            else {
                return false;
            };
            if !generic_arguments_preserve_union_structure(candidate, type_args, modules) {
                return false;
            }
            if structural_mapped_opaque_supported(candidate) {
                return true;
            }
            if candidate.is_enum() {
                return structural_enum_supported(candidate);
            }
            visiting.insert(key.to_string());
            let supported = structural_record_supported_with_visiting(candidate, modules, visiting);
            visiting.remove(key);
            supported
        }
        Type::Newtype { .. } => false,
        _ => false,
    }
}

fn generic_arguments_preserve_union_structure(
    class: &HirClass,
    type_args: &[Type],
    modules: &[(&str, &HirModule)],
) -> bool {
    let bindings = class
        .type_params
        .iter()
        .cloned()
        .zip(type_args.iter().cloned())
        .collect::<std::collections::HashMap<_, _>>();
    let class_scope = |identity: Option<&str>, name: &str| {
        structural_class_candidate(identity, name, modules).map(generic_union_scope)
    };
    class.fields.iter().all(|(_, ty)| {
        sifr_type_system::substitution_preserves_union_structure_with_class_scopes(
            ty,
            &bindings,
            &class_scope,
        )
    }) && class.methods.iter().all(|method| {
        method.params.iter().all(|param| {
            sifr_type_system::substitution_preserves_union_structure_with_class_scopes(
                &param.ty,
                &bindings,
                &class_scope,
            )
        }) && sifr_type_system::substitution_preserves_union_structure_with_class_scopes(
            &method.return_type,
            &bindings,
            &class_scope,
        )
    })
}

fn generic_union_scope(class: &HirClass) -> sifr_type_system::UnionStructureClassScope {
    let mut member_types = class
        .fields
        .iter()
        .map(|(_, ty)| ty.clone())
        .collect::<Vec<_>>();
    for method in &class.methods {
        member_types.extend(method.params.iter().map(|param| param.ty.clone()));
        member_types.push(method.return_type.clone());
    }
    sifr_type_system::UnionStructureClassScope {
        type_params: class.type_params.clone(),
        member_types,
    }
}

fn structural_class_candidate<'a>(
    identity: Option<&str>,
    name: &str,
    modules: &'a [(&'a str, &'a HirModule)],
) -> Option<&'a HirClass> {
    if let Some(identity) = identity {
        if let Some(candidate) = modules.iter().find_map(|(module_name, module)| {
            module.classes.iter().find(|class| {
                class.identity.as_deref() == Some(identity)
                    || format!("{module_name}.{}", class.name) == identity
            })
        }) {
            return Some(candidate);
        }
        if modules.len() != 1 || !modules[0].0.is_empty() {
            return None;
        }
    }
    let mut candidates = modules
        .iter()
        .flat_map(|(_, module)| &module.classes)
        .filter(|class| class.name == name);
    let candidate = candidates.next()?;
    candidates.next().is_none().then_some(candidate)
}

pub(crate) fn structural_enum_supported(class: &HirClass) -> bool {
    sifr_ir::structural_identity_enum_variants_supported(&class.enum_variants)
        && crate::structural_identity_codegen::class_identity_inputs_supported(class)
        && !class.is_error_type
        && class.python_opaque_declaration().is_none()
        && !class
            .rust_interop
            .iter()
            .any(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)
}

pub(crate) fn structural_union_names(
    module: &HirModule,
    unions: &std::collections::HashMap<String, Vec<Type>>,
) -> HashSet<String> {
    structural_union_names_in(unions, &[("", module)])
}

pub(crate) fn structural_union_names_for_project(
    unions: &std::collections::HashMap<String, Vec<Type>>,
    modules: &[(&str, &HirModule)],
) -> HashSet<String> {
    structural_union_names_in(unions, modules)
}

fn structural_union_names_in(
    unions: &std::collections::HashMap<String, Vec<Type>>,
    modules: &[(&str, &HirModule)],
) -> HashSet<String> {
    let mut names = HashSet::new();
    for (name, members) in unions {
        if members
            .iter()
            .all(|member| structural_type_supported(member, modules, &mut BTreeSet::new()))
        {
            names.insert(name.clone());
        }
    }
    names
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
    target: &str,
    type_params: Vec<RustTypeParam>,
    module_name: Option<&str>,
    identity: String,
) -> RustItem {
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
    let fields = structural_record_fields(class);
    let edge_cases = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let name = field.name;
            format!("{STRUCTURAL}::StructuralEdgeKind::RecordField(\"{name}\") => {index},")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut field_values = Vec::with_capacity(fields.len());
    for (index, field) in fields.iter().enumerate() {
        let name = field.name;
        let ty = field.ty;
        let rust_name = crate::Renderer::render_identifier(name);
        let present = if matches!(ty.resolve_alias(), Type::Bytes) {
            format!(
                "{STRUCTURAL}::construct_bytes_at(__sifr_source, __sifr_child, __sifr_construct_token)?"
            )
        } else {
            let rust_type = emitter.class_struct_field_rust_type(class, name, ty);
            let rust_type = crate::Renderer::render_type_string(&rust_type);
            format!(
                "<{rust_type} as {STRUCTURAL}::StructuralConstruct>::structural_construct_at(__sifr_source, __sifr_child, __sifr_construct_token)?"
            )
        };
        let missing = class
            .field_defaults
            .iter()
            .find(|(field, _)| *field == index)
            .map_or_else(
                || format!("return Err({STRUCTURAL}::StructuralContractError::ArityMismatch)"),
                |(_, value)| {
                    let value = emitter
                        .lower_class_expr_strict(value, "structural field default construction");
                    crate::render::render_expr(&value)
                },
            );
        field_values.push(format!(
            "let {rust_name} = match __sifr_child_nodes[{index}] {{ Some(__sifr_child) => {present}, None => {missing}, }};"
        ));
    }
    let field_values = field_values.join("\n");
    let required_prechecks = fields
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            !class
                .field_defaults
                .iter()
                .any(|(field_index, _)| field_index == index)
        })
        .map(|(index, _)| {
            format!(
                "if __sifr_child_nodes[{index}].is_none() {{ return Err({STRUCTURAL}::StructuralContractError::ArityMismatch); }}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let inherited_names = fields
        .iter()
        .filter(|field| field.inherited)
        .map(|field| crate::Renderer::render_identifier(field.name))
        .collect::<Vec<_>>();
    let mut initializers = Vec::new();
    if let (Some(parent_name), Some(parent_type)) = (&class.parent_class, &class.parent_type) {
        let parent_rust_type =
            crate::Renderer::render_type_string(&crate::sifr_type_to_rust_type(parent_type));
        initializers.push(format!(
            "{}: <{parent_rust_type}>::new({})",
            parent_name.to_lowercase(),
            inherited_names.join(", ")
        ));
    }
    initializers.extend(
        fields
            .iter()
            .filter(|field| !field.inherited)
            .map(|field| crate::Renderer::render_identifier(field.name)),
    );
    if RustEmitter::class_needs_phantom_marker(class) {
        initializers.push("__sifr_type_marker: std::marker::PhantomData".to_string());
    }
    let collect_children = if fields.is_empty() {
        format!(
            "if !__sifr_description.edges().is_empty() {{ return Err({STRUCTURAL}::StructuralContractError::MemberMismatch); }}"
        )
    } else {
        format!(
            "let mut __sifr_child_nodes: [Option<{STRUCTURAL}::NodeId>; {}] = [None; {}];\nfor __sifr_edge in __sifr_description.edges() {{\nlet __sifr_field_index: usize = match __sifr_edge.kind() {{\n{edge_cases}\n_ => return Err({STRUCTURAL}::StructuralContractError::MemberMismatch),\n}};\nif __sifr_child_nodes[__sifr_field_index].replace(__sifr_edge.node()).is_some() {{ return Err({STRUCTURAL}::StructuralContractError::MemberMismatch); }}\n}}",
            fields.len(),
            fields.len()
        )
    };
    let body = format!(
        "let __sifr_description = __sifr_source.node(__sifr_node)?;\nif __sifr_description.kind() != {STRUCTURAL}::StructuralKind::Record {{ return Err({STRUCTURAL}::StructuralContractError::KindMismatch); }}\nif __sifr_description.nominal_identity() != Some({}) {{ return Err({STRUCTURAL}::StructuralContractError::MemberMismatch); }}\n{collect_children}\n{required_prechecks}\n{field_values}\nOk(Self {{ {} }})",
        nominal_identity,
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
                    name: "__sifr_source".to_string(),
                    ty: RustType::Ref {
                        mutable: true,
                        inner: Box::new(RustType::Named("S".to_string())),
                    },
                },
                RustParam::Named {
                    name: "__sifr_node".to_string(),
                    ty: RustType::Named(format!("{STRUCTURAL}::NodeId")),
                },
                RustParam::Named {
                    name: "__sifr_construct_token".to_string(),
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
    let fields = structural_record_fields(class);
    let visits = fields
        .iter()
        .map(|field| {
            let name = field.name;
            let ty = field.ty;
            let rust_name = crate::Renderer::render_identifier(name);
            let access = if field.inherited {
                format!(
                    "self.{}.{rust_name}",
                    class
                        .parent_class
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                )
            } else {
                format!("self.{rust_name}")
            };
            if matches!(ty.resolve_alias(), Type::Bytes) {
                return format!(
                    "visitor.edge({STRUCTURAL}::StructuralEdge::new({STRUCTURAL}::StructuralEdgeKind::RecordField(\"{name}\")))?;\n{STRUCTURAL}::project_bytes(&{access}, visitor)?;"
                );
            }
            format!(
                "visitor.edge({STRUCTURAL}::StructuralEdge::new({STRUCTURAL}::StructuralEdgeKind::RecordField(\"{name}\")))?;\n{STRUCTURAL}::StructuralProject::structural_project(&{access}, visitor)?;"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "let control = visitor.enter({STRUCTURAL}::StructuralEnter::new({STRUCTURAL}::StructuralKind::Record, Some({}), {}))?;\nif control == {STRUCTURAL}::VisitControl::Continue {{\n{visits}\n}}\nvisitor.exit({STRUCTURAL}::StructuralKind::Record)",
        nominal_identity,
        fields.len()
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
    let mut construct_arms = Vec::with_capacity(class.enum_variants.len());
    let mut project_arms = Vec::with_capacity(class.enum_variants.len());
    for (index, (name, value)) in crate::type_emitters::resolved_enum_variants(class)
        .into_iter()
        .enumerate()
    {
        construct_arms.push(format!(
            "{STRUCTURAL}::StructuralEdgeKind::ActiveMember {{ name: \"{name}\", index: {index} }} => match <i64 as {STRUCTURAL}::StructuralConstruct>::structural_construct_at(__sifr_source, child, __sifr_construct_token) {{ Ok({value}) => Ok(Self::{name}), Ok(_) => Err({STRUCTURAL}::StructuralContractError::MemberMismatch), Err(error) => Err(error), }},"
        ));
        project_arms.push(format!(
            "Self::{name} => {{ visitor.edge({STRUCTURAL}::StructuralEdge::new({STRUCTURAL}::StructuralEdgeKind::ActiveMember {{ name: \"{name}\", index: {index} }}))?; visitor.scalar({STRUCTURAL}::StructuralScalarRef::SignedInteger {{ value: i128::from({value}_i64), width: 64 }})?; }}"
        ));
    }
    let construct_body = format!(
        "let description = __sifr_source.node(__sifr_node)?;\nif description.kind() != {STRUCTURAL}::StructuralKind::Enum {{ return Err({STRUCTURAL}::StructuralContractError::KindMismatch); }}\nif description.nominal_identity() != Some({nominal_identity}) {{ return Err({STRUCTURAL}::StructuralContractError::MemberMismatch); }}\nlet [edge] = description.edges() else {{ return Err({STRUCTURAL}::StructuralContractError::ArityMismatch); }};\nlet edge_kind = edge.kind();\nlet child = edge.node();\nmatch edge_kind {{\n{}\n_ => Err({STRUCTURAL}::StructuralContractError::MemberMismatch),\n}}",
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
                        name: "__sifr_source".to_string(),
                        ty: RustType::Ref {
                            mutable: true,
                            inner: Box::new(RustType::Named("S".to_string())),
                        },
                    },
                    RustParam::Named {
                        name: "__sifr_node".to_string(),
                        ty: RustType::Named(format!("{STRUCTURAL}::NodeId")),
                    },
                    RustParam::Named {
                        name: "__sifr_construct_token".to_string(),
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
