use super::*;

#[test]
fn accepts_the_codegen_name_form_for_const_isinstance_targets() {
    let parsed = sifr_syntax::parse_module_suite("", None).expect("fixture parses");
    let lowered = sifr_lowering::lower_module(&parsed).expect("fixture lowers");
    let mut evaluator = DeterministicConstEvaluator::new(&lowered.module);
    let value = evaluator
        .eval_isinstance(
            &[
                HirExpr::StringLiteral("value".to_string()),
                HirExpr::Name {
                    name: "str".to_string(),
                    binding_id: None,
                    ty: Type::Str,
                },
            ],
            &mut Environment::new(),
            0,
        )
        .expect("the codegen HIR target form must be accepted");

    assert_eq!(value, ConstValue::Bool(true));
}
