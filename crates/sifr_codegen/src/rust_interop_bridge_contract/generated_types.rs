use super::{
    ModuleCatalog, RustGeneratedBridgeField, RustGeneratedBridgeType, RustGeneratedBridgeTypeKind,
    RustGeneratedBridgeVariant,
};
use sifr_ir::{HirClass, RustInteropDecoratorKind, RustInteropValue};
use sifr_type_system::Type;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct GeneratedTypeCollector {
    entries: BTreeMap<(Option<String>, String), RustGeneratedBridgeType>,
    in_progress: BTreeSet<(Option<String>, String)>,
}

impl GeneratedTypeCollector {
    pub(super) fn into_types(self) -> Vec<RustGeneratedBridgeType> {
        self.entries.into_values().collect()
    }

    pub(super) fn insert_record(
        &mut self,
        module_name: Option<&String>,
        name: &str,
        fields: &[(String, Type)],
        is_error: bool,
        module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    ) {
        let key = (module_name.cloned(), name.to_string());
        if self.entries.contains_key(&key) || self.in_progress.contains(&key) {
            return;
        }
        self.in_progress.insert(key.clone());
        let mut bridge_fields = Vec::with_capacity(fields.len());
        for (field_name, field_ty) in fields {
            let rust_type = self.generated_field_rust_type(field_ty, module_name, module_catalogs);
            bridge_fields.push(RustGeneratedBridgeField {
                name: field_name.clone(),
                sifr_type: field_ty.display_name(),
                rust_type,
            });
        }
        self.in_progress.remove(&key);
        self.entries.insert(
            key,
            RustGeneratedBridgeType {
                module_name: module_name.cloned(),
                name: format!("{name}Bridge"),
                rust_type_path: generated_bridge_type_path(module_name, name),
                kind: if is_error {
                    RustGeneratedBridgeTypeKind::Error
                } else {
                    RustGeneratedBridgeTypeKind::Record
                },
                fields: bridge_fields,
                variants: Vec::new(),
            },
        );
    }

    pub(super) fn insert_enum(
        &mut self,
        module_name: Option<&String>,
        name: &str,
        variants: &[(String, Option<i64>)],
    ) -> Result<(), String> {
        let key = (module_name.cloned(), name.to_string());
        if self.entries.contains_key(&key) {
            return Ok(());
        }
        let mut next = 0_u32;
        let mut bridge_variants = Vec::with_capacity(variants.len());
        for (variant, explicit) in variants {
            let discriminant = if let Some(value) = explicit {
                match u32::try_from(*value) {
                    Ok(value) => value,
                    Err(_) => return Err(enum_discriminant_reason(name, variant, *value)),
                }
            } else {
                next
            };
            next = discriminant.saturating_add(1);
            bridge_variants.push(RustGeneratedBridgeVariant {
                name: variant.clone(),
                discriminant,
            });
        }
        self.entries.insert(
            key,
            RustGeneratedBridgeType {
                module_name: module_name.cloned(),
                name: format!("{name}Bridge"),
                rust_type_path: generated_bridge_type_path(module_name, name),
                kind: RustGeneratedBridgeTypeKind::ClosedEnum,
                fields: Vec::new(),
                variants: bridge_variants,
            },
        );
        Ok(())
    }

