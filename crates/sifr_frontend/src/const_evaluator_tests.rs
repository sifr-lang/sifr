use super::*;
use sifr_lowering::lower_module;
use sifr_syntax::parse_module_suite;

fn lower(source: &str) -> sifr_lowering::LoweringResult {
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    lower_module(&parsed).expect("fixture lowers")
}

#[test]
fn evaluates_bounded_pure_const_function_deterministically() {
    let lowered = lower(
        "@const_eval\ndef describe(values: list[int]) -> int:\n    total: int = 0\n    for value in values:\n        total = total + value\n    return total\n",
    );
    let args = vec![ConstValue::List(vec![
        ConstValue::Integer(BigInt::from(2)),
        ConstValue::Integer(BigInt::from(5)),
    ])];
    let first = DeterministicConstEvaluator::new(&lowered.module)
        .evaluate_function("describe", args.clone())
        .expect("const evaluation succeeds");
    let second = DeterministicConstEvaluator::new(&lowered.module)
        .evaluate_function("describe", args)
        .expect("const evaluation succeeds");
    assert_eq!(first, ConstValue::Integer(BigInt::from(7)));
    assert_eq!(first, second);
}

#[test]
fn fails_closed_on_unbounded_const_evaluation() {
    let lowered =
        lower("@const_eval\ndef forever() -> int:\n    while True:\n        pass\n    return 0\n");
    let error = DeterministicConstEvaluator::with_limits(&lowered.module, 20, 4, 8)
        .evaluate_function("forever", Vec::new())
        .expect_err("step budget is enforced");
    assert_eq!(error.kind, ConstEvalErrorKind::StepLimit);
}

#[test]
fn augmented_assignment_preserves_floor_division_and_modulo_semantics() {
    let lowered = lower(
        "@const_eval\ndef arithmetic() -> tuple[int, int]:\n    quotient: int = -7\n    quotient += 0\n    quotient //= 3\n    remainder: int = -7\n    remainder %= 3\n    return (quotient, remainder)\n",
    );
    let value = DeterministicConstEvaluator::new(&lowered.module)
        .evaluate_function("arithmetic", Vec::new())
        .expect("const evaluation succeeds");
    assert_eq!(
        value,
        ConstValue::Tuple(vec![
            ConstValue::Integer(BigInt::from(-3)),
            ConstValue::Integer(BigInt::from(2)),
        ])
    );
}

#[test]
fn preserves_bytes_as_the_closed_bytes_const_variant() {
    let lowered = lower("@const_eval\ndef payload() -> bytes:\n    return b\"typed\"\n");
    let value = DeterministicConstEvaluator::new(&lowered.module)
        .evaluate_function("payload", Vec::new())
        .expect("byte const evaluation succeeds");
    assert_eq!(value, ConstValue::Bytes(b"typed".to_vec()));
}

#[test]
fn evaluates_primitive_isinstance_for_typed_union_normalization() {
    let lowered = lower(
        "@const_eval\ndef kind(value: int | str) -> str:\n    if isinstance(value, str):\n        return \"text\"\n    return \"integer\"\n",
    );
    let text = DeterministicConstEvaluator::new(&lowered.module)
        .evaluate_function("kind", vec![ConstValue::String("value".to_string())])
        .expect("string branch evaluates");
    let integer = DeterministicConstEvaluator::new(&lowered.module)
        .evaluate_function("kind", vec![ConstValue::Integer(BigInt::ONE)])
        .expect("integer branch evaluates");
    assert_eq!(text, ConstValue::String("text".to_string()));
    assert_eq!(integer, ConstValue::String("integer".to_string()));
}

#[test]
fn iterates_closed_record_keys_in_canonical_order() {
    let lowered = lower(
        "@const_eval\ndef keys(value: dict[str, str]) -> list[str]:\n    output: list[str] = []\n    for key in value:\n        output.append(key)\n    return output\n",
    );
    let value = DeterministicConstEvaluator::new(&lowered.module)
        .evaluate_function(
            "keys",
            vec![ConstValue::Record(BTreeMap::from([
                ("zeta".to_string(), ConstValue::String("last".to_string())),
                ("alpha".to_string(), ConstValue::String("first".to_string())),
            ]))],
        )
        .expect("record iteration succeeds");
    assert_eq!(
        value,
        ConstValue::List(vec![
            ConstValue::String("alpha".to_string()),
            ConstValue::String("zeta".to_string()),
        ])
    );
}

#[test]
fn rejects_runtime_function_as_const_entrypoint() {
    let lowered = lower("def runtime() -> int:\n    return 1\n");
    let error = DeterministicConstEvaluator::new(&lowered.module)
        .evaluate_function("runtime", Vec::new())
        .expect_err("runtime function is rejected");
    assert_eq!(error.kind, ConstEvalErrorKind::FunctionNotConst);
}
