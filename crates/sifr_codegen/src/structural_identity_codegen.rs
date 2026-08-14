use sifr_ir::{canonical_structural_identity_value, HirClass, HirModule};
use sifr_structural_identity::{self as identity, NominalField, ShapeIdentity, ALGORITHM_VERSION};
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

const STRUCTURAL: &str = "::sifr_runtime::interop::structural";

#[derive(Debug)]
enum CompiledIdentity {
    Static(ShapeIdentity),
    Dynamic(String),
}

struct IdentityContext<'a> {
    modules: &'a [(&'a str, &'a HirModule)],
}

impl<'a> IdentityContext<'a> {
    fn wire_module_name(module_name: &'a str) -> Option<&'a str> {
        (!module_name.is_empty()).then_some(module_name)
    }

    fn class_candidate(
        &self,
        identity: Option<&str>,
        name: &str,
        scope_module_name: &str,
    ) -> Option<(&'a str, &'a HirModule, &'a HirClass)> {
        if let Some(identity) = identity {
            if let Some(candidate) = self.modules.iter().find_map(|(module_name, module)| {
                module
                    .classes
                    .iter()
                    .find(|class| {
                        class.identity.as_deref() == Some(identity)
                            || format!("{module_name}.{}", class.name) == identity
                    })
                    .map(|class| (*module_name, *module, class))
            }) {
                return Some(candidate);
            }
            if self.modules.len() != 1 || !self.modules[0].0.is_empty() {
                return None;
            }
        }
        if let Some((module_name, module)) = self
            .modules
            .iter()
            .find(|(module_name, _)| *module_name == scope_module_name)
        {
            if let Some(class) = module.classes.iter().find(|class| class.name == name) {
                return Some((*module_name, *module, class));
            }
        }
        let mut candidates = self.modules.iter().filter_map(|(module_name, module)| {
            module
                .classes
                .iter()
                .find(|class| class.name == name)
                .map(|class| (*module_name, *module, class))
        });
        let candidate = candidates.next()?;
        candidates.next().is_none().then_some(candidate)
    }
}

impl CompiledIdentity {
    fn expression(&self) -> String {
        match self {
            Self::Static(value) => static_expression(*value),
            Self::Dynamic(value) => value.clone(),
        }
    }

