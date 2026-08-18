use super::test_support::package;
use super::{
    discover_python_bridge_inventory, required_python_bridge_archive_entries,
    validate_python_bridge_inventory_manifest, write_python_bridge_inventory, PythonBridgeImport,
    PYTHON_BRIDGE_INVENTORY,
};
use crate::manifest::sifr::{PythonConfig, TrustPolicy};
use sifr_diagnostics::DiagnosticCode;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn bridge_inventory_classifies_static_external_and_same_package_imports() {
    let fixture = BridgeFixture::new("static_imports");
    fixture.write("helpers.py", "def normalize(value):\n    return value\n");
    fixture.write("pkg/__init__.py", "from . import local\n");
    fixture.write("pkg/local.py", "VALUE = 3\n");
    fixture.write(
        "pkg/adapter.py",
        "import requests.sessions\nfrom bridge import helpers\nfrom . import local\n",
    );

    let inventory = discover_python_bridge_inventory(&fixture.package)
        .expect("static bridge inventory should be valid");

    assert_eq!(
        inventory
            .modules
            .iter()
            .map(|module| module.module.as_str())
            .collect::<Vec<_>>(),
        ["helpers", "pkg", "pkg.adapter", "pkg.local"]
    );
    let adapter = inventory
        .modules
        .iter()
        .find(|module| module.module == "pkg.adapter")
        .expect("adapter module");
    assert_eq!(
        adapter.imports,
        [
            PythonBridgeImport::SamePackage {
                module: "helpers".to_string()
            },
            PythonBridgeImport::SamePackage {
                module: "pkg".to_string()
            },
            PythonBridgeImport::SamePackage {
                module: "pkg.local".to_string()
            },
            PythonBridgeImport::ThirdParty {
                root: "requests".to_string()
            },
        ]
    );
}

#[test]
fn invalid_python_bridge_source_reports_pyimp_0002() {
    let fixture = BridgeFixture::new("invalid_syntax");
    fixture.write("broken.py", "def broken(:\n    pass\n");

    let diagnostics = discover_python_bridge_inventory(&fixture.package)
        .expect_err("invalid Python syntax must fail inventory");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYIMP_INVALID_BRIDGE_SOURCE
    );
    assert!(diagnostics[0].message.contains("invalid Python syntax"));
}

#[test]
fn dynamic_import_calls_and_aliases_are_rejected() {
    for (case, source, call) in [
        ("builtin", "value = __import__('json')\n", "__import__"),
        (
            "module_alias",
            "import importlib as loader\nvalue = loader.import_module('json')\n",
            "loader.import_module",
        ),
        (
            "function_alias",
            "from importlib import import_module as load\nvalue = load('json')\n",
            "load",
        ),
        (
            "module_assignment",
            "import importlib\nloader = importlib\nvalue = loader.import_module('json')\n",
            "loader.import_module",
        ),
        (
            "function_assignment",
            "import importlib\nload = importlib.import_module\nvalue = load('json')\n",
            "load",
        ),
        (
            "tuple_assignment",
            "import importlib\nloader, load = importlib, importlib.import_module\nvalue = load('json')\n",
            "load",
        ),
        (
            "getattr_dispatch",
            "import importlib\nvalue = getattr(importlib, 'import_module')('json')\n",
            "getattr(importlib, import_module)",
        ),
        (
            "importlib_dunder",
            "import importlib\nvalue = importlib.__import__('json')\n",
            "importlib.__import__",
        ),
        (
            "importlib_dunder_alias",
            "from importlib import __import__ as load\nvalue = load('json')\n",
            "load",
        ),
        (
            "builtin_assignment",
            "load = __import__\nvalue = load('json')\n",
            "load",
        ),
        (
            "importlib_star",
            "from importlib import *\nvalue = import_module('json')\n",
            "import_module",
        ),
        (
            "builtins_star",
            "from builtins import *\nvalue = __import__('json')\n",
            "__import__",
        ),
    ] {
        let fixture = BridgeFixture::new(case);
        fixture.write("dynamic.py", source);
        let diagnostics = discover_python_bridge_inventory(&fixture.package)
            .expect_err("dynamic import calls must fail inventory");
        assert!(diagnostics[0].message.contains(call));
        assert_eq!(
            diagnostics[0].code,
            DiagnosticCode::PYIMP_INVALID_BRIDGE_SOURCE
        );
    }
}

#[test]
fn dynamic_import_callable_reference_without_a_call_is_allowed() {
    let fixture = BridgeFixture::new("dynamic_reference");
    fixture.write(
        "reference.py",
        "import importlib\nresolver = importlib.import_module\nNAME = resolver.__name__\n",
    );

    discover_python_bridge_inventory(&fixture.package)
        .expect("a callable reference without a dynamic import call is valid");
}

