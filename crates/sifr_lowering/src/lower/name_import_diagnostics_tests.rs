use std::collections::HashMap;

use crate::{
    lower_module, lower_module_sysroot_private_declaration_with_externals,
    lower_module_sysroot_public_stdlib_with_externals, lower_module_with_externals, ExternalDefs,
    HirDiagnostic,
};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_ir::HirStmt;
use sifr_python_parser::parse_module;
use sifr_type_system::{FunctionType, Type};

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("parse failed");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    }
}

fn range_for(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle should exist") as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

fn string_arg<'a>(error: &'a HirDiagnostic, name: &str) -> Option<&'a str> {
    match error.args.get(name) {
        Some(DiagnosticArg::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

#[test]
fn undefined_variable_has_name_code() {
    let source = "def main():\n    print(x)\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "undefined variable: 'x'"
            && error.code == Some(DiagnosticCode::NAME_UNDEFINED_VARIABLE)
            && error.primary_range == Some(range_for(source, "x"))
    }));
}

#[test]
fn undefined_function_has_name_code() {
    let source = "def main():\n    foo()\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "undefined function: 'foo'"
            && error.code == Some(DiagnosticCode::NAME_UNDEFINED_CALLABLE)
            && error.primary_range == Some(range_for(source, "foo"))
    }));
}

#[test]
fn missing_stdlib_member_has_name_code() {
    let source = "from local_math import nonexistent_func\n\ndef main():\n    pass\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .insert("local_math".to_string(), HashMap::new());
    let errors = match lower_module_with_externals(parsed.suite(), &externals) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.message == "module 'local_math' has no member 'nonexistent_func'"
            && error.code == Some(DiagnosticCode::NAME_MISSING_MODULE_MEMBER)
            && error.primary_range == Some(range_for(source, "nonexistent_func"))
    }));
}

#[test]
fn deferred_stdlib_module_has_import_code() {
    let source = "from sifr.contextvars import ContextVar\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "module 'sifr.contextvars' is intentionally deferred: context-local state is deferred; pass task state explicitly"
            && error.code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)
            && string_arg(error, "module") == Some("sifr.contextvars")
            && error.primary_range == Some(range_for(source, "from sifr.contextvars import ContextVar"))
    }));
}

#[test]
fn unknown_sifr_module_uses_generic_import_diagnostic() {
    let source = "from sifr.not_a_module import value\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "unknown import target: 'sifr.not_a_module'"
            && error.code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)
            && string_arg(error, "module") == Some("sifr.not_a_module")
            && error.primary_range == Some(range_for(source, "from sifr.not_a_module import value"))
    }));
}

#[test]
fn forbidden_intrinsic_import_has_import_code() {
    let source = "from _sifr.fs import read_text\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot import from '_sifr.fs' — private sysroot declarations can only be imported by public sysroot stdlib source"
            && error.code == Some(DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC)
            && error.primary_range == Some(range_for(source, "from _sifr.fs import read_text"))
    }));
}

#[test]
fn user_source_cannot_import_compiled_private_constant() {
    let source = "from _sifr.math import pi\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot import from '_sifr.math' — private sysroot declarations can only be imported by public sysroot stdlib source"
            && error.code == Some(DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC)
            && error.primary_range == Some(range_for(source, "from _sifr.math import pi"))
    }));
}

#[test]
fn public_sysroot_stdlib_source_can_import_private_declarations() {
    let source = "from _sifr.fs import read_text\n\ndef main():\n    pass\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals.functions.insert(
        "_sifr.fs".to_string(),
        HashMap::from([(
            "read_text".to_string(),
            FunctionType::all_borrow(
                vec![("path".to_string(), Type::Str)],
                Type::Result(Box::new(Type::Str), Box::new(Type::Any)),
            ),
        )]),
    );
    let result = lower_module_sysroot_public_stdlib_with_externals(parsed.suite(), &externals)
        .expect("public stdlib source should import private declarations");

    assert!(result
        .module
        .imports
        .iter()
        .any(|import| import.module == "_sifr.fs" && import.names == ["read_text".to_string()]));
}

