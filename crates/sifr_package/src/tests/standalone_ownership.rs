use super::*;

#[test]
fn standalone_virtual_workspace_requires_proven_source_ownership() {
    let temp = TestPackage::new("dx15_virtual");
    temp.write_package_manifest(
        "[package]\nname = \"standalone\"\nversion = \"0.0.0\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n[source]\nroot = \".\"\n",
    );
    temp.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"broken-unrelated-member\"]\n",
    );
    temp.write("files/main.sifr", "def main():\n    pass\n");
    let file = temp.path().join("files/main.sifr");
    let eligible = || {
        PackageSession::standalone_file_in_virtual_workspace(
            temp.path(),
            &file,
            &mut DiskSourceProvider::new(),
        )
    };
    assert!(
        eligible(),
        "unrelated workspace members cannot own a standalone file"
    );
    temp.write(
        "files/Cargo.toml",
        "[package]\nname = \"real\"\nversion = \"0.0.0\"\n",
    );
    assert!(
        !eligible(),
        "an actual nested package must retain normal resolution"
    );
    fs::remove_file(temp.path().join("files/Cargo.toml")).test_unwrap("remove nested manifest");
    temp.write("files/sifr.toml", "[source]\nroot = \".\"\n");
    assert!(
        eligible(),
        "validated nested source-only ownership remains standalone"
    );
    for invalid in [
        "[source]\nroot = \"../\"\n",
        "[source]\nroot = \"absent\"\n",
        "[source]\nroot = 42\n",
        "[source]\nroot = \".\"\nbackend = \"native\"\n",
        "[package]\nname = \"nested-package\"\n[source]\nroot = \".\"\n",
    ] {
        temp.write("files/sifr.toml", invalid);
        assert!(
            !eligible(),
            "unknown or invalid nested authority must resolve normally"
        );
    }
    temp.write("files/sifr.toml", "invalid = [");
    assert!(!eligible(), "nested source policy must not be bypassed");
    fs::remove_file(temp.path().join("files/sifr.toml")).test_unwrap("remove nested policy");
    temp.write(
        "Cargo.toml",
        "[package]\nname = \"real\"\nversion = \"0.0.0\"\n[workspace]\n",
    );
    assert!(
        !eligible(),
        "root Cargo packages retain dependency and trust validation"
    );
    temp.write("Cargo.toml", "[workspace]\nmembers = [");
    assert!(
        !eligible(),
        "malformed authority must use ordinary diagnostics"
    );
}
