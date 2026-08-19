use super::*;
use sifr_ir::HirClassKind;

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
fn structural_substitution_canonicalizes_generic_union_members() {
    let modules = Vec::new();
    let context = IdentityContext { modules: &modules };
    let template = sifr_type_system::make_union(vec![Type::Int, Type::TypeVar("T".to_string())]);
    let bindings = HashMap::from([("T".to_string(), Type::Bool)]);
    let actual = substitute_structural_type(&template, &bindings, "main", &context);

    assert_eq!(actual, Type::Union(vec![Type::Bool, Type::Int]));
}

#[test]
fn structural_substitution_uses_project_nominal_scope_authority() {
    let mut inner = class(
        "Inner",
        vec![("value".to_string(), Type::TypeVar("T".to_string()))],
    );
    inner.type_params = vec!["T".to_string()];
    let module = HirModule {
        functions: Vec::new(),
        classes: vec![inner],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };
    let modules = [("models", &module)];
    let context = IdentityContext { modules: &modules };
    let template = Type::Class {
        identity: Some("models.Inner".to_string()),
        type_args: vec![Type::Str],
        name: "Inner".to_string(),
        fields: vec![("value".to_string(), Type::TypeVar("T".to_string()))],
        methods: Vec::new(),
        parent_class: None,
    };
    let bindings = HashMap::from([("T".to_string(), Type::Int)]);
    let resolved = substitute_structural_type(&template, &bindings, "models", &context);
    let Type::Class {
        type_args, fields, ..
    } = resolved
    else {
        panic!("resolved nested type must stay nominal");
    };
    assert_eq!(type_args, vec![Type::Str]);
    assert_eq!(fields[0].1, Type::Str);

    let missing = Type::Class {
        identity: Some("missing.Inner".to_string()),
        type_args: vec![Type::TypeVar("T".to_string())],
        name: "Inner".to_string(),
        fields: vec![("value".to_string(), Type::TypeVar("T".to_string()))],
        methods: Vec::new(),
        parent_class: None,
    };
    let unresolved = substitute_structural_type(&missing, &bindings, "models", &context);
    let Type::Class {
        type_args, fields, ..
    } = unresolved
    else {
        panic!("unresolved nested type must stay nominal");
    };
    assert_eq!(type_args, vec![Type::Int]);
    assert_eq!(fields[0].1, Type::TypeVar("T".to_string()));
}

#[test]
fn imported_outer_fields_resolve_nested_scopes_in_the_declaring_module() {
    let mut model_inner = class(
        "Inner",
        vec![("value".to_string(), Type::TypeVar("T".to_string()))],
    );
    model_inner.type_params = vec!["T".to_string()];
    let mut model_outer = class(
        "Outer",
        vec![(
            "inner".to_string(),
            Type::Class {
                identity: None,
                type_args: vec![Type::Str],
                name: "Inner".to_string(),
                fields: vec![("value".to_string(), Type::TypeVar("T".to_string()))],
                methods: Vec::new(),
                parent_class: None,
            },
        )],
    );
    model_outer.identity = Some("models.Outer".to_string());
    model_outer.type_params = vec!["T".to_string()];
    let models = HirModule {
        functions: Vec::new(),
        classes: vec![model_inner, model_outer],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };

    let mut consumer_inner = class(
        "Inner",
        vec![("value".to_string(), Type::TypeVar("U".to_string()))],
    );
    consumer_inner.type_params = vec!["U".to_string()];
    let main = HirModule {
        functions: Vec::new(),
        classes: vec![consumer_inner],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };
    let modules = [("models", &models), ("main", &main)];
    let context = IdentityContext { modules: &modules };
    let concrete_outer = Type::Class {
        identity: Some("models.Outer".to_string()),
        type_args: vec![Type::Int],
        name: "Outer".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    };

    let actual = compile_type(&concrete_outer, "main", &context, &mut Vec::new()).static_value();
    let inner = identity::nominal_record(
        "models.Inner",
        &[identity::primitive("str")],
        &[NominalField {
            name: "value",
            identity: identity::primitive("str"),
            required: true,
            default_identity: None,
        }],
        identity::metadata(&[]),
    );
    let expected = identity::nominal_record(
        "models.Outer",
        &[identity::primitive("int")],
        &[NominalField {
            name: "inner",
            identity: inner,
            required: true,
            default_identity: None,
        }],
        identity::metadata(&[]),
    );

    assert_eq!(actual, Some(expected));
}

#[test]
fn generic_union_identity_matches_reordered_collapsed_and_nested_substitutions() {
    let modules = Vec::new();
    let context = IdentityContext { modules: &modules };
    let template = sifr_type_system::make_union(vec![Type::Str, Type::TypeVar("T".to_string())]);
    assert_eq!(
        compile_type(&template, "main", &context, &mut Vec::new()).expression(),
        format!(
            "{STRUCTURAL}::union(&[{STRUCTURAL}::ShapeIdentity::from_bytes({:?}), <T as {STRUCTURAL}::StructuralType>::shape_identity()])",
            identity::primitive("str").as_bytes()
        )
    );

    let reordered = substitute_structural_type(
        &template,
        &HashMap::from([("T".to_string(), Type::Int)]),
        "main",
        &context,
    );
    let reordered_identity =
        compile_type(&reordered, "main", &context, &mut Vec::new()).static_value();
    assert_eq!(
        reordered_identity,
        Some(identity::union(&[
            identity::primitive("str"),
            identity::primitive("int"),
        ]))
    );

    let collapsed = substitute_structural_type(
        &template,
        &HashMap::from([("T".to_string(), Type::Str)]),
        "main",
        &context,
    );
    let collapsed_identity =
        compile_type(&collapsed, "main", &context, &mut Vec::new()).static_value();
    assert_eq!(collapsed, Type::Str);
    assert_eq!(collapsed_identity, Some(identity::primitive("str")));

    let nested_argument = sifr_type_system::make_union(vec![Type::Int, Type::Bool]);
    let nested = substitute_structural_type(
        &template,
        &HashMap::from([("T".to_string(), nested_argument)]),
        "main",
        &context,
    );
    let nested_identity = compile_type(&nested, "main", &context, &mut Vec::new()).static_value();
    assert_eq!(
        nested_identity,
        Some(identity::union(&[
            identity::primitive("str"),
            identity::primitive("bool"),
            identity::primitive("int"),
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
        "main",
        &context,
    );
    let nested_optional_identity =
        compile_type(&nested_optional, "main", &context, &mut Vec::new()).static_value();
    assert_eq!(
        nested_optional_identity,
        Some(identity::unary_container(
            "optional",
            identity::primitive("str")
        ))
    );
}
