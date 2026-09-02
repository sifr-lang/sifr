use crate::test_support::TestUnwrap as _;

use crate::{
    CargoPackageId, ResolvedPackageCapabilities, SifrManifest, SifrPackageGraph, SifrPackageId,
    SifrPackageMetadata,
};
use sifr_sql_contract::{PackageCapabilityResolver, UnsafeSyntaxLint};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[test]
fn root_manifest_is_the_only_unsafe_sql_authority() {
    let app = SifrPackageId("app@1.0.0".to_string());
    let dependency = SifrPackageId("dependency@1.0.0".to_string());
    let dependency_grant_graph = graph(
        (&app, manifest("")),
        (
            &dependency,
            manifest("[trust]\nsecurity-capabilities = [\"sql.unsafe-syntax\"]\n"),
        ),
    );

    let denied = ResolvedPackageCapabilities::from_root_package(&dependency_grant_graph, &app)
        .test_unwrap("test precondition should hold");
    assert!(!denied.allows(&app.0, "sql.unsafe-syntax"));
    assert!(
        denied
            .unsafe_syntax_grant(&app, UnsafeSyntaxLint::Warn, "audited operator command",)
            .is_err()
    );
    assert!(
        ResolvedPackageCapabilities::from_root_package(&dependency_grant_graph, &dependency)
            .is_err()
    );

    let root_grant_graph = graph(
        (
            &app,
            manifest("[trust]\nsecurity-capabilities = [\"sql.unsafe-syntax\"]\n"),
        ),
        (&dependency, manifest("")),
    );
    let allowed = ResolvedPackageCapabilities::from_root_package(&root_grant_graph, &app)
        .test_unwrap("test precondition should hold");
    assert!(allowed.allows(&app.0, "sql.unsafe-syntax"));
    assert!(
        allowed
            .unsafe_syntax_grant(&app, UnsafeSyntaxLint::Warn, "audited operator command",)
            .is_ok()
    );
    assert!(
        allowed
            .unsafe_syntax_grant(&app, UnsafeSyntaxLint::Deny, "audited operator command",)
            .is_err()
    );
    assert!(
        allowed
            .unsafe_syntax_grant(
                &dependency,
                UnsafeSyntaxLint::Warn,
                "dependency tries to use the root grant",
            )
            .is_err()
    );
}

fn graph(
    root: (&SifrPackageId, SifrManifest),
    dependency: (&SifrPackageId, SifrManifest),
) -> SifrPackageGraph {
    SifrPackageGraph {
        packages: BTreeMap::from([
            (root.0.clone(), metadata(root.0, root.1)),
            (dependency.0.clone(), metadata(dependency.0, dependency.1)),
        ]),
        cargo_edges: BTreeMap::from([(root.0.clone(), BTreeSet::from([dependency.0.clone()]))]),
        direct_dependency_scopes: BTreeMap::new(),
        backend_crates: BTreeMap::new(),
        classifications: BTreeMap::new(),
    }
}

fn metadata(package_id: &SifrPackageId, manifest: SifrManifest) -> SifrPackageMetadata {
    let cargo_package_id = CargoPackageId(format!("{} (path+file:///fixture)", package_id.0));
    SifrPackageMetadata {
        package_id: package_id.clone(),
        cargo_package_id,
        cargo_package_name: manifest.package_name.0.clone(),
        cargo_version: "1.0.0".to_string(),
        cargo_source: None,
        package_root: PathBuf::from("/fixture"),
        sifr_manifest: PathBuf::from("/fixture/sifr.toml"),
        sifr_name: manifest.package_name.clone(),
        manifest,
        aliases: BTreeMap::new(),
    }
}

fn manifest(extra: &str) -> SifrManifest {
    let source =
        format!("[package]\nname = \"app\"\nedition = \"2026\"\nsifr-version = \"*\"\n{extra}");
    SifrManifest::parse(
        &CargoPackageId("app 0.1.0 (path+file:///app)".to_string()),
        Path::new("/app/sifr.toml"),
        &source,
    )
    .test_unwrap("test manifest")
}