#[test]
fn invalid_module_paths_duplicates_and_relative_escape_are_distinct() {
    let fixture = BridgeFixture::new("invalid_modules");
    fixture.write("__init__.py", "VALUE = 1\n");
    fixture.write("class.py", "VALUE = 2\n");
    fixture.write("duplicate.py", "VALUE = 3\n");
    fixture.write("duplicate/__init__.py", "VALUE = 4\n");
    fixture.write("escape.py", "from .. import outside\n");

    let diagnostics = discover_python_bridge_inventory(&fixture.package)
        .expect_err("invalid bridge module paths must fail");

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("root __init__.py is reserved")));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("valid Python identifiers")));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("is defined by both")));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("from .. import outside")));
}

#[test]
fn non_utf8_bridge_source_has_an_encoding_diagnostic() {
    let fixture = BridgeFixture::new("non_utf8");
    fixture.write_bytes("legacy.py", &[b'V', b'A', b'L', b'U', b'E', b'=', 0xff]);

    let diagnostics = discover_python_bridge_inventory(&fixture.package)
        .expect_err("non-UTF-8 bridge source must fail");
    assert!(diagnostics[0].message.contains("must be UTF-8 encoded"));
}

#[test]
fn same_package_imports_include_package_and_intermediate_ancestors() {
    let fixture = BridgeFixture::new("ancestor_imports");
    fixture.write("pkg/__init__.py", "NAME = 1\n");
    fixture.write("pkg/local.py", "VALUE = 2\n");
    fixture.write(
        "pkg/consumer.py",
        "from . import local, NAME\nfrom .deep import helper, NAME\n",
    );
    fixture.write("pkg/deep/helper.py", "VALUE = 3\n");
    fixture.write(
        "adapter.py",
        "import bridge.pkg.deep.helper\nfrom bridge.pkg import local, NAME\n",
    );

    let inventory = discover_python_bridge_inventory(&fixture.package).expect("inventory");
    let adapter = inventory
        .modules
        .iter()
        .find(|module| module.module == "adapter")
        .expect("adapter");
    assert_eq!(
        adapter.imports,
        [
            PythonBridgeImport::SamePackage {
                module: "pkg".to_string()
            },
            PythonBridgeImport::SamePackage {
                module: "pkg.deep".to_string()
            },
            PythonBridgeImport::SamePackage {
                module: "pkg.deep.helper".to_string()
            },
            PythonBridgeImport::SamePackage {
                module: "pkg.local".to_string()
            },
        ]
    );
    let consumer = inventory
        .modules
        .iter()
        .find(|module| module.module == "pkg.consumer")
        .expect("consumer");
    assert_eq!(
        consumer.imports,
        [
            PythonBridgeImport::SamePackage {
                module: "pkg".to_string()
            },
            PythonBridgeImport::SamePackage {
                module: "pkg.deep".to_string()
            },
            PythonBridgeImport::SamePackage {
                module: "pkg.deep.helper".to_string()
            },
            PythonBridgeImport::SamePackage {
                module: "pkg.local".to_string()
            },
        ]
    );
}

#[test]
fn misplaced_bridge_root_and_reserved_runtime_import_are_rejected() {
    let fixture = BridgeFixture::new("misplaced");
    fixture.write_at("python_bridges/wrong.py", "VALUE = 1\n");
    fixture.write("reserved.py", "import __sifr_bridge__.foreign\n");

    let diagnostics = discover_python_bridge_inventory(&fixture.package)
        .expect_err("misplaced and reserved bridge sources must fail");

    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("source root must be")));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("reserved runtime namespace")));
}

#[test]
fn inventory_digest_is_stable_and_changes_with_source_or_imports() {
    let fixture = BridgeFixture::new("digests");
    fixture.write("adapter.py", "import json\nVALUE = 1\n");
    let first = discover_python_bridge_inventory(&fixture.package).expect("first inventory");
    let second = discover_python_bridge_inventory(&fixture.package).expect("second inventory");
    assert_eq!(first.inventory_digest, second.inventory_digest);
    assert_eq!(
        first.modules[0].source_digest,
        second.modules[0].source_digest
    );

    fixture.write("adapter.py", "import decimal\nVALUE = 2\n");
    let changed = discover_python_bridge_inventory(&fixture.package).expect("changed inventory");
    assert_ne!(first.inventory_digest, changed.inventory_digest);
    assert_ne!(
        first.modules[0].source_digest,
        changed.modules[0].source_digest
    );
}

