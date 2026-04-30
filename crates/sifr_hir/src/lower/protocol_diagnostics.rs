use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(super) fn bound_not_satisfied(
    ctx: &mut LowerCtx,
    actual: &str,
    protocol: &str,
    type_param: &str,
) {
    ctx.error_with_code(
        DiagnosticCode::PROTO_BOUND_NOT_SATISFIED,
        format!(
            "type '{actual}' does not implement protocol '{protocol}' required by type parameter '{type_param}'"
        ),
    );
}

#[cfg(test)]
mod tests {
    use crate::{lower_module, LoweringError};
    use sifr_diagnostics::DiagnosticCode;
    use sifr_python_parser::parse_module;

    fn lower_errors(source: &str) -> Vec<LoweringError> {
        let parsed = parse_module(source).expect("parse failed");
        match lower_module(parsed.suite()) {
            Ok(_) => panic!("expected lowering error"),
            Err(errors) => errors,
        }
    }

    #[test]
    fn concrete_type_missing_protocol_bound_has_proto_code() {
        let errors = lower_errors(
            "class Comparable(Protocol):\n    def __lt__(self, other: Self) -> bool:\n        pass\n\nclass Blob:\n    data: int\n\ndef choose[T: Comparable](x: T) -> T:\n    return x\n\ndef main():\n    out: Blob = choose(Blob(1))\n    print(out.data)\n",
        );

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'Blob' does not implement protocol 'Comparable' required by type parameter 'T'"
                && error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
        }));
    }

    #[test]
    fn forwarded_typevar_missing_protocol_bound_has_proto_code() {
        let errors = lower_errors(
            "class Readable(Protocol):\n    def read(self) -> str:\n        pass\n\nclass Closable(Protocol):\n    def close(self) -> None:\n        pass\n\ndef take_readable[T: Readable](x: T) -> T:\n    return x\n\ndef relay_bad[U: Closable](x: U) -> U:\n    return take_readable(x)\n",
        );

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'U' does not implement protocol 'Readable' required by type parameter 'T'"
                && error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
        }));
    }
}
