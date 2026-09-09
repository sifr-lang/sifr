//! Closed compiler-component registration and resolved artifact integrity.

use super::package_graph;
use crate::cargo::metadata::CargoPackageId;
use crate::manifest::sifr::{SifrManifest, TrustPolicy};
use crate::test_support::{TestExpectErr as _, TestUnwrap as _};
use crate::{CompilerComponentConfig, compiler_component_registrations, resolve_package_component};
use semver::Version;
use sifr_compiler_component::{
    ComponentErrorKind, ComponentIdentity, ComponentRequirement, DiagnosticRegistry,
    DiagnosticRegistryOwner, ProtocolRange,
};
use std::path::PathBuf;

#[test]
fn compiler_component_manifest_parses_closed_registration() {
    let cargo_package_id = CargoPackageId("path+file:///ws/words#words@1.2.3".to_string());
    let manifest = SifrManifest::parse(
        &cargo_package_id,
        &PathBuf::from("/ws/words/sifr.toml"),
        r#"
[package]
name = "words"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[compiler-components.words]
kind = "embedded-language-provider"
artifact = "components/words.wasm"
version = "1.2.3"
sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
protocol-min = 1
protocol-max = 1
processors = ["words.document"]
diagnostic-namespace = "WORDS"
diagnostics = [
  { code = "SIFR-WORDS-0001", lifecycle = "active" },
  { code = "SIFR-WORDS-0002", lifecycle = "deprecated" },
]
"#,
    )
    .test_unwrap("component registration should parse");

    let component = &manifest.compiler_components["words"];
    assert_eq!(component.version.to_string(), "1.2.3");
    assert_eq!(component.processors, ["words.document"]);
    assert_eq!(component.diagnostics.declarations.len(), 2);
    assert_eq!(
        component.registrations("words")[0].diagnostics,
        component.diagnostics
    );
}

#[test]
fn compiler_component_manifest_rejects_compiler_diagnostic_namespace() {
    let cargo_package_id = CargoPackageId("path+file:///ws/words#words@1.2.3".to_string());
    let diagnostic = SifrManifest::parse(
        &cargo_package_id,
        &PathBuf::from("/ws/words/sifr.toml"),
        r#"
[package]
name = "words"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[compiler-components.words]
kind = "embedded-language-provider"
artifact = "components/words.wasm"
version = "1.2.3"
sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
protocol-min = 1
protocol-max = 1
processors = ["words.document"]
diagnostic-namespace = "COMPONENT"
diagnostics = []
"#,
    )
    .test_expect_err("provider must not claim the compiler namespace");

    assert!(diagnostic.message.contains("compiler-owned"));
}

#[test]
fn package_component_resolution_checks_exact_identity_and_artifact_hash() {
    let root = std::env::temp_dir().join(format!(
        "sifr-package-component-resolution-{}",
        std::process::id()
    ));
    let component_dir = root.join("components");
    std::fs::create_dir_all(&component_dir).test_unwrap("component fixture directory should exist");
    let artifact = component_dir.join("words.wasm");
    let bytes = b"fixture-component";
    std::fs::write(&artifact, bytes).test_unwrap("component fixture should be written");
    let sha256 = crate::digest::sha256_hex(bytes);
    let mut graph = package_graph(TrustPolicy::default(), Vec::new());
    let package = graph
        .packages
        .values_mut()
        .next()
        .test_unwrap("package exists");
    package.package_root.clone_from(&root);
    package.manifest.compiler_components.insert(
        "words".to_string(),
        CompilerComponentConfig {
            kind: "embedded-language-provider".to_string(),
            artifact: PathBuf::from("components/words.wasm"),
            version: Version::new(1, 0, 0),
            sha256: sha256.clone(),
            protocol: ProtocolRange {
                minimum: 1,
                maximum: 1,
            },
            processors: vec!["fixture.words".to_string()],
            diagnostics: DiagnosticRegistry {
                owner: DiagnosticRegistryOwner::Provider {
                    namespace: "FIXTURE".to_string(),
                },
                declarations: Vec::new(),
            },
        },
    );
    let requirement = ComponentRequirement {
        identity: ComponentIdentity {
            package: package.package_id.0.clone(),
            processor: "fixture.words".to_string(),
            version: Version::new(1, 0, 0),
            sha256,
        },
        protocol_major: 1,
    };
    let resolved = resolve_package_component(&graph, &requirement)
        .test_unwrap("exact package component should resolve");
    assert_eq!(resolved.bytes, bytes);

    std::fs::write(&artifact, b"drifted").test_unwrap("fixture should change");
    assert!(resolve_package_component(&graph, &requirement).is_err());
    std::fs::remove_dir_all(root).test_unwrap("component fixture should be removable");
}

#[test]
fn package_component_graph_rejects_duplicate_diagnostic_namespaces() {
    let mut graph = package_graph(TrustPolicy::default(), Vec::new());
    let package = graph
        .packages
        .values_mut()
        .next()
        .test_unwrap("package exists");
    for (name, processor) in [("first", "fixture.first"), ("second", "fixture.second")] {
        package.manifest.compiler_components.insert(
            name.to_string(),
            CompilerComponentConfig {
                kind: "embedded-language-provider".to_string(),
                artifact: PathBuf::from(format!("components/{name}.wasm")),
                version: Version::new(1, 0, 0),
                sha256: "a".repeat(64),
                protocol: ProtocolRange {
                    minimum: 1,
                    maximum: 1,
                },
                processors: vec![processor.to_string()],
                diagnostics: DiagnosticRegistry {
                    owner: DiagnosticRegistryOwner::Provider {
                        namespace: "FIXTURE".to_string(),
                    },
                    declarations: Vec::new(),
                },
            },
        );
    }

    let error = compiler_component_registrations(&graph)
        .test_expect_err("one provider namespace must have one component owner");
    assert_eq!(error.kind, ComponentErrorKind::DiagnosticRegistry);
}
