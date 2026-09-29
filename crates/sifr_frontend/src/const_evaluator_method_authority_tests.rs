use super::*;
use sifr_ir::visit_hir_function_exprs_mut;
use sifr_lowering::lower_module;
use sifr_syntax::parse_module_suite;

fn lower(source: &str) -> HirModule {
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    lower_module(&parsed).expect("fixture lowers").module
}

fn integer(value: i64) -> ConstValue {
    ConstValue::Integer(BigInt::from(value))
}

#[test]
fn supported_methods_match_runtime_semantics() {
    let module = lower(
        "@const_eval\ndef sizes(text: str, data: bytes, values: list[int], pair: tuple[int, int], record: dict[str, int]) -> tuple[int, int, int, int, int]:\n    return (text.len(), data.len(), values.len(), pair.len(), record.len())\n\n@const_eval\ndef appended(own mut values: list[int]) -> list[int]:\n    values.append(7)\n    return values\n",
    );
    let result = DeterministicConstEvaluator::new(&module)
        .evaluate_function(
            "sizes",
            vec![
                ConstValue::String("aé🙂".to_string()),
                ConstValue::Bytes(vec![0, 255]),
                ConstValue::List(vec![integer(1), integer(2)]),
                ConstValue::Tuple(vec![integer(1), integer(2)]),
                ConstValue::Record(BTreeMap::from([("key".to_string(), integer(1))])),
            ],
        )
        .expect("the closed length subset evaluates");
    assert_eq!(
        result,
        ConstValue::Tuple(vec![
            integer(3),
            integer(2),
            integer(2),
            integer(2),
            integer(1)
        ])
    );
    let appended = DeterministicConstEvaluator::new(&module)
        .evaluate_function("appended", vec![ConstValue::List(vec![integer(2)])])
        .expect("local list append evaluates");
    assert_eq!(appended, ConstValue::List(vec![integer(2), integer(7)]));

    let limit = DeterministicConstEvaluator::with_limits(&module, 100, 4, 1)
        .evaluate_function("appended", vec![ConstValue::List(vec![integer(2)])])
        .expect_err("local append respects the collection limit");
    assert_eq!(limit.kind, ConstEvalErrorKind::CollectionLimit);

    let mismatch = DeterministicConstEvaluator::new(&module)
        .evaluate_function(
            "sizes",
            vec![
                ConstValue::List(vec![integer(1)]),
                ConstValue::Bytes(vec![]),
                ConstValue::List(vec![]),
                ConstValue::Tuple(vec![integer(1), integer(2)]),
                ConstValue::Record(BTreeMap::new()),
            ],
        )
        .expect_err("a value of another receiver type cannot reuse the method");
    assert_eq!(mismatch.kind, ConstEvalErrorKind::TypeMismatch);
}

#[test]
fn unsupported_methods_decline() {
    let module = lower(
        "@const_eval\ndef clear(mut values: list[int]) -> None:\n    values.clear()\n\n@const_eval\ndef upper(text: str) -> str:\n    return text.upper()\n\n@const_eval\ndef size(values: list[int]) -> int:\n    return values.len()\n",
    );
    for (name, argument) in [
        ("clear", ConstValue::List(vec![integer(1)])),
        ("upper", ConstValue::String("hello".to_string())),
    ] {
        let declined = DeterministicConstEvaluator::new(&module)
            .evaluate_function(name, vec![argument])
            .expect_err("effectful or unsupported method must decline");
        assert_eq!(declined.kind, ConstEvalErrorKind::UnsupportedExpression);
    }

    let mut nonbuiltin = module.clone();
    let function = nonbuiltin
        .functions
        .iter_mut()
        .find(|f| f.name == "size")
        .unwrap();
    visit_hir_function_exprs_mut(function, &mut |expr| {
        if let HirExpr::MethodCall { authority, .. } = expr {
            let MethodAuthority::BuiltinIntrinsic { declaration } = authority.clone() else {
                panic!("fixture method must be builtin");
            };
            *authority = MethodAuthority::LocalNominal { declaration };
        }
    });
    let declined = DeterministicConstEvaluator::new(&nonbuiltin)
        .evaluate_function("size", vec![ConstValue::List(vec![integer(1)])])
        .expect_err("a nominal method named len cannot use builtin semantics");
    assert_eq!(declined.kind, ConstEvalErrorKind::UnsupportedExpression);

    let mut forged = module;
    let function = forged
        .functions
        .iter_mut()
        .find(|f| f.name == "size")
        .unwrap();
    visit_hir_function_exprs_mut(function, &mut |expr| {
        if let HirExpr::MethodCall {
            authority: MethodAuthority::BuiltinIntrinsic { declaration },
            ..
        } = expr
        {
            declaration.owner = Some("str".to_string());
        }
    });
    let declined = DeterministicConstEvaluator::new(&forged)
        .evaluate_function("size", vec![ConstValue::List(vec![integer(1)])])
        .expect_err("a mismatched builtin declaration cannot use list semantics");
    assert_eq!(declined.kind, ConstEvalErrorKind::UnsupportedExpression);
}
