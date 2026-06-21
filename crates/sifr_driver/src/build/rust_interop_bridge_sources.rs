use sifr_codegen::{
    RustGeneratedBridgeType, RustGeneratedBridgeTypeKind, RustGeneratedBridgeVariant,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) fn generated_bridge_sources(
    bridge_types: &[RustGeneratedBridgeType],
) -> BTreeMap<PathBuf, String> {
    let mut modules: BTreeMap<String, Vec<&RustGeneratedBridgeType>> = BTreeMap::new();
    for bridge_type in bridge_types {
        modules
            .entry(bridge_module_name(bridge_type.module_name.as_deref()))
            .or_default()
            .push(bridge_type);
    }
    if modules.is_empty() {
        return BTreeMap::new();
    }

    let mut sources = BTreeMap::new();
    let mut root = String::new();
    for (module_name, bridge_types) in modules {
        root.push_str("pub mod ");
        root.push_str(&module_name);
        root.push_str(";\n");
        let mut module_source = String::new();
        for bridge_type in bridge_types {
            module_source.push_str(&render_bridge_type(bridge_type));
            module_source.push('\n');
        }
        sources.insert(
            PathBuf::from("__sifr_bridge").join(format!("{module_name}.rs")),
            module_source,
        );
    }
    sources.insert(PathBuf::from("__sifr_bridge/mod.rs"), root);
    sources
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
    out.push_str("#[derive(Clone, Debug, PartialEq)]\n");
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
                fields: Vec::new(),
                variants: vec![RustGeneratedBridgeVariant {
                    name: "Word".to_string(),
                    discriminant: 1,
                }],
            },
        ]);

        assert_eq!(
            sources
                .get(&PathBuf::from("__sifr_bridge/mod.rs"))
                .map(String::as_str),
            Some("pub mod app;\n")
        );
        let app_source = sources
            .get(&PathBuf::from("__sifr_bridge/app.rs"))
            .expect("app bridge source");
        assert!(app_source.contains("pub struct TokenBridge"));
        assert!(app_source.contains("pub text: String"));
        assert!(app_source.contains("#[repr(u32)]"));
        assert!(app_source.contains("Word = 1"));
    }
}
