use sifr_codegen::{
    RustGeneratedBridgeType, RustGeneratedBridgeTypeKind, RustGeneratedBridgeVariant,
};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

pub(super) fn generated_bridge_sources(
    bridge_types: &[RustGeneratedBridgeType],
) -> Result<BTreeMap<String, String>, String> {
    let mut canonical_modules = BTreeMap::<String, Option<String>>::new();
    let mut unique_types = BTreeMap::<(String, String), &RustGeneratedBridgeType>::new();
    let mut modules: BTreeMap<String, Vec<&RustGeneratedBridgeType>> = BTreeMap::new();
    for bridge_type in bridge_types {
        let module_name = bridge_module_name(bridge_type.module_name.as_deref());
        match canonical_modules.entry(module_name.clone()) {
            Entry::Occupied(entry) if entry.get() != &bridge_type.module_name => {
                return Err(format!(
                    "generated Rust bridge modules {:?} and {:?} collide at `{module_name}`",
                    entry.get(),
                    bridge_type.module_name
                ));
            }
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                entry.insert(bridge_type.module_name.clone());
            }
        }
        let key = (module_name, bridge_type.name.clone());
        match unique_types.entry(key) {
            Entry::Occupied(entry) => {
                if *entry.get() != bridge_type {
                    let (module_name, type_name) = entry.key();
                    return Err(format!(
                        "conflicting generated Rust bridge bodies share `{type_name}` in module `{module_name}`"
                    ));
                }
            }
            Entry::Vacant(entry) => {
                let module_name = entry.key().0.clone();
                entry.insert(bridge_type);
                modules.entry(module_name).or_default().push(bridge_type);
            }
        }
    }
    if modules.is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut sources = BTreeMap::new();
    let mut root = String::new();
    for (module_name, bridge_types) in modules {
        root.push_str("pub mod ");
        root.push_str(&module_name);
        root.push_str(";\n");
        let mut module_source = String::new();
        for bridge_type in bridge_types {
            if !module_source.is_empty() {
                module_source.push('\n');
            }
            module_source.push_str(&render_bridge_type(bridge_type));
        }
        sources.insert(format!("__sifr_bridge::{module_name}"), module_source);
    }
    sources.insert("__sifr_bridge".to_string(), root);
    Ok(sources)
}

