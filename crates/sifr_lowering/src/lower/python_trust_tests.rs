use crate::{ExternalDefs, HirDiagnostic, LoweringOptions, PythonTrustPolicy};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn python_externals() -> ExternalDefs {
    let object_ty = Type::Class {
        identity: None,
        name: "Object".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    };
    let error_ty = Type::Class {
        identity: None,
        name: "PythonError".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    };
    let mut functions = HashMap::new();
    functions.insert(
        "import_module".to_string(),
        FunctionType::all_borrow(
            vec![("name".to_string(), Type::Str)],
            Type::Result(Box::new(object_ty.clone()), Box::new(error_ty.clone())),
        ),
    );
    let mut classes = HashMap::new();
    classes.insert("Object".to_string(), object_ty);
    classes.insert("PythonError".to_string(), error_ty);

    let mut externals = ExternalDefs::default();
    externals
        .functions
        .insert("sifr.python".to_string(), functions);
    externals.classes.insert("sifr.python".to_string(), classes);
    externals.error_types.insert("PythonError".to_string());
    externals
}

fn lower_errors(source: &str, policy: Option<PythonTrustPolicy>) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    let result = crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &python_externals(),
        LoweringOptions {
            python_trust_policy: policy,
            ..LoweringOptions::default()
        },
    );
    match result {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

fn source_for_import(argument: &str, decorator: &str) -> String {
    format!(
        "from sifr.python import Object, PythonError, import_module\n\n{decorator}def load(name: str) -> Result[Object, PythonError]:\n    try:\n        obj: Object = import_module({argument})\n        return obj\n    except PythonError as e:\n        raise e\n"
    )
}

#[test]
fn static_python_import_literal_uses_package_trust_policy() {
    let source = source_for_import("\"json\"", "");
    let errors = lower_errors(
        &source,
        Some(PythonTrustPolicy {
            required_import_roots: vec!["math".to_string()],
            trusted_import_roots: vec!["math".to_string()],
        }),
    );

    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED)));
}

#[test]
fn static_python_import_literal_without_policy_is_rejected() {
    let source = source_for_import("\"math\"", "");
    let errors = lower_errors(&source, None);

    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED)));
}

#[test]
fn static_python_import_literal_accepts_root_wildcard_policy() {
    let source = source_for_import("\"json\"", "");
    let parsed = parse_module(&source).expect("source should parse");
    crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &python_externals(),
        LoweringOptions {
            python_trust_policy: Some(PythonTrustPolicy {
                required_import_roots: vec!["*".to_string()],
                trusted_import_roots: vec!["*".to_string()],
            }),
            ..LoweringOptions::default()
        },
    )
    .expect("root wildcard policy should allow literal import");
}

#[test]
fn dynamic_python_import_requires_trust_decorator() {
    let errors = lower_errors(&source_for_import("name", ""), None);
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PYTRUST_DYNAMIC_IMPORT_REQUIRES_TRUST)
    }));

    let source = source_for_import("name", "@trust_python_dynamic\n");
    let parsed = parse_module(&source).expect("source should parse");
    crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &python_externals(),
        LoweringOptions::default(),
    )
    .expect("trusted dynamic import should lower");
}