#[test]
fn public_sysroot_stdlib_source_resolves_compiled_private_constants() {
    let source = "from _sifr.math import pi\n\ndef main() -> float:\n    return pi\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals.constants.insert(
        "_sifr.math".to_string(),
        HashMap::from([("pi".to_string(), Type::Float)]),
    );

    let result = lower_module_sysroot_public_stdlib_with_externals(parsed.suite(), &externals)
        .expect("public stdlib source should import compiled private constants");

    assert!(result
        .module
        .imports
        .iter()
        .any(|import| import.module == "_sifr.math" && import.names == ["pi".to_string()]));
}

#[test]
fn public_sysroot_stdlib_source_rejects_uncompiled_private_import_name() {
    let source =
        "from _sifr.hidden import missing_name\n\ndef main(data: bytes) -> bytes:\n    return missing_name(data)\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals.constants.insert(
        "_sifr.hidden".to_string(),
        HashMap::from([("__compiled_marker".to_string(), Type::Bool)]),
    );

    let errors = match lower_module_sysroot_public_stdlib_with_externals(parsed.suite(), &externals)
    {
        Ok(_) => panic!("public stdlib source must not synthesize missing private names"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::NAME_MISSING_MODULE_MEMBER)
            && error.message == "module '_sifr.hidden' has no member 'missing_name'"
            && error.primary_range == Some(range_for(source, "missing_name"))
    }));
}

#[test]
fn public_sysroot_stdlib_source_resolves_compiled_private_classes() {
    let source = "from _sifr.hidden import PrivateThing\n\ndef make() -> PrivateThing:\n    return PrivateThing(1)\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals.classes.insert(
        "_sifr.hidden".to_string(),
        HashMap::from([(
            "PrivateThing".to_string(),
            Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "PrivateThing".to_string(),
                fields: vec![("value".to_string(), Type::Int)],
                methods: Vec::new(),
                parent_class: None,
            },
        )]),
    );

    let result = lower_module_sysroot_public_stdlib_with_externals(parsed.suite(), &externals)
        .expect("public stdlib source should import compiled private classes");

    assert!(result.module.imports.iter().any(|import| {
        import.module == "_sifr.hidden" && import.names == ["PrivateThing".to_string()]
    }));
}

#[test]
fn attached_api_set_import_requires_the_stored_canonical_identity() {
    let source = "from declared import Api\n\nclass Child(Api):\n    pass\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals.classes.insert(
        "declared".to_string(),
        HashMap::from([(
            "Api".to_string(),
            Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "Api".to_string(),
                fields: Vec::new(),
                methods: Vec::new(),
                parent_class: None,
            },
        )]),
    );
    externals.attached_api_sets.insert(
        "declared".to_string(),
        HashMap::from([(
            "Api".to_string(),
            sifr_ir::AttachedApiSetDeclaration {
                identity: sifr_ir::AttachedApiSetIdentity {
                    module: "actual".to_string(),
                    symbol: "Api".to_string(),
                },
                range: ruff_text_size::TextRange::default(),
            },
        )]),
    );

    lower_module_with_externals(parsed.suite(), &externals)
        .expect("a mismatched stored identity must not erase the imported class");
}

#[test]
fn builtin_open_preserves_an_aliased_imported_text_handle_identity() {
    let source = "from sifr.io import TextFileHandle as Handle\n\ndef main():\n    handle: Handle = open(\"out.txt\", \"w\", encoding=\"utf-8\")\n    handle.close()\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals.classes.insert(
        "sifr.io".to_string(),
        HashMap::from([(
            "TextFileHandle".to_string(),
            Type::Class {
                identity: Some("sifr.io.TextFileHandle".to_string()),
                type_args: Vec::new(),
                name: "TextFileHandle".to_string(),
                fields: Vec::new(),
                methods: vec![(
                    "close".to_string(),
                    FunctionType::all_borrow(Vec::new(), Type::None)
                        .with_receiver(sifr_type_system::ReceiverConvention::MutableBorrow),
                )],
                parent_class: None,
            },
        )]),
    );

    lower_module_with_externals(parsed.suite(), &externals)
        .expect("builtin open should return the canonical imported TextFileHandle type");
}