fn bridge_module_name(module_name: Option<&str>) -> String {
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

fn render_bridge_type(bridge_type: &RustGeneratedBridgeType) -> String {
    match bridge_type.kind {
        RustGeneratedBridgeTypeKind::Record | RustGeneratedBridgeTypeKind::Error => {
            render_record_bridge_type(bridge_type)
        }
        RustGeneratedBridgeTypeKind::ClosedEnum => render_enum_bridge_type(bridge_type),
    }
}

fn render_record_bridge_type(bridge_type: &RustGeneratedBridgeType) -> String {
    let mut out = String::new();
    if bridge_type.supports_eq {
        out.push_str("#[derive(Clone, Debug, PartialEq, Eq)]\n");
    } else {
        out.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    }
    out.push_str("pub struct ");
    out.push_str(&bridge_type.name);
    out.push_str(" {\n");
    for field in &bridge_type.fields {
        out.push_str("    pub ");
        out.push_str(&field.name);
        out.push_str(": ");
        out.push_str(&field.rust_type);
        out.push_str(",\n");
    }
    out.push_str("}\n");
    out
}

fn render_enum_bridge_type(bridge_type: &RustGeneratedBridgeType) -> String {
    let mut out = String::new();
    out.push_str("#[repr(u32)]\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n");
    out.push_str("pub enum ");
    out.push_str(&bridge_type.name);
    out.push_str(" {\n");
    for variant in &bridge_type.variants {
        render_enum_variant(&mut out, variant);
    }
    out.push_str("}\n");
    out
}

fn render_enum_variant(out: &mut String, variant: &RustGeneratedBridgeVariant) {
    out.push_str("    ");
    out.push_str(&variant.name);
    out.push_str(" = ");
    out.push_str(&variant.discriminant.to_string());
    out.push_str(",\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_codegen::{
        RustGeneratedBridgeField, RustGeneratedBridgeType, RustGeneratedBridgeTypeKind,
        RustGeneratedBridgeVariant,
    };

    #[test]
    fn generated_bridge_sources_render_module_record_and_enum() {
        let sources = generated_bridge_sources(&[
            RustGeneratedBridgeType {
                module_name: Some("app".to_string()),
                name: "TokenBridge".to_string(),
                rust_type_path: "crate::__sifr_bridge::app::TokenBridge".to_string(),
                kind: RustGeneratedBridgeTypeKind::Record,
                supports_eq: true,
                fields: vec![RustGeneratedBridgeField {
                    name: "text".to_string(),
                    sifr_type: "str".to_string(),
                    rust_type: "String".to_string(),
                }],
                variants: Vec::new(),
            },
            RustGeneratedBridgeType {
                module_name: Some("app".to_string()),
                name: "KindBridge".to_string(),
                rust_type_path: "crate::__sifr_bridge::app::KindBridge".to_string(),
                kind: RustGeneratedBridgeTypeKind::ClosedEnum,
                supports_eq: true,
                fields: Vec::new(),
                variants: vec![RustGeneratedBridgeVariant {
                    name: "Word".to_string(),
                    discriminant: 1,
                }],
            },
        ])
        .expect("bridge sources should be canonical");

        assert_eq!(
            sources.get("__sifr_bridge").map(String::as_str),
            Some("pub mod app;\n")
        );
        let app_source = sources
            .get("__sifr_bridge::app")
            .expect("app bridge source");
        assert!(app_source.contains("pub struct TokenBridge"));
        assert!(app_source.contains("pub text: String"));
        assert!(app_source.contains("#[repr(u32)]"));
        assert!(app_source.contains("Word = 1"));
        assert!(app_source.contains("}\n\n#[repr(u32)]"));
        assert!(app_source.ends_with("}\n"));
        assert!(!app_source.ends_with("}\n\n"));
    }

    #[test]
    fn generated_bridge_sources_emit_an_identical_type_once() {
        let bridge_type = RustGeneratedBridgeType {
            module_name: Some("app".to_string()),
            name: "TokenBridge".to_string(),
            rust_type_path: "crate::__sifr_bridge::app::TokenBridge".to_string(),
            kind: RustGeneratedBridgeTypeKind::Record,
            supports_eq: true,
            fields: vec![RustGeneratedBridgeField {
                name: "text".to_string(),
                sifr_type: "str".to_string(),
                rust_type: "String".to_string(),
            }],
            variants: Vec::new(),
        };
        let sources = generated_bridge_sources(&[bridge_type.clone(), bridge_type])
            .expect("identical bridge demand should deduplicate");
        let source = &sources["__sifr_bridge::app"];
        assert_eq!(source.matches("pub struct TokenBridge").count(), 1);
    }

    #[test]
    fn generated_bridge_sources_reject_conflicting_bodies() {
        let mut first = RustGeneratedBridgeType {
            module_name: Some("app".to_string()),
            name: "TokenBridge".to_string(),
            rust_type_path: "crate::__sifr_bridge::app::TokenBridge".to_string(),
            kind: RustGeneratedBridgeTypeKind::Record,
            supports_eq: true,
            fields: Vec::new(),
            variants: Vec::new(),
        };
        let mut second = first.clone();
        second.supports_eq = false;
        let error = generated_bridge_sources(&[first.clone(), second])
            .expect_err("conflicting bridge bodies must fail closed");
        assert!(error.contains("conflicting generated Rust bridge bodies"));

        first.module_name = Some("app-name".to_string());
        let mut collision = first.clone();
        collision.module_name = Some("app.name".to_string());
        let error = generated_bridge_sources(&[first, collision])
            .expect_err("canonical module collision must fail closed");
        assert!(error.contains("collide"));
    }
}
