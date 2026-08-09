use crate::lower_module;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

#[test]
fn rust_opaque_resource_direct_construction_is_rejected() {
    let parsed = parse_module(
        r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=close)
class Resource:
    @rust(bridge.resources.close)
    def close(own self) -> Result[None, ResourceError]: ...

def invalid() -> Resource:
    return Resource()
",
    )
    .expect("source should parse");
    let errors = lower_module(parsed.suite()).expect_err("direct construction must fail");

    assert_eq!(errors.len(), 1, "rejection must have one stable owner");
    assert_eq!(
        errors[0].code,
        Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)
    );
    assert_eq!(
        errors[0].message,
        "sealed Rust opaque resource `Resource` cannot be constructed in Sifr; use its declared package factory"
    );
}

#[test]
fn rust_opaque_resource_alias_construction_is_rejected() {
    let parsed = parse_module(
        r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=close)
class Resource:
    @rust(bridge.resources.close)
    def close(own self) -> Result[None, ResourceError]: ...

type ResourceAlias = Resource

def invalid() -> Resource:
    return ResourceAlias()
",
    )
    .expect("source should parse");
    let errors = lower_module(parsed.suite()).expect_err("alias construction must fail");

    assert_eq!(errors.len(), 1, "rejection must have one stable owner");
    assert_eq!(
        errors[0].code,
        Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)
    );
    assert_eq!(
        errors[0].message,
        "sealed Rust opaque resource `ResourceAlias` cannot be constructed in Sifr; use its declared package factory"
    );
}
