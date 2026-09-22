use super::*;

fn source(op: &str, rhs: &str, in_try: bool) -> String {
    let statement = format!("self.value {op}= {rhs}");
    let body = if in_try {
        format!("        try:\n            {statement}\n        except Error:\n            pass")
    } else {
        format!("        {statement}")
    };
    format!(
        "class Counter:\n    value: int\n    def __init__(self):\n        self.value = -17\n    def update(mut self, divisor: int):\n{body}\n"
    )
}

#[test]
fn integer_field_augassign_rejects_unhandled_failure_inside_and_outside_try() {
    for in_try in [false, true] {
        for (op, rhs) in [
            ("/", "2"),
            ("/", "divisor"),
            ("%", "0"),
            ("%", "divisor"),
            ("//", "0"),
            ("//", "divisor"),
        ] {
            let errors = lower_source(&source(op, rhs, in_try))
                .expect_err("field assignment must obey the same integer contract as a local");
            assert!(
                errors.iter().any(|error| error.code
                    == Some(DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING)),
                "{errors:?}"
            );
        }
    }
}

#[test]
fn integer_field_augassign_accepts_proven_nonzero_floor_operations() {
    for in_try in [false, true] {
        for op in ["%", "//"] {
            for rhs in ["3", "-3"] {
                lower_source(&source(op, rhs, in_try)).expect("nonzero field operation lowers");
            }
        }
    }
}

#[test]
fn integer_field_augassign_aliases_preserve_failure_contract() {
    for in_try in [false, true] {
        for (op, rhs) in [("/", "2"), ("%", "divisor"), ("//", "divisor")] {
            let source = format!(
                "type Count = int\n{}",
                source(op, rhs, in_try).replace(": int", ": Count")
            );
            let errors = lower_source(&source).expect_err("aliases cannot bypass integer safety");
            assert!(
                errors.iter().any(|error| error.code
                    == Some(DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING)),
                "{errors:?}"
            );
        }
        let source = format!(
            "type Count = int\n{}",
            source("%", "3", in_try).replace(": int", ": Count")
        );
        lower_source(&source).expect("safe aliased field operation lowers");
    }
}
