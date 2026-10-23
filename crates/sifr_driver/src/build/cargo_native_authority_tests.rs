//! Cargo storage reuse never substitutes for each request's lock authority.
use super::*;
use crate::build::materialize::materialize_binary_project_at_path;
use crate::build::materialize::tests::{base_project, test_dependency_plan};

#[test]
fn shared_target_rechecks_independent_authorities_and_captures_current_outputs() {
    let root = fixture("authorities");
    let dependency = root.join("dependency");
    fs::create_dir_all(dependency.join("src")).expect("dependency");
    let manifest = |version: &str| {
        format!(
            "[package]\nname = \"authority_dependency\"\nversion = \"{version}\"\nedition = \"2024\"\n"
        )
    };
    fs::write(dependency.join("Cargo.toml"), manifest("1.0.0")).expect("manifest");
    fs::write(dependency.join("value.txt"), "first").expect("build input");
    fs::write(dependency.join("build.rs"), r#"use std::io::Write;
fn main() {
    std::fs::OpenOptions::new().create(true).append(true).open("runs.log").unwrap().write_all(b"run\n").unwrap();
    println!("cargo:rerun-if-changed=value.txt");
    println!("cargo:rustc-env=VALUE={}", std::fs::read_to_string("value.txt").unwrap());
}"#).expect("build script");
    fs::write(
        dependency.join("src/lib.rs"),
        "pub fn value() -> &'static str { env!(\"VALUE\") }",
    )
    .expect("library");
    let authority_a = root.join("a.lock");
    let authority_b = root.join("b.lock");
    for path in [&authority_a, &authority_b] {
        fs::write(path, "version = 4\n").expect("independent authority");
    }
    let mut a = CargoResolutionPolicy::normal();
    a.cargo_vendor_mode = CargoVendorMode::PackageOwned;
    a.authoritative_locks = vec![authority_a.clone()];
    let mut b = a.clone();
    b.authoritative_locks = vec![authority_b.clone()];
    let mut plan = test_dependency_plan("authority-storage");
    plan.sysroot_root = root.join("runtime");
    plan.cargo_vendor_mode = CargoVendorMode::PackageOwned;
    plan.retained_direct_dependencies.push(format!(
        "authority_dependency = {{ path = {dependency:?} }}"
    ));
    let family_root = |policy: &CargoResolutionPolicy| {
        policy
            .native_family(
                &plan.sysroot_root.join("crates/sifr_runtime"),
                plan.cargo_vendor_mode,
                &[],
                "",
            )
            .expect("family")
            .root
            .clone()
    };
    let family = family_root(&a);
    assert_eq!(family, family_root(&b));
    assert_ne!(
        a.normal_seed_cache_fragment(),
        b.normal_seed_cache_fragment()
    );
    let build = |policy: &CargoResolutionPolicy, scope: &str| {
        let mut project = base_project();
        project.main_rs =
            format!("fn main() {{ println!(\"{scope}:{{}}\", authority_dependency::value()); }}");
        materialize_binary_project_at_path(
            &root.join(scope).join("project"),
            "same_name",
            project,
            &plan,
            policy,
        )
    };
    let execute = |path: &std::path::Path| {
        let output = std::process::Command::new(path)
            .output()
            .expect("actual executable");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        String::from_utf8(output.stdout).expect("UTF-8 output")
    };
    let first = build(&a, "one").expect("first authority");
    let second = build(&b, "two").expect("second authority");
    assert_ne!(first.binary_path, second.binary_path);
    assert_eq!(execute(&first.binary_path), "one:first\n");
    assert_eq!(execute(&second.binary_path), "two:first\n");
    assert_eq!(
        fs::read_to_string(dependency.join("runs.log")).expect("build count"),
        "run\n"
    );

    let old_seed = b.normal_seed_cache_fragment();
    fs::write(&authority_b, "version = 4\n# new package authority\n").expect("changed authority");
    fs::write(dependency.join("Cargo.toml"), manifest("2.0.0")).expect("changed package version");
    fs::write(dependency.join("value.txt"), "second").expect("changed dependency input");
    assert_ne!(old_seed, b.normal_seed_cache_fragment());
    assert_eq!(family, family_root(&b));
    let changed = build(&b, "two").expect("changed authority and dependency");
    assert_eq!(execute(&changed.binary_path), "two:second\n");
    assert_eq!(
        execute(&first.binary_path),
        "one:first\n",
        "the other request's published output must survive"
    );
    assert!(
        fs::read_to_string(root.join("two/project/Cargo.lock"))
            .expect("resolved lock")
            .contains("version = \"2.0.0\"")
    );
    assert_eq!(
        fs::read_to_string(dependency.join("runs.log")).expect("rebuild count"),
        "run\nrun\n"
    );

    fs::write(&authority_b, "invalid TOML [").expect("malformed authority");
    let error = build(&b, "two")
        .err()
        .expect("warm target cannot authorize malformed lock");
    assert!(
        error
            .iter()
            .any(|e| e.message.contains("failed to parse Cargo lock authority"))
    );
    fs::remove_file(&authority_b).expect("missing authority");
    let error = build(&b, "two")
        .err()
        .expect("warm target cannot authorize missing lock");
    assert!(error.iter().any(
        |e| e.message.contains("authoritative Cargo lockfile") && e.message.contains("missing")
    ));
    assert_eq!(
        fs::read_to_string(dependency.join("runs.log")).expect("no build after rejected authority"),
        "run\nrun\n"
    );
    assert_eq!(execute(&changed.binary_path), "two:second\n");
    fs::remove_dir_all(family).expect("owned family cleanup");
    fs::remove_dir_all(root).expect("fixture cleanup");
}
