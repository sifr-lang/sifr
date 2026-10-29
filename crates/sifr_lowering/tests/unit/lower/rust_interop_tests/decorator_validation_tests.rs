use super::{assert_malformed, lower_errors};

#[test]
fn rust_interop_requires_the_canonical_positional_target() {
    let errors = lower_errors(
        r"
@rust(crate=crc32fast, path=hash)
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_self_target_outside_methods() {
    let errors = lower_errors(
        r"
@rust(Self.poll)
def poll() -> int:
    return 1
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_opaque_on_functions() {
    let errors = lower_errors(
        r"
@rust.opaque(type=bridge.kafka.Consumer)
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_function_decorators_on_classes() {
    let errors = lower_errors(
        r"
@rust.async()
class Consumer:
    pass
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_unknown_decorator_names() {
    let errors = lower_errors(
        r"
@rust.unknown()
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_double_star_keyword_splat() {
    let errors = lower_errors(
        r"
@rust(bridge.hash.digest, **options)
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_bare_rust_decorators() {
    let errors = lower_errors(
        r"
@rust
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}