    fn static_value(&self) -> Option<ShapeIdentity> {
        match self {
            Self::Static(value) => Some(*value),
            Self::Dynamic(_) => None,
        }
    }
}

pub(crate) fn class_identity_expression(
    class: &HirClass,
    module: &HirModule,
    module_name: Option<&str>,
) -> String {
    let module_key = module_name.unwrap_or("");
    let modules = [(module_key, module)];
    let context = IdentityContext { modules: &modules };
    if class.is_enum() {
        compile_enum(class, module_name).expression()
    } else {
        compile_class(class, module_key, &context).expression()
    }
}

pub(crate) fn class_identity_expressions_for_project(
    modules: &[(&str, &HirModule)],
    structural_record_identities: &HashSet<String>,
) -> HashMap<String, String> {
    let context = IdentityContext { modules };
    modules
        .iter()
        .flat_map(|(module_name, module)| {
            let context = &context;
            module.classes.iter().filter_map(move |class| {
                let key = project_class_identity(class, module_name);
                let supported = if class.is_enum() {
                    crate::structural_impl_codegen::structural_enum_supported(class)
                } else {
                    structural_record_identities.contains(&key)
                };
                supported.then(|| {
                    let compiled = if class.is_enum() {
                        compile_enum(class, IdentityContext::wire_module_name(module_name))
                    } else {
                        compile_class(class, module_name, context)
                    };
                    (key, compiled.expression())
                })
            })
        })
        .collect()
}

pub(crate) fn static_class_identities_for_project(
    modules: &[(&str, &HirModule)],
    structural_record_identities: &HashSet<String>,
) -> HashMap<String, ShapeIdentity> {
    let context = IdentityContext { modules };
    modules
        .iter()
        .flat_map(|(module_name, module)| {
            let context = &context;
            module.classes.iter().filter_map(move |class| {
                let key = project_class_identity(class, module_name);
                let supported = if class.is_enum() {
                    crate::structural_impl_codegen::structural_enum_supported(class)
                } else {
                    structural_record_identities.contains(&key)
                };
                let compiled = supported.then(|| {
                    if class.is_enum() {
                        compile_enum(class, IdentityContext::wire_module_name(module_name))
                    } else {
                        compile_class(class, module_name, context)
                    }
                })?;
                compiled.static_value().map(|identity| (key, identity))
            })
        })
        .collect()
}

pub(crate) fn static_class_identity(
    class: &HirClass,
    module: &HirModule,
    module_name: Option<&str>,
) -> Option<ShapeIdentity> {
    let module_key = module_name.unwrap_or("");
    let modules = [(module_key, module)];
    let context = IdentityContext { modules: &modules };
    if class.is_enum() {
        compile_enum(class, module_name).static_value()
    } else {
        compile_class(class, module_key, &context).static_value()
    }
}

pub(crate) fn enum_identity_expression(class: &HirClass, module_name: Option<&str>) -> String {
    compile_enum(class, module_name).expression()
}

pub(crate) const fn algorithm_version() -> u32 {
    ALGORITHM_VERSION
}

pub(crate) fn class_identity_inputs_supported(class: &HirClass) -> bool {
    class
        .field_defaults
        .iter()
        .all(|(_, value)| canonical_structural_identity_value(value).is_some())
        && class
            .declaration_metadata
            .iter()
            .all(|metadata| canonical_structural_identity_value(&metadata.value).is_some())
}

fn compile_class(
    class: &HirClass,
    module_name: &str,
    context: &IdentityContext<'_>,
) -> CompiledIdentity {
    let nominal = class_nominal_identity(class, IdentityContext::wire_module_name(module_name));
    let mut stack = vec![nominal.clone()];
    let type_arguments = class
        .type_params
        .iter()
        .map(|name| {
            CompiledIdentity::Dynamic(format!(
                "<{name} as {STRUCTURAL}::StructuralType>::shape_identity()"
            ))
        })
        .collect::<Vec<_>>();
    compile_record(
        &nominal,
        &type_arguments,
        &class.fields,
        Some(class),
        module_name,
        context,
        &mut stack,
    )
}

fn compile_enum(class: &HirClass, module_name: Option<&str>) -> CompiledIdentity {
    let nominal = class_nominal_identity(class, module_name);
    let members = class
        .enum_variants
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect::<Vec<_>>();
    CompiledIdentity::Static(identity::enum_shape(
        &nominal,
        &members,
        class_metadata_identity(Some(class)),
    ))
}

fn compile_type(
    ty: &Type,
    scope_module_name: &str,
    context: &IdentityContext<'_>,
    stack: &mut Vec<String>,
) -> CompiledIdentity {
    match ty.resolve_alias() {
        Type::Int => CompiledIdentity::Static(identity::primitive("int")),
        Type::FixedInt(value) => CompiledIdentity::Static(identity::primitive(value.rust_name())),
        Type::Float => CompiledIdentity::Static(identity::primitive("f64")),
        Type::Bool => CompiledIdentity::Static(identity::primitive("bool")),
        Type::Str => CompiledIdentity::Static(identity::primitive("str")),
        Type::Bytes => CompiledIdentity::Static(identity::primitive("bytes")),
        Type::None => CompiledIdentity::Static(identity::primitive("None")),
        Type::TypeVar(name) => CompiledIdentity::Dynamic(format!(
            "<{name} as {STRUCTURAL}::StructuralType>::shape_identity()"
        )),
        Type::List(value) => compile_unary("list", value, scope_module_name, context, stack),
        Type::Set(value) => compile_unary("set", value, scope_module_name, context, stack),
        Type::Dict(key, value) => {
            let key = compile_type(key, scope_module_name, context, stack);
            let value = compile_type(value, scope_module_name, context, stack);
            match (key.static_value(), value.static_value()) {
                (Some(key), Some(value)) => {
                    CompiledIdentity::Static(identity::binary_container("mapping", key, value))
                }
                _ => CompiledIdentity::Dynamic(format!(
                    "{STRUCTURAL}::binary_container(\"mapping\", {}, {})",
                    key.expression(),
                    value.expression()
                )),
            }
        }
        Type::Tuple(values) => compile_many("tuple", values, scope_module_name, context, stack),
        Type::Union(values) => match optional_member(values) {
            Some(member) => compile_unary("optional", member, scope_module_name, context, stack),
            None => compile_many("union", values, scope_module_name, context, stack),
        },
        Type::Enum {
            identity: declared_identity,
            name,
            variants,
        } => {
            if let Some((module_name, _, class)) = context
                .class_candidate(declared_identity.as_deref(), name, scope_module_name)
                .filter(|(_, _, class)| class.is_enum())
            {
                compile_enum(class, IdentityContext::wire_module_name(module_name))
            } else {
                let nominal = declared_identity.clone().unwrap_or_else(|| {
                    IdentityContext::wire_module_name(scope_module_name)
                        .map_or_else(|| name.clone(), |module| format!("{module}.{name}"))
                });
                let members = variants
                    .iter()
                    .map(|(name, value)| (name.as_str(), *value))
                    .collect::<Vec<_>>();
                CompiledIdentity::Static(identity::enum_shape(
                    &nominal,
                    &members,
                    identity::metadata(&[]),
                ))
            }
        }
        Type::Class {
            identity: class_identity,
            type_args,
            name,
            fields,
            ..
        } => {
            let candidate =
                context.class_candidate(class_identity.as_deref(), name, scope_module_name);
            let nominal = candidate.map_or_else(
                || class_identity.clone().unwrap_or_else(|| name.clone()),
                |(module_name, _, class)| {
                    class_nominal_identity(class, IdentityContext::wire_module_name(module_name))
                },
            );
            if let Some((module_name, _, class)) = candidate {
                if class.is_enum() {
                    return compile_enum(class, IdentityContext::wire_module_name(module_name));
                }
            }
            if let Some(index) = stack.iter().position(|entry| entry == &nominal) {
                let Ok(index) = u32::try_from(index) else {
                    unreachable!(
                        "a compiler-owned structural recursion stack cannot exceed u32::MAX"
                    );
                };
                return CompiledIdentity::Static(identity::recursive_reference(index));
            }
            stack.push(nominal.clone());
            let arguments = type_args
                .iter()
                .map(|argument| compile_type(argument, scope_module_name, context, stack))
                .collect::<Vec<_>>();
            let concrete_fields = candidate.map_or_else(
                || fields.clone(),
                |(_, _, class)| {
                    let bindings = class
                        .type_params
                        .iter()
                        .cloned()
                        .zip(type_args.iter().cloned())
                        .collect::<HashMap<_, _>>();
                    class
                        .fields
                        .iter()
                        .map(|(name, ty)| (name.clone(), substitute_structural_type(ty, &bindings)))
                        .collect()
                },
            );
            let result = compile_record(
                &nominal,
                &arguments,
                &concrete_fields,
                candidate.map(|(_, _, class)| class),
                candidate.map_or(scope_module_name, |(module_name, _, _)| module_name),
                context,
                stack,
            );
            stack.pop();
            result
        }
        other => unreachable!(
            "unsupported structural type reached compiler-owned identity generation: {other:?}"
        ),
    }
}

fn substitute_structural_type(ty: &Type, bindings: &HashMap<String, Type>) -> Type {
    match ty {
        Type::TypeVar(name) => bindings.get(name).cloned().unwrap_or_else(|| ty.clone()),
        Type::List(value) => Type::List(Box::new(substitute_structural_type(value, bindings))),
        Type::Set(value) => Type::Set(Box::new(substitute_structural_type(value, bindings))),
        Type::Dict(key, value) => Type::Dict(
            Box::new(substitute_structural_type(key, bindings)),
            Box::new(substitute_structural_type(value, bindings)),
        ),
        Type::Tuple(values) => Type::Tuple(
            values
                .iter()
                .map(|value| substitute_structural_type(value, bindings))
                .collect(),
        ),
        Type::Union(values) => Type::Union(
            values
                .iter()
                .map(|value| substitute_structural_type(value, bindings))
                .collect(),
        ),
        Type::Class {
            identity,
            type_args,
            name,
            fields,
            methods,
            parent_class,
        } => Type::Class {
            identity: identity.clone(),
            type_args: type_args
                .iter()
                .map(|value| substitute_structural_type(value, bindings))
                .collect(),
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), substitute_structural_type(value, bindings)))
                .collect(),
            methods: methods.clone(),
            parent_class: parent_class.clone(),
        },
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|value| substitute_structural_type(value, bindings))
                .collect(),
            body: Box::new(substitute_structural_type(body, bindings)),
        },
        other => other.clone(),
    }
}

