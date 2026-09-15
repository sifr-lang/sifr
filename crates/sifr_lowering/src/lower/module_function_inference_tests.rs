use super::{LowerCtx, infer_unannotated_returns};
use crate::{HirDiagnostic, HirModule, lower_module};
use sifr_python_parser::parse_module;
use sifr_type_system::Type;

fn lower(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("valid test syntax");
    lower_module(parsed.suite()).map(|result| result.module)
}

#[test]
fn annotated_module_prepass_leaves_diagnostic_state_untouched() {
    // Invalid annotations still belong to normal declaration/body lowering;
    // a return-seeding pass with no consumers must not resolve them again.
    for source in [
        "def value(arg: Missing) -> Missing:\n    return arg\n",
        "value: Missing = 1\n",
        "",
    ] {
        let parsed = parse_module(source).expect("valid test syntax");
        let mut ctx = LowerCtx::new();
        let functions = format!("{:?}", ctx.functions);
        infer_unannotated_returns(parsed.suite(), &mut ctx);
        assert!(ctx.errors.is_empty());
        assert!(ctx.last_error_taint.is_none());
        assert_eq!(format!("{:?}", ctx.functions), functions);
    }
}

#[test]
fn annotated_outer_keeps_nested_return_inference_and_forward_calls() {
    let module = lower(
        "def outer(value: int) -> int:\n    result = helper(value)\n    def helper(value: int):\n        return value + 1\n    return result\n",
    )
    .expect("normal body lowering still infers nested helpers");
    assert_eq!(module.functions[0].return_type, Type::Int);
}

#[test]
fn mixed_top_level_annotations_keep_group_return_inference() {
    for source in [
        "def first():\n    return second()\ndef second() -> int:\n    return 7\n",
        "def second() -> int:\n    return 7\ndef first():\n    return second()\n",
        "def first():\n    return second()\ndef second():\n    return third()\ndef third() -> int:\n    return 7\n",
    ] {
        let module = lower(source).expect("mixed group must still infer forward returns");
        for function in module.functions {
            assert_eq!(function.return_type, Type::Int, "{}", function.name);
        }
    }
}

#[test]
fn annotated_modules_still_report_body_and_signature_errors() {
    for source in [
        "def value() -> int:\n    return \"wrong\"\n",
        "def value(arg: Missing) -> int:\n    return 1\n",
        "def value() -> int:\n    return missing\n",
        "def value() -> int:\n    def inner() -> int:\n        return \"wrong\"\n    return inner()\n",
    ] {
        let errors = lower(source).expect_err("normal lowering retains diagnostics");
        assert!(!errors.is_empty());
        assert!(errors.iter().all(|error| error.primary_range.is_some()));
    }
}
