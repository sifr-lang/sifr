use super::*;

#[test]
fn mapper_target_requires_the_named_map_error_policy_shape() {
    let target = RustTargetPath {
        segments: vec![
            "bridge".to_string(),
            "wrapper".to_string(),
            "map".to_string(),
        ],
        span: ruff_text_size::TextRange::default(),
    };
    let declaration = RustInteropDeclaration {
        kind: sifr_ir::RustInteropDecoratorKind::Function,
        target: None,
        arguments: vec![sifr_ir::RustInteropArgument {
            name: Some("panic".to_string()),
            value: RustInteropValue::PolicyCall {
                name: "map_error".to_string(),
                argument: Box::new(RustInteropValue::TargetPath(target.clone())),
                span: ruff_text_size::TextRange::default(),
            },
            span: ruff_text_size::TextRange::default(),
        }],
        span: ruff_text_size::TextRange::default(),
        effect: sifr_ir::RustInteropEffect::Sync,
        abi_requirements: sifr_ir::RustInteropAbiRequirements::default(),
        consumes_receiver: false,
    };

    assert_eq!(map_error_target(&declaration), Some(&target));
}
