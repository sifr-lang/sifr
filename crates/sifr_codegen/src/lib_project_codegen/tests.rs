use super::*;
use sifr_ir::{HirClass, HirClassKind, HirExceptHandler, HirFunction, HirImport, MethodKind};

fn empty_function(name: &str, return_type: sifr_type_system::Type) -> HirFunction {
    HirFunction {
        name: name.to_string(),
        params: Vec::new(),
        return_type,
        body: vec![sifr_ir::HirStmt::Return {
            value: Some(sifr_ir::HirExpr::IntLiteral(1)),
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    }
}

fn module_with(functions: Vec<HirFunction>, imports: Vec<HirImport>) -> HirModule {
    HirModule {
        functions,
        classes: Vec::new(),
        imports,
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}

fn error_type(name: &str) -> sifr_type_system::Type {
    sifr_type_system::Type::Class {
        identity: Some(format!("errors.{name}")),
        type_args: Vec::new(),
        name: name.to_string(),
        fields: vec![("message".to_string(), sifr_type_system::Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}

fn error_class(name: &str) -> HirClass {
    HirClass {
        name: name.to_string(),
        identity: Some(format!("errors.{name}")),
        fields: vec![("message".to_string(), sifr_type_system::Type::Str)],
        field_defaults: Vec::new(),
        field_default_identities: Vec::new(),
        declaration_metadata: Vec::new(),
        methods: Vec::new(),
        is_hashable: true,
        is_error_type: true,
        kind: HirClassKind::Regular,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: Some("Error".to_string()),
        parent_type: None,
        type_params: Vec::new(),
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    }
}

#[test]
fn project_unions_have_one_crate_root_definition() {
    let union = sifr_type_system::Type::Union(vec![
        sifr_type_system::Type::Int,
        sifr_type_system::Type::Str,
    ]);
    let enum_name = union.union_enum_name();
    let provider = module_with(vec![empty_function("produce", union.clone())], Vec::new());
    let consumer = module_with(
        vec![empty_function("marker", sifr_type_system::Type::Int)],
        vec![HirImport {
            module: "provider".to_string(),
            names: vec!["produce".to_string()],
            aliases: Vec::new(),
        }],
    );

    let generated = generate_rust_multi_with_metadata(
        &[("main", &consumer), ("provider", &provider)],
        &StdlibCode::default(),
    );
    let provider_source = &generated.rust_files["provider"];
    assert!(
        generated
            .project_union_prelude
            .contains(&format!("pub enum {enum_name}")),
        "{}",
        generated.project_union_prelude
    );
    assert_eq!(
        generated
            .project_union_prelude
            .matches(&format!("enum {enum_name}"))
            .count(),
        1
    );
    assert!(
        provider_source.contains(&format!("use crate::{enum_name};")),
        "{provider_source}"
    );
    assert!(!generated.rust_files["main"].contains(&format!("enum {enum_name}")));
}

#[test]
fn main_owned_union_is_imported_from_the_crate_root() {
    let union = sifr_type_system::Type::Union(vec![
        sifr_type_system::Type::Int,
        sifr_type_system::Type::Str,
    ]);
    let enum_name = union.union_enum_name();
    let owner = module_with(vec![empty_function("produce", union.clone())], Vec::new());
    let consumer = module_with(vec![empty_function("relay", union.clone())], Vec::new());

    let generated = generate_rust_multi_with_metadata(
        &[("main", &owner), ("support", &consumer)],
        &StdlibCode::default(),
    );

    assert!(
        generated
            .project_union_prelude
            .contains(&format!("enum {enum_name}"))
    );
    assert!(
        generated.rust_files["support"].contains(&format!("use crate::{enum_name};")),
        "{}",
        generated.rust_files["support"]
    );
}

#[test]
fn dotted_union_user_imports_the_crate_root_definition() {
    let union = sifr_type_system::Type::Union(vec![
        sifr_type_system::Type::Int,
        sifr_type_system::Type::Str,
    ]);
    let enum_name = union.union_enum_name();
    let owner = module_with(vec![empty_function("produce", union.clone())], Vec::new());
    let consumer = module_with(
        vec![empty_function("marker", sifr_type_system::Type::Int)],
        vec![HirImport {
            module: "pkg.errors".to_string(),
            names: vec!["produce".to_string()],
            aliases: Vec::new(),
        }],
    );

    let generated = generate_rust_multi_with_metadata(
        &[("pkg.errors", &owner), ("main", &consumer)],
        &StdlibCode::default(),
    );

    assert!(
        generated.rust_files["pkg.errors"].contains(&format!("use crate::{enum_name};")),
        "{}",
        generated.rust_files["pkg.errors"]
    );
}

#[test]
fn root_prelude_combines_try_conversions_with_ordinary_union_traits() {
    let first = error_type("FirstError");
    let second = error_type("SecondError");
    let union = sifr_type_system::Type::Union(vec![first.clone(), second.clone()]);
    let enum_name = union.union_enum_name();
    let mut try_function = empty_function("guarded", sifr_type_system::Type::None);
    try_function.body = vec![sifr_ir::HirStmt::TryExcept {
        body: vec![sifr_ir::HirStmt::Pass],
        handlers: vec![HirExceptHandler {
            error_type: Some("FirstError".to_string()),
            error_resolved_type: Some(first.clone()),
            name: None,
            body: vec![sifr_ir::HirStmt::Pass],
        }],
        body_error_types: vec![first, second],
    }];
    let mut owner = module_with(vec![try_function], Vec::new());
    owner.classes = vec![error_class("FirstError"), error_class("SecondError")];
    let consumer = module_with(vec![empty_function("ordinary", union)], Vec::new());

    let generated = generate_rust_multi_with_metadata(
        &[("errors", &owner), ("main", &consumer)],
        &StdlibCode::default(),
    );
    let prelude = &generated.project_union_prelude;

    assert!(
        prelude.contains("#[derive(Debug, Clone, PartialEq, Eq, Hash)]"),
        "{prelude}"
    );
    assert!(
        prelude.contains("impl From<crate::errors::FirstError>")
            && prelude.contains(&format!("for {enum_name}")),
        "{prelude}"
    );
    assert!(
        prelude.contains("impl From<crate::errors::SecondError>"),
        "{prelude}"
    );
}

#[test]
fn root_prelude_uses_crate_rooted_nominal_payload_paths() {
    let first = error_type("FirstError");
    let second = error_type("SecondError");
    let union = sifr_type_system::Type::Union(vec![first, second]);
    let mut errors = module_with(vec![empty_function("produce", union.clone())], Vec::new());
    errors.classes = vec![error_class("FirstError"), error_class("SecondError")];
    let unrelated = module_with(vec![empty_function("consume", union)], Vec::new());

    let generated = generate_rust_multi_with_metadata(
        &[("app", &unrelated), ("errors", &errors)],
        &StdlibCode::default(),
    );

    assert!(
        generated
            .project_union_prelude
            .contains("crate::errors::FirstError"),
        "{}",
        generated.project_union_prelude
    );
    assert!(
        generated
            .project_union_prelude
            .contains("crate::errors::SecondError"),
        "{}",
        generated.project_union_prelude
    );
}

#[test]
fn root_union_plan_distinguishes_non_class_nominal_identities() {
    let nominal_pairs = [
        (
            sifr_type_system::Type::Newtype {
                identity: Some("left.Token".to_string()),
                name: "Token".to_string(),
                inner: Box::new(sifr_type_system::Type::Int),
            },
            sifr_type_system::Type::Newtype {
                identity: Some("right.Token".to_string()),
                name: "Token".to_string(),
                inner: Box::new(sifr_type_system::Type::Int),
            },
        ),
        (
            sifr_type_system::Type::Enum {
                identity: Some("left.Status".to_string()),
                name: "Status".to_string(),
                variants: vec![("READY".to_string(), Some(1))],
            },
            sifr_type_system::Type::Enum {
                identity: Some("right.Status".to_string()),
                name: "Status".to_string(),
                variants: vec![("READY".to_string(), Some(1))],
            },
        ),
        (
            sifr_type_system::Type::Protocol {
                identity: Some("left.Readable".to_string()),
                name: "Readable".to_string(),
                methods: Vec::new(),
            },
            sifr_type_system::Type::Protocol {
                identity: Some("right.Readable".to_string()),
                name: "Readable".to_string(),
                methods: Vec::new(),
            },
        ),
    ];

    for (left, right) in nominal_pairs {
        let left = module_with(
            vec![empty_function(
                "left",
                sifr_type_system::Type::Union(vec![sifr_type_system::Type::Int, left]),
            )],
            Vec::new(),
        );
        let right = module_with(
            vec![empty_function(
                "right",
                sifr_type_system::Type::Union(vec![sifr_type_system::Type::Int, right]),
            )],
            Vec::new(),
        );
        let usage = project_union_usage(
            &[("left", &left), ("right", &right)],
            &StdlibCode::default(),
            false,
        );

        assert_eq!(usage.unions.len(), 2, "{:?}", usage.unions);
    }
}
