use sifr_ir::HirClass;
use std::collections::HashMap;

const STRUCTURAL: &str = "::sifr_runtime::interop::structural";

pub(super) fn mapped_opaque_identity_expression(class: &HirClass) -> Option<String> {
    if !crate::structural_impl_codegen::structural_mapped_opaque_supported(class) {
        return None;
    }
    let native = sifr_ir::rust_opaque_type_path(&class.rust_interop)?;
    let mapping = sifr_ir::rust_opaque_structural_mapping(&class.rust_interop)?;
    Some(format!(
        "<{} as {STRUCTURAL}::StructuralMapping<{}>>::shape_identity()",
        absolute_rust_path(&mapping.dotted()),
        absolute_rust_path(&native.dotted()),
    ))
}

pub(super) fn mapped_opaque_identity_expression_for_imported_type(
    class: &HirClass,
    module_name: &str,
    nominal_type_paths: &HashMap<String, String>,
) -> Option<String> {
    if !crate::structural_impl_codegen::structural_mapped_opaque_supported(class) {
        return None;
    }
    let canonical = class
        .identity
        .clone()
        .unwrap_or_else(|| format!("{module_name}.{}", class.name));
    let generated_path = nominal_type_paths.get(&canonical)?;
    Some(format!(
        "<{generated_path} as {STRUCTURAL}::StructuralType>::shape_identity()"
    ))
}

fn absolute_rust_path(path: &str) -> String {
    let path = path.replace('.', "::");
    if path.starts_with("::") {
        path
    } else {
        format!("::{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_ir::{
        HirClassKind, HirModule, RustInteropAbiRequirements, RustInteropArgument,
        RustInteropDeclaration, RustInteropDecoratorKind, RustInteropEffect, RustInteropValue,
        RustTargetPath,
    };
    use sifr_type_system::Type;
    use std::collections::{HashMap, HashSet};

    fn class(name: &str, fields: Vec<(String, Type)>) -> HirClass {
        HirClass {
            name: name.to_string(),
            identity: None,
            fields,
            field_defaults: Vec::new(),
            field_default_identities: Vec::new(),
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

    #[test]
    fn imported_identity_uses_the_generated_package_type() {
        let mut token = class("Token", Vec::new());
        token.identity = Some("values.Token".to_string());
        token.rust_interop = vec![RustInteropDeclaration {
            kind: RustInteropDecoratorKind::Opaque,
            target: None,
            arguments: vec![
                RustInteropArgument {
                    name: Some("type".to_string()),
                    value: RustInteropValue::TargetPath(RustTargetPath {
                        segments: ["bridge", "token", "Token"].map(str::to_string).to_vec(),
                        span: Default::default(),
                    }),
                    span: Default::default(),
                },
                RustInteropArgument {
                    name: Some("structural".to_string()),
                    value: RustInteropValue::TargetPath(RustTargetPath {
                        segments: ["bridge", "token", "TokenMapping"]
                            .map(str::to_string)
                            .to_vec(),
                        span: Default::default(),
                    }),
                    span: Default::default(),
                },
            ],
            span: Default::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements {
                opaque_handle: true,
                ..RustInteropAbiRequirements::default()
            },
            consumes_receiver: false,
        }];
        let values = HirModule {
            functions: Vec::new(),
            classes: vec![token],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let mut envelope = class(
            "Envelope",
            vec![(
                "token".to_string(),
                Type::Class {
                    identity: Some("values.Token".to_string()),
                    type_args: Vec::new(),
                    name: "Token".to_string(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: None,
                },
            )],
        );
        envelope.identity = Some("main.Envelope".to_string());
        let main = HirModule {
            functions: Vec::new(),
            classes: vec![envelope],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let expressions = super::super::class_identity_expressions_for_project(
            &[("values", &values), ("main", &main)],
            &HashSet::from(["main.Envelope".to_string()]),
            &HashMap::from([(
                "values.Token".to_string(),
                "crate::values::Token".to_string(),
            )]),
        );
        let expression = expressions
            .get("main.Envelope")
            .expect("consumer identity must be generated");
        assert!(
            expression.contains(
                "<crate::values::Token as ::sifr_runtime::interop::structural::StructuralType>::shape_identity()"
            ),
            "{expression}"
        );
        assert!(!expression.contains("::bridge::"), "{expression}");
        let static_identities = super::super::static_class_identities_for_project(
            &[("values", &values), ("main", &main)],
            &HashSet::from(["main.Envelope".to_string()]),
        );
        assert!(
            !static_identities.contains_key("main.Envelope"),
            "mapped identities must remain dynamic: {static_identities:?}"
        );
    }

    #[test]
    fn imported_identity_uses_crate_root_and_escaped_project_paths() {
        let mut token = class("std", Vec::new());
        token.identity = Some("main.std".to_string());
        token.rust_interop = vec![RustInteropDeclaration {
            kind: RustInteropDecoratorKind::Opaque,
            target: None,
            arguments: vec![
                RustInteropArgument {
                    name: Some("type".to_string()),
                    value: RustInteropValue::TargetPath(RustTargetPath {
                        segments: ["bridge", "token", "Token"].map(str::to_string).to_vec(),
                        span: Default::default(),
                    }),
                    span: Default::default(),
                },
                RustInteropArgument {
                    name: Some("structural".to_string()),
                    value: RustInteropValue::TargetPath(RustTargetPath {
                        segments: ["bridge", "token", "TokenMapping"]
                            .map(str::to_string)
                            .to_vec(),
                        span: Default::default(),
                    }),
                    span: Default::default(),
                },
            ],
            span: Default::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements {
                opaque_handle: true,
                ..RustInteropAbiRequirements::default()
            },
            consumes_receiver: false,
        }];
        let expression = mapped_opaque_identity_expression_for_imported_type(
            &token,
            "main",
            &HashMap::from([(
                "main.std".to_string(),
                "crate::__SifrSource_std".to_string(),
            )]),
        )
        .expect("mapped opaque identity must be available");

        assert!(
            expression.contains("<crate::__SifrSource_std as"),
            "{expression}"
        );
        assert!(!expression.contains("crate::main::std"), "{expression}");
    }
}
