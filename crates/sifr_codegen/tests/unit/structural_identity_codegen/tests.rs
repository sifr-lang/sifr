use super::*;
use sifr_ir::{DeclarationMetadataTargetKind, HirClassKind, HirExpr, TypedDeclarationMetadata};

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

fn class_type(name: &str, fields: Vec<(String, Type)>) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: fields.into(),
        methods: Vec::new().into(),
        parent_class: None,
    }
}

#[test]
fn exact_and_fixed_width_integer_identities_are_distinct() {
    let modules = Vec::new();
    let context = IdentityContext {
        modules: &modules,
        nominal_type_paths: None,
    };
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
    let context = IdentityContext {
        modules: &modules,
        nominal_type_paths: None,
    };
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
        fields: vec![("value".to_string(), Type::Int)].into(),
        methods: Vec::new().into(),
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
        &HashMap::new(),
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
