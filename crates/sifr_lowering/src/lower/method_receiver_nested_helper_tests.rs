use crate::{lower_module, HirExpr, HirFunction, HirModule, HirStmt};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::ReceiverConvention;

fn lower(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .expect("source should lower")
        .module
}

fn method<'a>(module: &'a HirModule, class: &str, name: &str) -> &'a HirFunction {
    module
        .classes
        .iter()
        .find(|candidate| candidate.name == class)
        .and_then(|class| {
            class
                .methods
                .iter()
                .find(|candidate| candidate.name == name)
        })
        .expect("method should exist")
}

#[test]
fn nested_mutating_helper_requires_a_mutable_method_receiver() {
    let source = r#"
class Bucket:
    values: list[int]

    def update(self) -> None:
        def helper(values: list[int]) -> None:
            values.append(1)

        helper(self.values)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("nested mutable call must require mut self"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION)
            && error.message.contains("self")
    }));
}

#[test]
fn nested_helper_receiver_analysis_uses_checked_lexical_call_metadata() {
    let source = r#"
class Bucket:
    values: list[int]

    def update(mut self) -> None:
        def helper(values: list[int]) -> None:
            values.append(1)

        helper(self.values)

    def size(self) -> int:
        def helper(values: list[int]) -> int:
            return len(values)

        return helper(self.values)
"#;
    let module = lower(source);
    let update = method(&module, "Bucket", "update");
    let size = method(&module, "Bucket", "size");

    assert_eq!(update.receiver, Some(ReceiverConvention::MutableBorrow));
    assert_eq!(size.receiver, Some(ReceiverConvention::SharedBorrow));
    let HirStmt::Expr {
        expr: HirExpr::Call {
            mutable_arg_places, ..
        },
    } = &update.body[1]
    else {
        panic!("nested helper call should remain in the method body");
    };
    assert!(mutable_arg_places[0].is_some());
}
