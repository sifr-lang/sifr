use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

use super::LowerCtx;

pub(in crate::lower) fn bound_not_satisfied(
    ctx: &mut LowerCtx,
    actual: &str,
    protocol: &str,
    type_param: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_BOUND_NOT_SATISFIED,
        format!(
            "type '{actual}' does not implement protocol '{protocol}' required by type parameter '{type_param}'"
        ),
        range,
    );
}

pub(in crate::lower) fn context_manager_missing(
    ctx: &mut LowerCtx,
    type_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING,
        format!(
            "type '{type_name}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"
        ),
        range,
    );
}

pub(in crate::lower) fn context_manager_incomplete(
    ctx: &mut LowerCtx,
    type_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING,
        format!(
            "type '{type_name}' used in 'with' statement must implement both __enter__ and __exit__ methods"
        ),
        range,
    );
}

pub(in crate::lower) fn iterator_invalid_return_signature(
    ctx: &mut LowerCtx,
    type_name: &str,
    expected: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
        format!("class '{type_name}' must return {expected}"),
        range,
    );
}

pub(in crate::lower) fn iterator_invalid_parameter_signature(
    ctx: &mut LowerCtx,
    type_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
        format!("class '{type_name}' must not declare parameters besides self"),
        range,
    );
}

pub(in crate::lower) fn iterator_element_mismatch(
    ctx: &mut LowerCtx,
    class_name: &str,
    left_method: &str,
    left_type: &str,
    right_method: &str,
    right_type: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
        format!(
            "class '{class_name}' iteration protocol mismatch: '{left_method}' yields '{left_type}' but '{right_method}' yields '{right_type}'"
        ),
        range,
    );
}

#[cfg(test)]
mod tests {
    use crate::{HirDiagnostic, lower_module};
    use ruff_text_size::{TextRange, TextSize};
    use sifr_diagnostics::DiagnosticCode;
    use sifr_python_parser::parse_module;

    fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
        let parsed = parse_module(source).expect("parse failed");
        match lower_module(parsed.suite()) {
            Ok(_) => panic!("expected lowering error"),
            Err(errors) => errors,
        }
    }

    fn range_for_after(source: &str, after: &str, needle: &str) -> TextRange {
        let search_start = source.find(after).expect("anchor should exist") + after.len();
        let relative_start = source[search_start..]
            .find(needle)
            .expect("needle should exist after anchor");
        let start = (search_start + relative_start) as u32;
        TextRange::new(
            TextSize::new(start),
            TextSize::new(start + needle.len() as u32),
        )
    }

    #[test]
    fn concrete_type_missing_protocol_bound_has_proto_code() {
        let source = "class Comparable(Protocol):\n    def __lt__(self, other: Self) -> bool:\n        pass\n\nclass Blob:\n    data: int\n\ndef choose[T: Comparable](x: T) -> T:\n    return x\n\ndef main():\n    out: Blob = choose(Blob(1))\n    print(out.data)\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'Blob' does not implement protocol 'Comparable' required by type parameter 'T'"
                && error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
                && error.primary_range == Some(range_for_after(source, "= ", "choose(Blob(1))"))
        }));
    }

    #[test]
    fn forwarded_typevar_missing_protocol_bound_has_proto_code() {
        let source = "class Readable(Protocol):\n    def read(self) -> str:\n        pass\n\nclass Closable(Protocol):\n    def close(self) -> None:\n        pass\n\ndef take_readable[T: Readable](x: T) -> T:\n    return x\n\ndef relay_bad[U: Closable](x: U) -> U:\n    return take_readable(x)\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'U' does not implement protocol 'Readable' required by type parameter 'T'"
                && error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
                && error.primary_range
                    == Some(range_for_after(source, "return ", "take_readable(x)"))
        }));
    }

    #[test]
    fn missing_context_manager_has_proto_code() {
        let source = "class PlainClass:\n    value: int\n\ndef main():\n    with PlainClass(42) as p:\n        print(p.value)\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'PlainClass' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"
                && error.code == Some(DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING)
                && error.primary_range == Some(range_for_after(source, "with ", "PlainClass(42)"))
        }));
    }

    #[test]
    fn incomplete_context_manager_has_proto_code() {
        let source = "class HalfContext:\n    def __enter__(self) -> HalfContext:\n        return self\n\ndef main():\n    with HalfContext() as ctx:\n        print(ctx)\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'HalfContext' used in 'with' statement must implement both __enter__ and __exit__ methods"
                && error.code == Some(DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING)
                && error.primary_range == Some(range_for_after(source, "with ", "HalfContext()"))
        }));
    }

    #[test]
    fn non_class_context_manager_has_proto_code() {
        let source = "def main():\n    with 1 as value:\n        print(value)\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "type 'int' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"
                && error.code == Some(DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING)
                && error.primary_range == Some(range_for_after(source, "with ", "1"))
        }));
    }

    #[test]
    fn invalid_iter_signature_has_proto_code() {
        let source = "class BadIter:\n    def __iter__(self) -> int:\n        return 1\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message == "class 'BadIter.__iter__' must return 'Iterator[T]' or 'Iterable[T]'"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
                && error.primary_range == Some(range_for_after(source, "def ", "__iter__"))
        }));
    }

    #[test]
    fn invalid_next_signature_has_proto_code() {
        let source = "class BadNext:\n    def __iter__(self) -> Iterator[int]:\n        return iter([1])\n\n    def __next__(self) -> int:\n        return 1\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message == "class 'BadNext.__next__' must return 'T | None'"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
                && error.primary_range == Some(range_for_after(source, "def ", "__next__"))
        }));
    }

    #[test]
    fn invalid_reversed_signature_has_proto_code() {
        let source = "class BadReversed:\n    def __iter__(self) -> Iterator[int]:\n        return iter([1, 2])\n\n    def __reversed__(self) -> int:\n        return 0\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "class 'BadReversed.__reversed__' must return 'Iterator[T]' or 'Iterable[T]'"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
                && error.primary_range == Some(range_for_after(source, "def ", "__reversed__"))
        }));
    }

    #[test]
    fn invalid_iter_parameter_signature_has_proto_code() {
        let source = "class BadIterParam:\n    def __iter__(self, limit: int) -> Iterator[int]:\n        return iter([1])\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "class 'BadIterParam.__iter__' must not declare parameters besides self"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
                && error.primary_range == Some(range_for_after(source, "def ", "__iter__"))
        }));
    }

    #[test]
    fn iter_next_element_mismatch_has_proto_code() {
        let source = "class BadNextMismatch:\n    def __iter__(self) -> Iterator[int]:\n        return iter([1])\n\n    def __next__(self) -> str | None:\n        return \"x\"\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "class 'BadNextMismatch' iteration protocol mismatch: '__iter__' yields 'int' but '__next__' yields 'str'"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
                && error.primary_range == Some(range_for_after(source, "class ", "BadNextMismatch"))
        }));
    }

    #[test]
    fn iter_reversed_element_mismatch_has_proto_code() {
        let source = "class BadReversedMismatch:\n    def __iter__(self) -> Iterator[int]:\n        return iter([1])\n\n    def __reversed__(self) -> Iterator[str]:\n        return iter([\"x\"])\n";
        let errors = lower_errors(source);

        assert!(errors.iter().any(|error| {
            error.message
                == "class 'BadReversedMismatch' iteration protocol mismatch: '__iter__' yields 'int' but '__reversed__' yields 'str'"
                && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
                && error.primary_range == Some(range_for_after(source, "class ", "BadReversedMismatch"))
        }));
    }
}