fn compile_record(
    nominal: &str,
    type_arguments: &[CompiledIdentity],
    fields: &[(String, Type)],
    class: Option<&HirClass>,
    scope_module_name: &str,
    context: &IdentityContext<'_>,
    stack: &mut Vec<String>,
) -> CompiledIdentity {
    let field_identities = fields
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| {
            let shape = compile_type(ty, scope_module_name, context, stack);
            let default = class.and_then(|class| field_default_identity(class, index));
            (name, shape, default)
        })
        .collect::<Vec<_>>();
    let metadata = class_metadata_identity(class);
    let static_arguments = type_arguments
        .iter()
        .map(CompiledIdentity::static_value)
        .collect::<Option<Vec<_>>>();
    let static_fields = field_identities
        .iter()
        .map(|(name, shape, default)| {
            Some(NominalField {
                name,
                identity: shape.static_value()?,
                required: default.is_none(),
                default_identity: *default,
            })
        })
        .collect::<Option<Vec<_>>>();
    if let (Some(arguments), Some(fields)) = (static_arguments, static_fields) {
        return CompiledIdentity::Static(identity::nominal_record(
            nominal, &arguments, &fields, metadata,
        ));
    }

    let arguments = type_arguments
        .iter()
        .map(CompiledIdentity::expression)
        .collect::<Vec<_>>()
        .join(", ");
    let fields = field_identities
        .iter()
        .map(|(name, shape, default)| {
            let required = default.is_none();
            let default = default.map_or_else(
                || "None".to_string(),
                |value| format!("Some({})", static_expression(value)),
            );
            format!(
                "{STRUCTURAL}::NominalField {{ name: {name:?}, identity: {}, required: {}, default_identity: {default} }}",
                shape.expression(),
                required
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    CompiledIdentity::Dynamic(format!(
        "{STRUCTURAL}::nominal_record({nominal:?}, &[{arguments}], &[{fields}], {})",
        static_expression(metadata)
    ))
}

fn compile_unary(
    tag: &str,
    value: &Type,
    scope_module_name: &str,
    context: &IdentityContext<'_>,
    stack: &mut Vec<String>,
) -> CompiledIdentity {
    let value = compile_type(value, scope_module_name, context, stack);
    value.static_value().map_or_else(
        || {
            CompiledIdentity::Dynamic(format!(
                "{STRUCTURAL}::unary_container({tag:?}, {})",
                value.expression()
            ))
        },
        |value| CompiledIdentity::Static(identity::unary_container(tag, value)),
    )
}

fn compile_many(
    tag: &str,
    values: &[Type],
    scope_module_name: &str,
    context: &IdentityContext<'_>,
    stack: &mut Vec<String>,
) -> CompiledIdentity {
    let values = values
        .iter()
        .map(|value| compile_type(value, scope_module_name, context, stack))
        .collect::<Vec<_>>();
    let static_values = values
        .iter()
        .map(CompiledIdentity::static_value)
        .collect::<Option<Vec<_>>>();
    if let Some(values) = static_values {
        return CompiledIdentity::Static(if tag == "tuple" {
            identity::tuple(&values)
        } else {
            identity::union(&values)
        });
    }
    let values = values
        .iter()
        .map(CompiledIdentity::expression)
        .collect::<Vec<_>>()
        .join(", ");
    CompiledIdentity::Dynamic(format!("{STRUCTURAL}::{tag}(&[{values}])"))
}

fn optional_member(values: &[Type]) -> Option<&Type> {
    if values.len() != 2
        || !values
            .iter()
            .any(|value| matches!(value.resolve_alias(), Type::None))
    {
        return None;
    }
    values
        .iter()
        .find(|value| !matches!(value.resolve_alias(), Type::None))
}

fn field_default_identity(class: &HirClass, field_index: usize) -> Option<ShapeIdentity> {
    class
        .field_defaults
        .iter()
        .find(|(index, _)| *index == field_index)
        .and_then(|(_, value)| canonical_structural_identity_value(value))
        .map(|value| identity::default_value(&value))
}

fn class_metadata_identity(class: Option<&HirClass>) -> ShapeIdentity {
    let mut entries = class
        .into_iter()
        .flat_map(|class| &class.declaration_metadata)
        .filter_map(|metadata| {
            canonical_structural_identity_value(&metadata.value).map(|value| {
                format!(
                    "{:?}|{}|{}|{:?}|{}",
                    metadata.target_kind,
                    metadata.target_name.as_deref().unwrap_or(""),
                    metadata.key,
                    metadata.value_type.resolve_alias(),
                    value
                )
            })
        })
        .collect::<Vec<_>>();
    entries.sort();
    identity::metadata(&entries.iter().map(String::as_str).collect::<Vec<_>>())
}

fn class_nominal_identity(class: &HirClass, module_name: Option<&str>) -> String {
    class.identity.clone().unwrap_or_else(|| {
        module_name.map_or_else(
            || class.name.clone(),
            |module| format!("{module}.{}", class.name),
        )
    })
}

fn project_class_identity(class: &HirClass, module_name: &str) -> String {
    class.identity.clone().unwrap_or_else(|| {
        if module_name.is_empty() {
            class.name.clone()
        } else {
            format!("{module_name}.{}", class.name)
        }
    })
}

fn static_expression(value: ShapeIdentity) -> String {
    let bytes = value
        .as_bytes()
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{STRUCTURAL}::ShapeIdentity::from_bytes([{bytes}])")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_ir::{DeclarationMetadataTargetKind, HirClassKind, HirExpr, TypedDeclarationMetadata};

    fn class(name: &str, fields: Vec<(String, Type)>) -> HirClass {
        HirClass {
            name: name.to_string(),
            identity: None,
            fields,
            field_defaults: Vec::new(),
            declaration_metadata: Vec::new(),
            methods: Vec::new(),
            is_hashable: false,
            is_error_type: false,
            kind: HirClassKind::Regular,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
            parent_type: None,
            type_params: Vec::new(),
            enum_variants: Vec::new(),
            rust_interop: Vec::new(),
        }
    }

    fn class_type(name: &str, fields: Vec<(String, Type)>) -> Type {
        Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: name.to_string(),
            fields,
            methods: Vec::new(),
            parent_class: None,
        }
    }

    #[test]
    fn structural_substitution_preserves_generic_union_grouping() {
        let template =
            sifr_type_system::make_union(vec![Type::Int, Type::TypeVar("T".to_string())]);
        let bindings = HashMap::from([("T".to_string(), Type::Bool)]);
        let actual = substitute_structural_type(&template, &bindings);

        assert_eq!(actual, Type::Union(vec![Type::Int, Type::Bool]));
    }

    #[test]
    fn generic_union_identity_matches_reordered_collapsed_and_nested_substitutions() {
        let modules = Vec::new();
        let context = IdentityContext { modules: &modules };
        let template =
            sifr_type_system::make_union(vec![Type::Str, Type::TypeVar("T".to_string())]);
        assert_eq!(
            compile_type(&template, "main", &context, &mut Vec::new()).expression(),
            format!(
                "{STRUCTURAL}::union(&[{STRUCTURAL}::ShapeIdentity::from_bytes({:?}), <T as {STRUCTURAL}::StructuralType>::shape_identity()])",
                identity::primitive("str").as_bytes()
            )
        );

        let reordered =
            substitute_structural_type(&template, &HashMap::from([("T".to_string(), Type::Int)]));
        let reordered_identity =
            compile_type(&reordered, "main", &context, &mut Vec::new()).static_value();
        assert_eq!(
            reordered_identity,
            Some(identity::union(&[
                identity::primitive("str"),
                identity::primitive("int"),
            ]))
        );

        let collapsed =
            substitute_structural_type(&template, &HashMap::from([("T".to_string(), Type::Str)]));
        let collapsed_identity =
            compile_type(&collapsed, "main", &context, &mut Vec::new()).static_value();
        assert_eq!(collapsed, Type::Union(vec![Type::Str, Type::Str]));
        assert_eq!(collapsed_identity, Some(identity::primitive("str")));

        let nested_argument = sifr_type_system::make_union(vec![Type::Int, Type::Bool]);
        let nested = substitute_structural_type(
            &template,
            &HashMap::from([("T".to_string(), nested_argument)]),
        );
        let nested_identity =
            compile_type(&nested, "main", &context, &mut Vec::new()).static_value();
        assert_eq!(
            nested_identity,
            Some(identity::union(&[
                identity::primitive("str"),
                identity::union(&[identity::primitive("bool"), identity::primitive("int"),]),
            ]))
        );

        let optional_template =
            sifr_type_system::make_union(vec![Type::None, Type::TypeVar("T".to_string())]);
        assert_eq!(
            compile_type(&optional_template, "main", &context, &mut Vec::new()).expression(),
            format!(
                "{STRUCTURAL}::unary_container(\"optional\", <T as {STRUCTURAL}::StructuralType>::shape_identity())"
            )
        );
        let optional_argument = sifr_type_system::make_union(vec![Type::None, Type::Str]);
        let nested_optional = substitute_structural_type(
            &optional_template,
            &HashMap::from([("T".to_string(), optional_argument)]),
        );
        let nested_optional_identity =
            compile_type(&nested_optional, "main", &context, &mut Vec::new()).static_value();
        assert_eq!(
            nested_optional_identity,
            Some(identity::unary_container(
                "optional",
                identity::unary_container("optional", identity::primitive("str")),
            ))
        );
    }

    #[test]
    fn exact_and_fixed_width_integer_identities_are_distinct() {
        let modules = Vec::new();
        let context = IdentityContext { modules: &modules };
        let exact = compile_type(&Type::Int, "main", &context, &mut Vec::new()).static_value();
        let fixed = compile_type(
            &Type::FixedInt(sifr_type_system::FixedIntType::I64),
            "main",
            &context,
            &mut Vec::new(),
        )
        .static_value();

        assert_eq!(exact, Some(identity::primitive("int")));
        assert_eq!(fixed, Some(identity::primitive("i64")));
        assert_ne!(exact, fixed);
    }

    #[test]
    fn structural_identity_recanonicalizes_raw_union_members() {
        let modules = Vec::new();
        let context = IdentityContext { modules: &modules };
        let raw = Type::Union(vec![Type::Int, Type::Bool]);
        let canonical = sifr_type_system::make_union(vec![Type::Int, Type::Bool]);
        let actual = compile_type(&raw, "main", &context, &mut Vec::new()).static_value();
        let expected = compile_type(&canonical, "main", &context, &mut Vec::new()).static_value();

        assert_eq!(actual, expected);
    }

    #[test]
    fn mutually_recursive_records_compile_to_static_numbered_identity() {
        let a_to_b = class_type("B", vec![("a".to_string(), class_type("A", Vec::new()))]);
        let b_to_a = class_type("A", vec![("b".to_string(), class_type("B", Vec::new()))]);
        let a = class("A", vec![("b".to_string(), a_to_b)]);
        let b = class("B", vec![("a".to_string(), b_to_a)]);
        let module = HirModule {
            functions: Vec::new(),
            classes: vec![a.clone(), b],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let expression = class_identity_expression(&a, &module, Some("main"));
        assert!(expression.contains("ShapeIdentity::from_bytes"));
        assert!(!expression.contains("<B as"));
    }

    #[test]
    fn defaults_and_metadata_change_precomputed_identity() {
        let plain = class("Payload", vec![("value".to_string(), Type::Int)]);
        let mut defaulted = plain.clone();
        defaulted.field_defaults = vec![(0, HirExpr::IntLiteral(1))];
        let mut annotated = plain.clone();
        annotated.declaration_metadata = vec![TypedDeclarationMetadata {
            owner: "Payload".to_string(),
            target_kind: DeclarationMetadataTargetKind::Field,
            target_name: Some("value".to_string()),
            key: "example.policy".to_string(),
            value_type: Type::Str,
            value: HirExpr::StringLiteral("strict".to_string()),
            range: Default::default(),
        }];
        let module = HirModule {
            functions: Vec::new(),
            classes: vec![plain.clone()],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let plain_identity = static_class_identity(&plain, &module, Some("main"));
        let default_identity = static_class_identity(&defaulted, &module, Some("main"));
        let metadata_identity = static_class_identity(&annotated, &module, Some("main"));

        assert_ne!(plain_identity, default_identity);
        assert_ne!(plain_identity, metadata_identity);
        assert_ne!(default_identity, metadata_identity);
    }

    #[test]
    fn project_identity_preserves_imported_defaults_and_metadata_inside_union() {
        let mut payload = class("Payload", vec![("value".to_string(), Type::Int)]);
        payload.field_defaults = vec![(0, HirExpr::IntLiteral(7))];
        payload.declaration_metadata = vec![TypedDeclarationMetadata {
            owner: "Payload".to_string(),
            target_kind: DeclarationMetadataTargetKind::Type,
            target_name: None,
            key: "example.policy".to_string(),
            value_type: Type::Str,
            value: HirExpr::StringLiteral("strict".to_string()),
            range: Default::default(),
        }];
        let models = HirModule {
            functions: Vec::new(),
            classes: vec![payload.clone()],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let payload_type = Type::Class {
            identity: Some("models.Payload".to_string()),
            type_args: Vec::new(),
            name: "Payload".to_string(),
            fields: vec![("value".to_string(), Type::Int)],
            methods: Vec::new(),
            parent_class: None,
        };
        let envelope = class(
            "Envelope",
            vec![(
                "payload".to_string(),
                Type::Union(vec![payload_type, Type::Str]),
            )],
        );
        let main = HirModule {
            functions: Vec::new(),
            classes: vec![envelope],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let modules = [("models", &models), ("main", &main)];
        let expressions = class_identity_expressions_for_project(
            &modules,
            &HashSet::from(["models.Payload".to_string(), "main.Envelope".to_string()]),
        );
        let payload_identity =
            static_class_identity(&payload, &models, Some("models")).expect("static payload");
        let expected = identity::nominal_record(
            "main.Envelope",
            &[],
            &[NominalField {
                name: "payload",
                identity: identity::union(&[identity::primitive("str"), payload_identity]),
                required: true,
                default_identity: None,
            }],
            identity::metadata(&[]),
        );

        assert_eq!(
            expressions.get("main.Envelope"),
            Some(&static_expression(expected))
        );
    }
}
