use crate::{CargoPackageId, ResolvedPackageCapabilities, SifrManifest};
use sifr_sql_contract::{PackageCapabilityResolver, UnsafeSyntaxGrant, UnsafeSyntaxLint};
use std::path::Path;

#[test]
fn manifest_capability_resolver_is_the_unsafe_sql_authority() {
    let denied = manifest("");
    let denied = ResolvedPackageCapabilities::from_manifest("app@1.0.0", &denied);
    assert!(!denied.allows("app@1.0.0", "sql.unsafe-syntax"));
    assert!(
        UnsafeSyntaxGrant::from_package_resolver(
            &denied,
            "app@1.0.0",
            UnsafeSyntaxLint::Warn,
            "audited operator command",
        )
        .is_err()
    );

    let allowed = manifest("[trust]\nsecurity-capabilities = [\"sql.unsafe-syntax\"]\n");
    let allowed = ResolvedPackageCapabilities::from_manifest("app@1.0.0", &allowed);
    assert!(allowed.allows("app@1.0.0", "sql.unsafe-syntax"));
    assert!(
        UnsafeSyntaxGrant::from_package_resolver(
            &allowed,
            "app@1.0.0",
            UnsafeSyntaxLint::Warn,
            "audited operator command",
        )
        .is_ok()
    );
    assert!(
        UnsafeSyntaxGrant::from_package_resolver(
            &allowed,
            "app@1.0.0",
            UnsafeSyntaxLint::Deny,
            "audited operator command",
        )
        .is_err()
    );
}

fn manifest(extra: &str) -> SifrManifest {
    let source =
        format!("[package]\nname = \"app\"\nedition = \"2026\"\nsifr-version = \"*\"\n{extra}");
    SifrManifest::parse(
        &CargoPackageId("app 0.1.0 (path+file:///app)".to_string()),
        Path::new("/app/sifr.toml"),
        &source,
    )
    .expect("test manifest")
}