#[test]
fn bridge_sources_and_generated_inventory_are_required_archive_entries() {
    let fixture = BridgeFixture::new("archive_entries");
    fixture.write("adapter.py", "VALUE = 1\n");
    fixture.write("nested/helper.py", "VALUE = 2\n");
    let inventory = discover_python_bridge_inventory(&fixture.package).expect("inventory");
    let inventory_path = write_python_bridge_inventory(&fixture.package, &inventory)
        .expect("write inventory")
        .expect("non-empty inventory path");

    assert_eq!(inventory_path, fixture.root.join(PYTHON_BRIDGE_INVENTORY));
    assert_eq!(
        required_python_bridge_archive_entries(&fixture.root),
        [
            PathBuf::from("src/python_bridges/__sifr_inventory__.json"),
            PathBuf::from("src/python_bridges/adapter.py"),
            PathBuf::from("src/python_bridges/nested/helper.py"),
        ]
        .into_iter()
        .collect()
    );
    let decoded: super::PythonBridgeInventory =
        serde_json::from_str(&fs::read_to_string(inventory_path).expect("read written inventory"))
            .expect("inventory JSON");
    assert_eq!(decoded, inventory);
    let package_required = crate::cargo::package::required_archive_entries(
        &fixture.package,
        &crate::imports::source_map::PackageSourceMap::default(),
    );
    assert!(package_required.contains(&PathBuf::from("src/python_bridges/__sifr_inventory__.json")));
    assert!(package_required.contains(&PathBuf::from("src/python_bridges/adapter.py")));
}

#[test]
fn package_validation_rejects_missing_and_stale_generated_inventory() {
    let fixture = BridgeFixture::new("stale_inventory");
    fixture.write("adapter.py", "VALUE = 1\n");
    let inventory = discover_python_bridge_inventory(&fixture.package).expect("inventory");

    let missing = validate_python_bridge_inventory_manifest(&fixture.package, &inventory)
        .expect_err("missing generated inventory must fail");
    assert!(missing.message.contains("missing or unreadable"));
    let archive_diagnostics = crate::cargo::package::validate_package_archive(
        &fixture.package,
        &crate::imports::source_map::PackageSourceMap::default(),
        &[
            crate::cargo::package::PackageArchiveEntry {
                relative_path: PathBuf::from("sifr.toml"),
            },
            crate::cargo::package::PackageArchiveEntry {
                relative_path: PathBuf::from("src/python_bridges/adapter.py"),
            },
            crate::cargo::package::PackageArchiveEntry {
                relative_path: PathBuf::from(PYTHON_BRIDGE_INVENTORY),
            },
        ],
    )
    .expect_err("package archive validation must inspect the generated inventory");
    assert!(archive_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PYIMP_INVALID_BRIDGE_SOURCE
            && diagnostic.message.contains("missing or unreadable")
    }));

    write_python_bridge_inventory(&fixture.package, &inventory).expect("write inventory");
    validate_python_bridge_inventory_manifest(&fixture.package, &inventory)
        .expect("fresh inventory should validate");

    fixture.write("adapter.py", "VALUE = 2\n");
    let changed = discover_python_bridge_inventory(&fixture.package).expect("changed inventory");
    let stale = validate_python_bridge_inventory_manifest(&fixture.package, &changed)
        .expect_err("stale generated inventory must fail");
    assert!(stale.message.contains("is stale"));
}

pub(super) struct BridgeFixture {
    pub(super) root: PathBuf,
    pub(super) package: crate::graph::derive::SifrPackageMetadata,
}

impl BridgeFixture {
    pub(super) fn new(label: &str) -> Self {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "sifr_python_bridge_inventory_{}_{}_{}",
            std::process::id(),
            sequence,
            label
        ));
        fs::create_dir_all(root.join("src/python_bridges")).expect("create bridge root");
        let mut package = package(
            "bridge_pkg",
            PythonConfig::default(),
            TrustPolicy::default(),
        );
        package.package_root = root.clone();
        package.sifr_manifest = root.join("sifr.toml");
        Self { root, package }
    }

    fn write(&self, relative: &str, source: &str) {
        self.write_at(&format!("src/python_bridges/{relative}"), source);
    }

    pub(super) fn write_at(&self, relative: &str, source: &str) {
        self.write_bytes_at(relative, source.as_bytes());
    }

    fn write_bytes(&self, relative: &str, source: &[u8]) {
        self.write_bytes_at(&format!("src/python_bridges/{relative}"), source);
    }

    fn write_bytes_at(&self, relative: &str, source: &[u8]) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("source parent")).expect("create source parent");
        fs::write(path, source).expect("write bridge source");
    }
}

impl Drop for BridgeFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