    fn generated_field_rust_type(
        &mut self,
        ty: &Type,
        module_name: Option<&String>,
        module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    ) -> String {
        match ty.resolve_alias() {
            Type::FixedInt(fixed) => fixed.rust_name().to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int => "sifr_runtime::interop::SifrIntBridge".to_string(),
            Type::Float => "f64".to_string(),
            Type::Str => "String".to_string(),
            Type::Bytes => "Vec<u8>".to_string(),
            Type::None => "()".to_string(),
            Type::List(inner) => format!(
                "Vec<{}>",
                self.generated_field_rust_type(inner, module_name, module_catalogs)
            ),
            Type::Dict(key, value) if matches!(key.resolve_alias(), Type::Str) => {
                format!(
                    "sifr_runtime::interop::IndexMap<String, {}>",
                    self.generated_field_rust_type(value, module_name, module_catalogs)
                )
            }
            Type::Class {
                name,
                fields,
                parent_class,
                ..
            } => {
                let definition_module =
                    bridge_type_definition_module(name, module_name, module_catalogs, false)
                        .unwrap_or_else(|_| module_name.cloned());
                self.insert_record(
                    definition_module.as_ref(),
                    name,
                    fields,
                    parent_class.as_deref() == Some("Error"),
                    module_catalogs,
                );
                generated_bridge_type_path(definition_module.as_ref(), name)
            }
            Type::Enum { name, variants } => {
                let definition_module =
                    bridge_type_definition_module(name, module_name, module_catalogs, true)
                        .unwrap_or_else(|_| module_name.cloned());
                let _ = self.insert_enum(definition_module.as_ref(), name, variants);
                generated_bridge_type_path(definition_module.as_ref(), name)
            }
            Type::Union(members) if members.len() == 2 => {
                let has_none = members
                    .iter()
                    .any(|member| matches!(member.resolve_alias(), Type::None));
                let non_none = members
                    .iter()
                    .find(|member| !matches!(member.resolve_alias(), Type::None));
                if has_none {
                    if let Some(non_none) = non_none {
                        return format!(
                            "Option<{}>",
                            self.generated_field_rust_type(non_none, module_name, module_catalogs)
                        );
                    }
                }
                ty.display_name()
            }
            _ => ty.display_name(),
        }
    }
}

pub(super) fn generated_bridge_type_path(module_name: Option<&String>, name: &str) -> String {
    let module = rust_bridge_module_name(module_name.map(String::as_str));
    format!("crate::__sifr_bridge::{module}::{name}Bridge")
}

fn rust_bridge_module_name(module_name: Option<&str>) -> String {
    module_name
        .unwrap_or("__sifr_binary_entry")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub(super) fn bridge_type_definition_module(
    name: &str,
    current_module: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    is_enum: bool,
) -> Result<Option<String>, String> {
    let current_module_key = current_module.cloned();
    if module_catalogs
        .get(&current_module_key)
        .is_some_and(|catalog| catalog_contains_bridge_type(catalog, name, is_enum))
    {
        return Ok(current_module_key);
    }

    let matches = module_catalogs
        .iter()
        .filter_map(|(module_name, catalog)| {
            catalog_contains_bridge_type(catalog, name, is_enum).then_some(module_name.clone())
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [module_name] => Ok(module_name.clone()),
        [] => Ok(current_module_key),
        _ => Err(format!(
            "bridge type `{name}` is ambiguous across Sifr modules; qualify or avoid duplicate exported bridge type names"
        )),
    }
}

fn catalog_contains_bridge_type(catalog: &ModuleCatalog, name: &str, is_enum: bool) -> bool {
    if is_enum {
        catalog.enum_classes.contains(name)
    } else {
        catalog.record_classes.contains(name)
    }
}

pub(super) fn opaque_rust_type_path(class: &HirClass) -> Option<String> {
    class
        .rust_interop
        .iter()
        .find(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)?
        .arguments
        .iter()
        .find(|argument| argument.name.as_deref() == Some("type"))
        .and_then(|argument| match &argument.value {
            RustInteropValue::TargetPath(path) => Some(path.dotted()),
            _ => None,
        })
}

pub(super) fn opaque_type_definition(
    name: &str,
    current_module: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
) -> Result<Option<String>, String> {
    let current_key = current_module.cloned();
    if let Some(target) = module_catalogs
        .get(&current_key)
        .and_then(|catalog| catalog.opaque_classes.get(name))
    {
        return Ok(Some(target.clone()));
    }
    let matches = module_catalogs
        .values()
        .filter_map(|catalog| catalog.opaque_classes.get(name))
        .cloned()
        .collect::<BTreeSet<_>>();
    match matches.len() {
        0 => Ok(None),
        1 => Ok(matches.into_iter().next()),
        _ => Err(format!(
            "opaque bridge type `{name}` is ambiguous across Sifr modules"
        )),
    }
}

fn enum_discriminant_reason(name: &str, variant: &str, value: i64) -> String {
    format!(
        "enum `{name}.{variant}` discriminant {value} is outside the Rust bridge repr(u32) range"
    )
}