#[test]
fn builtin_open_preserves_an_aliased_imported_binary_handle_identity() {
    let source = "from sifr.io import FileHandle as Handle\n\ndef main():\n    handle: Handle = open(\"out.bin\", \"rb\")\n    handle.close()\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals.classes.insert(
        "sifr.io".to_string(),
        HashMap::from([(
            "FileHandle".to_string(),
            Type::Class {
                identity: Some("sifr.io.FileHandle".to_string()),
                type_args: Vec::new(),
                name: "FileHandle".to_string(),
                fields: Vec::new(),
                methods: vec![(
                    "close".to_string(),
                    FunctionType::all_borrow(Vec::new(), Type::None)
                        .with_receiver(sifr_type_system::ReceiverConvention::MutableBorrow),
                )],
                parent_class: None,
            },
        )]),
    );

    lower_module_with_externals(parsed.suite(), &externals)
        .expect("builtin open should return the canonical imported FileHandle type");
}

#[test]
fn builtin_open_never_reuses_a_local_same_basename_handle() {
    for source in [
        "class TextFileHandle:\n    value: int\n\ndef main():\n    handle: TextFileHandle = open(\"out.txt\", \"w\", encoding=\"utf-8\")\n",
        "class FileHandle:\n    value: int\n\ndef main():\n    handle: FileHandle = open(\"out.bin\", \"rb\")\n",
    ] {
        let parsed = parse_module(source).expect("parse failed");
        let errors = match lower_module(parsed.suite()) {
            Ok(_) => panic!("local handle shadow should not match the canonical open result"),
            Err(errors) => errors,
        };

        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)),
            "expected canonical-identity type mismatch, got {errors:?}"
        );
    }
}

#[test]
fn builtin_open_inferred_bindings_keep_canonical_handle_identities() {
    let source = r#"class FileHandle:
    value: int

class TextFileHandle:
    value: int

def main() -> None:
    local_binary = FileHandle(1)
    local_text = TextFileHandle(2)
    try:
        binary = open("out.bin", "wb")
        text = open("out.txt", "w", encoding="utf-8")
        binary.close()
        text.close()
    except IOError as error:
        _ = error.message
"#;
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals.classes.insert(
        "sifr.io".to_string(),
        HashMap::from([
            (
                "FileHandle".to_string(),
                Type::Class {
                    identity: Some("sifr.io.FileHandle".to_string()),
                    type_args: Vec::new(),
                    name: "FileHandle".to_string(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: None,
                },
            ),
            (
                "TextFileHandle".to_string(),
                Type::Class {
                    identity: Some("sifr.io.TextFileHandle".to_string()),
                    type_args: Vec::new(),
                    name: "TextFileHandle".to_string(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: None,
                },
            ),
        ]),
    );
    let result = lower_module_with_externals(parsed.suite(), &externals)
        .expect("canonical handles should lower");
    let main = result
        .module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");
    let HirStmt::TryExcept { body, .. } = &main.body[2] else {
        panic!("main should retain the try body");
    };

    for (statement, expected_identity) in body
        .iter()
        .take(2)
        .zip(["sifr.io.FileHandle", "sifr.io.TextFileHandle"])
    {
        let HirStmt::Let { ty, value, .. } = statement else {
            panic!("open result should lower to a let binding");
        };
        assert!(matches!(
            ty.resolve_alias(),
            Type::Class {
                identity: Some(identity),
                ..
            } if identity == expected_identity
        ));
        assert!(matches!(
            value.ty().resolve_alias(),
            Type::Class {
                identity: Some(identity),
                ..
            } if identity == expected_identity
        ));
    }
}

#[test]
fn private_sysroot_declaration_source_cannot_import_private_declarations() {
    let source = "from _sifr.fs import read_text\n\ndef main():\n    pass\n";
    let parsed = parse_module(source).expect("parse failed");
    let errors = match lower_module_sysroot_private_declaration_with_externals(
        parsed.suite(),
        &ExternalDefs::default(),
    ) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot import from '_sifr.fs' — private sysroot declarations can only be imported by public sysroot stdlib source"
            && error.code == Some(DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC)
            && error.primary_range == Some(range_for(source, "from _sifr.fs import read_text"))
    }));
}

