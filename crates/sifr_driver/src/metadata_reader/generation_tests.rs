use std::{fs, path::Path, sync::Arc};
fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), dest).unwrap();
        }
    }
}
fn install(context: &crate::CompilerContext, dest: &Path) {
    let root = context.sysroot().unwrap();
    fs::create_dir_all(dest).unwrap();
    copy(&root.paths.stdlib_root, &dest.join("lib/sifr/stdlib"));
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        ".cargo/config.toml",
        "sysroot.toml",
        "crates/sifr_runtime/Cargo.toml",
        "crates/sifr_stdlib/Cargo.toml",
        "crates/sifr_structural_identity/Cargo.toml",
    ] {
        let target = dest.join(relative);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(root.root.join(relative), target).unwrap();
    }
    fs::create_dir(dest.join("vendor")).unwrap();
    let manifest = fs::read_to_string(dest.join("sysroot.toml"))
        .unwrap()
        .replace("source-tree", super::selection::target());
    fs::write(dest.join("sysroot.toml"), manifest).unwrap();
    let metadata = context.metadata_provider().unwrap().metadata.clone();
    fs::copy(&metadata.path, dest.join("lib/sifr/stdlib.sifrmeta")).unwrap();
    let hex = |b: &[u8]| b.iter().map(|b| format!("{b:02x}")).collect::<String>();
    fs::write(dest.join("lib/sifr/stdlib.metadata.json"),serde_json::to_vec(&serde_json::json!({
        "schema_version":1,"compiler_identity":context.identity().as_str(),
        "semantic_target":super::selection::target(),"semantic_target_id":hex(&metadata.compatibility.semantic_target),
        "stdlib_inputs_id":hex(&metadata.compatibility.stdlib_inputs),"metadata_id":metadata.metadata_id,
        "compiler_binary_sha256":"0".repeat(64)
    })).unwrap()).unwrap();
}
#[test]
fn dx8_m07_m08_installed_relocation_and_generation_switch_pin_all_sources() {
    let context = crate::CompilerContext::for_test();
    let scratch = tempfile::tempdir().unwrap();
    let first = scratch.path().join("first");
    install(&context, &first);
    let moved = scratch.path().join("relocated");
    fs::rename(&first, &moved).unwrap();
    let second = scratch.path().join("second");
    install(&context, &second);
    let selector = scratch.path().join("current");
    std::os::unix::fs::symlink(&moved, &selector).unwrap();
    let old = crate::CompilerContext::with_sysroot(
        context.identity().clone(),
        sifr_sysroot::resolve_sysroot(Some(selector.clone())).unwrap(),
    );
    let old_provider = old.metadata_provider().unwrap();
    let navigation = old.stdlib_navigation().unwrap();
    let symbol = navigation
        .symbols
        .iter()
        .find(|s| s.module == "sifr.math" && s.name == "sqrt")
        .unwrap();
    assert_eq!(navigation.loaded_sources(), 0);
    let pending = scratch.path().join("pending");
    std::os::unix::fs::symlink(&second, &pending).unwrap();
    fs::rename(&pending, &selector).unwrap();
    // New source generation is deliberately incompatible with old source facts.
    assert!(
        navigation
            .source(symbol.file)
            .unwrap()
            .unwrap()
            .contains("sqrt")
    );
    assert!(navigation.path(symbol.file).unwrap().starts_with(&moved));
    let new = crate::CompilerContext::with_sysroot(
        context.identity().clone(),
        sifr_sysroot::resolve_sysroot(Some(selector.clone())).unwrap(),
    );
    let new_provider = new.metadata_provider().unwrap();
    assert!(!Arc::ptr_eq(&old_provider, &new_provider));
    assert!(!Arc::ptr_eq(
        &old_provider.semantic("sifr.math").unwrap(),
        &new_provider.semantic("sifr.math").unwrap()
    ));
    let new_navigation = new.stdlib_navigation().unwrap();
    let new_symbol = new_navigation
        .symbols
        .iter()
        .find(|s| s.module == "sifr.math" && s.name == "sqrt")
        .unwrap();
    assert!(
        new_navigation
            .path(new_symbol.file)
            .unwrap()
            .starts_with(&second)
    );
    fs::write(
        new_navigation.path(new_symbol.file).unwrap(),
        "# new generation\n",
    )
    .unwrap();
    assert!(
        new_navigation
            .source(new_symbol.file)
            .unwrap_err()
            .contains("identity differs")
    );
    std::os::unix::fs::symlink(&moved, &pending).unwrap();
    fs::rename(pending, selector).unwrap();
    assert!(old.qualify_metadata().is_ok());
}
