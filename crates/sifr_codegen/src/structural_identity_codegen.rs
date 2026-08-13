use sifr_ir::{HirClass, HirExpr, HirModule};
use sifr_structural_identity::{self as identity, NominalField, ShapeIdentity, ALGORITHM_VERSION};
use sifr_type_system::Type;
use std::collections::HashMap;

const STRUCTURAL: &str = "::sifr_runtime::interop::structural";

#[derive(Debug)]
enum CompiledIdentity {
    Static(ShapeIdentity),
    Dynamic(String),
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
    if class.is_enum() {
        compile_enum(class, module_name).expression()
    } else {
        compile_class(class, module, module_name).expression()
    }
}

pub(crate) fn static_class_identity(
    class: &HirClass,
    module: &HirModule,
    module_name: Option<&str>,
) -> Option<ShapeIdentity> {
    if class.is_enum() {
        compile_enum(class, module_name).static_value()
    } else {
        compile_class(class, module, module_name).static_value()
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
        .all(|(_, value)| canonical_hir_value(value).is_some())
        && class
            .declaration_metadata
            .iter()
            .all(|metadata| canonical_hir_value(&metadata.value).is_some())
}

fn compile_class(
    class: &HirClass,
    module: &HirModule,
    module_name: Option<&str>,
) -> CompiledIdentity {
    let nominal = class_nominal_identity(class, module_name);
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
        module,
        module_name,
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
    module: &HirModule,
    module_name: Option<&str>,
    stack: &mut Vec<String>,
) -> CompiledIdentity {
    match ty.resolve_alias() {
        Type::Int => CompiledIdentity::Static(identity::primitive("i64")),
        Type::FixedInt(value) => CompiledIdentity::Static(identity::primitive(value.rust_name())),
        Type::Float => CompiledIdentity::Static(identity::primitive("f64")),
        Type::Bool => CompiledIdentity::Static(identity::primitive("bool")),
        Type::Str => CompiledIdentity::Static(identity::primitive("str")),
        Type::Bytes => CompiledIdentity::Static(identity::primitive("bytes")),
        Type::None => CompiledIdentity::Static(identity::primitive("None")),
        Type::TypeVar(name) => CompiledIdentity::Dynamic(format!(
            "<{name} as {STRUCTURAL}::StructuralType>::shape_identity()"
        )),
        Type::List(value) => compile_unary("list", value, module, module_name, stack),
        Type::Set(value) => compile_unary("set", value, module, module_name, stack),
        Type::Dict(key, value) => {
            let key = compile_type(key, module, module_name, stack);
            let value = compile_type(value, module, module_name, stack);
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
        Type::Tuple(values) => compile_many("tuple", values, module, module_name, stack),
        Type::Union(values) => match optional_member(values) {
            Some(member) => compile_unary("optional", member, module, module_name, stack),
            None => compile_many("union", values, module, module_name, stack),
        },
        Type::Enum {
            identity: declared_identity,
            name,
            variants,
        } => {
            if let Some(class) = module
                .classes
                .iter()
                .find(|class| class.name == *name && class.is_enum())
            {
                compile_enum(class, module_name)
            } else {
                let nominal = declared_identity.clone().unwrap_or_else(|| {
                    module_name.map_or_else(|| name.clone(), |module| format!("{module}.{name}"))
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
            let candidate = module.classes.iter().find(|class| class.name == *name);
            let nominal = class_identity.clone().unwrap_or_else(|| {
                candidate.map_or_else(
                    || name.clone(),
                    |class| class_nominal_identity(class, module_name),
                )
            });
            if let Some(class) = candidate {
                if class.is_enum() {
                    return compile_enum(class, module_name);
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
                .map(|argument| compile_type(argument, module, module_name, stack))
                .collect::<Vec<_>>();
            let concrete_fields = candidate.map_or_else(
                || fields.clone(),
                |class| {
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
                candidate,
                module,
                module_name,
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
    module: &HirModule,
    module_name: Option<&str>,
    stack: &mut Vec<String>,
) -> CompiledIdentity {
    let field_identities = fields
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| {
            let shape = compile_type(ty, module, module_name, stack);
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
    module: &HirModule,
    module_name: Option<&str>,
    stack: &mut Vec<String>,
) -> CompiledIdentity {
    let value = compile_type(value, module, module_name, stack);
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
    module: &HirModule,
    module_name: Option<&str>,
    stack: &mut Vec<String>,
) -> CompiledIdentity {
    let values = values
        .iter()
        .map(|value| compile_type(value, module, module_name, stack))
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
        .and_then(|(_, value)| canonical_hir_value(value))
        .map(|value| identity::default_value(&value))
}

fn class_metadata_identity(class: Option<&HirClass>) -> ShapeIdentity {
    let mut entries = class
        .into_iter()
        .flat_map(|class| &class.declaration_metadata)
        .filter_map(|metadata| {
            canonical_hir_value(&metadata.value).map(|value| {
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

fn canonical_hir_value(value: &HirExpr) -> Option<String> {
    match value {
        HirExpr::IntLiteral(value) => Some(format!("int:{value}")),
        HirExpr::LargeIntLiteral(value) => Some(format!("int:{value}")),
        HirExpr::FloatLiteral(value) => Some(format!("float:{:016x}", value.to_bits())),
        HirExpr::StringLiteral(value) => Some(format!("str:{}:{value}", value.len())),
        HirExpr::BoolLiteral(value) => Some(format!("bool:{value}")),
        HirExpr::NoneLiteral => Some("none".to_string()),
        HirExpr::ListLiteral { elements, .. } => canonical_sequence("list", elements),
        HirExpr::TupleLiteral { elements, .. } => canonical_sequence("tuple", elements),
        HirExpr::SetLiteral { elements, .. } => {
            let mut elements = elements
                .iter()
                .map(canonical_hir_value)
                .collect::<Option<Vec<_>>>()?;
            elements.sort();
            Some(format!("set[{}]", elements.join(",")))
        }
        HirExpr::DictLiteral { keys, values, .. } if keys.len() == values.len() => {
            let mut entries = keys
                .iter()
                .zip(values)
                .map(|(key, value)| {
                    Some(format!(
                        "{}={}",
                        canonical_hir_value(key)?,
                        canonical_hir_value(value)?
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            entries.sort();
            Some(format!("dict[{}]", entries.join(",")))
        }
        _ => None,
    }
}

fn canonical_sequence(tag: &str, values: &[HirExpr]) -> Option<String> {
    let values = values
        .iter()
        .map(canonical_hir_value)
        .collect::<Option<Vec<_>>>()?;
    Some(format!("{tag}[{}]", values.join(",")))
}

fn class_nominal_identity(class: &HirClass, module_name: Option<&str>) -> String {
    class.identity.clone().unwrap_or_else(|| {
        module_name.map_or_else(
            || class.name.clone(),
            |module| format!("{module}.{}", class.name),
        )
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
    use sifr_ir::{DeclarationMetadataTargetKind, HirClassKind, TypedDeclarationMetadata};

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
}