#[test]
fn unknown_module_import_has_import_code() {
    let source = "from missing_module import value\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "unknown import target: 'missing_module'"
            && error.code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)
            && string_arg(error, "module") == Some("missing_module")
            && error.primary_range == Some(range_for(source, "from missing_module import value"))
    }));
}

#[test]
fn bare_stdlib_import_from_has_targeted_import_code_and_args() {
    let source = "from math import sqrt\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "bare stdlib import 'math'; Sifr stdlib lives under 'sifr.*'"
            && error.code == Some(DiagnosticCode::IMPORT_BARE_STDLIB)
            && error.primary_range == Some(range_for(source, "math"))
            && string_arg(error, "bare_module") == Some("math")
            && string_arg(error, "suggested_module") == Some("sifr.math")
            && string_arg(error, "imported_names") == Some("sqrt")
            && error.help.as_deref() == Some("use 'from sifr.math import sqrt'")
    }));
}

#[test]
fn bare_stdlib_import_from_alias_preserves_imported_names_arg() {
    let source = "from math import sqrt as root\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::IMPORT_BARE_STDLIB)
            && string_arg(error, "bare_module") == Some("math")
            && string_arg(error, "suggested_module") == Some("sifr.math")
            && string_arg(error, "imported_names") == Some("sqrt as root")
            && error.help.as_deref() == Some("use 'from sifr.math import sqrt as root'")
    }));
}

#[test]
fn bare_stdlib_import_statement_has_targeted_import_code() {
    let source = "import math as m\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message == "bare stdlib import 'math'; Sifr stdlib lives under 'sifr.*'"
            && error.code == Some(DiagnosticCode::IMPORT_BARE_STDLIB)
            && error.primary_range == Some(range_for(source, "math"))
            && string_arg(error, "bare_module") == Some("math")
            && string_arg(error, "suggested_module") == Some("sifr.math")
            && string_arg(error, "imported_names") == Some("")
            && error.help.as_deref() == Some("use 'from sifr.math import <name>'")
    }));
}

#[test]
fn bare_stdlib_submodule_root_fallback_reports_unavailable_embedded_module() {
    let source = "from collections.abc import Iterable\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::IMPORT_BARE_STDLIB)
            && string_arg(error, "bare_module") == Some("collections.abc")
            && string_arg(error, "suggested_module") == Some("sifr.collections")
            && string_arg(error, "imported_names") == Some("Iterable")
            && error.help.as_deref()
                == Some(
                    "use 'from sifr.collections import Iterable'; no embedded sifr.collections.abc module exists",
                )
    }));
}

#[test]
fn unsupported_import_statement_has_import_code() {
    let source = "import local_math\n\ndef main():\n    pass\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "unsupported import form: import local_math; use 'from local_math import <name>'"
            && error.code == Some(DiagnosticCode::IMPORT_UNSUPPORTED_FORM)
            && error.primary_range == Some(range_for(source, "local_math"))
    }));
}

#[test]
fn private_import_member_has_import_code() {
    let source = "from local_math import _secret\n\ndef main():\n    pass\n";
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .insert("local_math".to_string(), HashMap::new());
    let errors = match lower_module_with_externals(parsed.suite(), &externals) {
        Ok(_) => panic!("expected lowering error"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.message == "cannot import private name '_secret' from module 'local_math'"
            && error.code == Some(DiagnosticCode::IMPORT_PRIVATE_MEMBER)
            && error.primary_range == Some(range_for(source, "_secret"))
    }));
}
