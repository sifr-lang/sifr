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

pub(super) fn context_manager_missing(ctx: &mut LowerCtx, type_name: &str) {
    ctx.error_with_code(
        DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING,
        format!(
            "type '{type_name}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"
        ),
    );
}

pub(super) fn context_manager_incomplete(ctx: &mut LowerCtx, type_name: &str) {
    ctx.error_with_code(
        DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING,
        format!(
            "type '{type_name}' used in 'with' statement must implement both __enter__ and __exit__ methods"
        ),
    );
}

pub(super) fn iterator_invalid_return_signature(
    ctx: &mut LowerCtx,
    type_name: &str,
    expected: &str,
) {
    ctx.error_with_code(
        DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
        format!("class '{type_name}' must return {expected}"),
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

    #[test]
    fn missing_context_manager_has_proto_code() {
        let errors = lower_errors(
            "class PlainClass:\n    value: int\n\ndef main():\n    with PlainClass(42) as p:\n        print(p.value)\n",
        );

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'PlainClass' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"
                && error.code == Some(DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING)
        }));
    }

    #[test]
    fn incomplete_context_manager_has_proto_code() {
        let errors = lower_errors(
            "class HalfContext:\n    def __enter__(self) -> HalfContext:\n        return self\n\ndef main():\n    with HalfContext() as ctx:\n        print(ctx)\n",
        );

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'HalfContext' used in 'with' statement must implement both __enter__ and __exit__ methods"
                && error.code == Some(DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING)
        }));
    }

    #[test]
    fn non_class_context_manager_has_proto_code() {
        let errors = lower_errors("def main():\n    with 1 as value:\n        print(value)\n");

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'int' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"
                && error.code == Some(DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING)
        }));
    }

    #[test]
    fn invalid_iter_signature_has_proto_code() {
        let errors =
            lower_errors("class BadIter:\n    def __iter__(self) -> int:\n        return 1\n");

        assert!(errors.iter().any(|error| {
            error.message == "class 'BadIter.__iter__' must return 'Iterator[T]' or 'Iterable[T]'"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
        }));
    }

    #[test]
    fn invalid_next_signature_has_proto_code() {
        let errors = lower_errors(
            "class BadNext:\n    def __iter__(self) -> Iterator[int]:\n        return iter([1])\n\n    def __next__(self) -> int:\n        return 1\n",
        );

        assert!(errors.iter().any(|error| {
            error.message == "class 'BadNext.__next__' must return 'T | None'"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
        }));
    }

    #[test]
    fn invalid_reversed_signature_has_proto_code() {
        let errors = lower_errors(
            "class BadReversed:\n    def __iter__(self) -> Iterator[int]:\n        return iter([1, 2])\n\n    def __reversed__(self) -> int:\n        return 0\n",
        );

        assert!(errors.iter().any(|error| {
            error.message
                == "class 'BadReversed.__reversed__' must return 'Iterator[T]' or 'Iterable[T]'"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
        }));
    }
}
