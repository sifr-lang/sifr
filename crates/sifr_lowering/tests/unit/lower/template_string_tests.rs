use crate::{ExternalDefs, HirExpr, HirStmt, LoweringOptions};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::Type;

fn lower(source: &str) -> Result<crate::HirModule, Vec<crate::HirDiagnostic>> {
    let parsed = parse_module(source).expect("source should parse");
    crate::lower_module_with_externals_name_and_options(
        "main",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            source_text: Some(source.to_string()),
            ..LoweringOptions::default()
        },
    )
    .map(|result| result.module)
}

fn returned_expression<'a>(module: &'a crate::HirModule, function: &str) -> &'a HirExpr {
    let function = module
        .functions
        .iter()
        .find(|candidate| candidate.name == function)
        .expect("function should exist");
    let HirStmt::Return { value: Some(value) } = &function.body[0] else {
        panic!("expected returned expression");
    };
    value
}

#[test]
fn template_lowering_preserves_typed_holes_source_and_evaluation_order() {
    let source = "def consume(value: Template) -> int:\n    return 1\n\ndef main(user_id: int, name: str) -> int:\n    return consume(t\"SELECT {user_id} / {name!r:>8}\")\n";
    let module = lower(source).expect("template processor call should lower");
    let HirExpr::Call { args, .. } = returned_expression(&module, "main") else {
        panic!("expected processor call");
    };
    let HirExpr::TemplateString(template) = &args[0] else {
        panic!("expected retained template");
    };
    assert_eq!(template.ty, Type::Template(vec![Type::Int, Type::Str]));
    assert_eq!(template.virtual_source, "SELECT \u{fffc} / \u{fffc}");
    assert_eq!(template.segments.len(), 3);
    assert_eq!(template.interpolations.len(), 2);
    assert_eq!(template.interpolations[0].expression_source, "user_id");
    assert_eq!(template.interpolations[1].expression_source, "name");
    assert!(!template.interpolations[0].clone_from_borrow);
    assert!(template.interpolations[1].clone_from_borrow);
    assert_eq!(template.interpolations[1].conversion, Some('r'));
    assert!(template.interpolations[1].format_spec.is_some());
    assert!(template.interpolations.windows(2).all(|pair| {
        pair[0].source_range.end() <= pair[1].source_range.start()
            && pair[0].virtual_range.end() <= pair[1].virtual_range.start()
    }));
}

#[test]
fn nested_format_spec_holes_are_retained_in_eager_order() {
    let source = "def make(amount: float, precision: int) -> Template:\n    return t\"value={amount:.{precision}f}\"\n";
    let module = lower(source).expect("nested format spec should lower");
    let HirExpr::TemplateString(template) = returned_expression(&module, "make") else {
        panic!("expected template");
    };
    let spec = template.interpolations[0]
        .format_spec
        .as_ref()
        .expect("format spec should be retained");
    assert_eq!(spec.parts.len(), 3);
    assert!(matches!(
        &spec.parts[1],
        sifr_ir::HirTemplateFormatSpecPart::Interpolation { value, .. }
            if matches!(value.as_ref(), HirExpr::Name { name, .. } if name == "precision")
    ));
}

#[test]
fn templates_do_not_support_value_equality() {
    let source = "def invalid() -> bool:\n    return t\"left\" == t\"left\"\n";
    let diagnostics = lower(source).expect_err("template equality must fail");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && diagnostic.message
                == "cannot compare values without structural equality 'Template' and 'Template' with =="
    }));
}
