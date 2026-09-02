use crate::test_support::TestUnwrap as _;

use crate::cargo::commands::{
    CargoCommandPlan, CargoFeatureSelection, CargoPackageArchiveOptions, CargoPackageSelection,
    CargoPublishOptions, CargoVendorOptions,
};
use crate::cargo::lock_modes::CargoLockMode;
use crate::ops::plan::PackageOperation;
use crate::ops::publish::{publish_plan, vendor_plan};
use crate::ops::session::PackageSession;
use std::path::PathBuf;

#[test]
fn publish_and_vendor_plans_delegate_to_cargo_with_redaction_ready_commands() {
    let publish = publish_plan(PathBuf::from("/ws/app"), CargoLockMode::Frozen, true);
    assert_eq!(
        publish.cargo_command.args,
        ["publish", "--frozen", "--dry-run"]
    );

    let vendor = vendor_plan(
        PathBuf::from("/ws"),
        CargoLockMode::Locked,
        PathBuf::from("vendor"),
    );
    assert_eq!(vendor.cargo_command.args, ["vendor", "--locked", "vendor"]);
}

#[test]
fn package_publish_vendor_command_plans_cover_release_flags() {
    let features = CargoFeatureSelection {
        features: vec!["serde".to_string(), "json".to_string()],
        all_features: false,
        no_default_features: true,
    };
    let selection = CargoPackageSelection {
        workspace: true,
        packages: vec!["sifr-app".to_string()],
        excludes: vec!["sifr-tools".to_string()],
    };
    let package = CargoCommandPlan::package_with_options(
        PathBuf::from("/ws"),
        CargoLockMode::Frozen,
        &features,
        &selection,
        &CargoPackageArchiveOptions {
            list: true,
            no_verify: true,
            no_metadata: true,
            allow_dirty: true,
            exclude_lockfile: true,
        },
    );
    assert_eq!(
        package.args,
        [
            "package",
            "--frozen",
            "--workspace",
            "-p",
            "sifr-app",
            "--exclude",
            "sifr-tools",
            "--no-default-features",
            "--features",
            "json,serde",
            "--list",
            "--no-verify",
            "--no-metadata",
            "--allow-dirty",
            "--exclude-lockfile"
        ]
    );

    let publish = CargoCommandPlan::publish_with_options(
        PathBuf::from("/ws"),
        CargoLockMode::Locked,
        &features,
        &selection,
        &CargoPublishOptions {
            dry_run: true,
            no_verify: true,
            allow_dirty: true,
        },
    );
    assert_eq!(
        publish.args,
        [
            "publish",
            "--locked",
            "--workspace",
            "-p",
            "sifr-app",
            "--exclude",
            "sifr-tools",
            "--no-default-features",
            "--features",
            "json,serde",
            "--dry-run",
            "--no-verify",
            "--allow-dirty"
        ]
    );

    let vendor = CargoCommandPlan::vendor_with_options(
        PathBuf::from("/ws"),
        CargoLockMode::Offline,
        &PathBuf::from("vendor"),
        &CargoVendorOptions {
            sync: vec![PathBuf::from("member/Cargo.toml")],
            no_delete: true,
            respect_source_config: true,
            versioned_dirs: true,
        },
    );
    assert_eq!(
        vendor.args,
        [
            "vendor",
            "--offline",
            "--sync",
            "member/Cargo.toml",
            "--no-delete",
            "--respect-source-config",
            "--versioned-dirs",
            "vendor"
        ]
    );
}

#[test]
fn package_publish_vendor_session_plans_route_through_package_session() {
    let session = PackageSession {
        workspace_root: PathBuf::from("/ws"),
        manifest_path: Some(PathBuf::from("/ws/sifr.toml")),
        source_root: Some(PathBuf::from("/ws/src")),
        manifest_less_mode: false,
        lock_mode: CargoLockMode::Locked,
        manifest: None,
        app_targets: Ok(Vec::new()),
    };
    let selection = CargoPackageSelection {
        workspace: false,
        packages: vec!["sifr-app".to_string()],
        excludes: Vec::new(),
    };
    let package = session.plan_package(
        &CargoFeatureSelection::default(),
        &selection,
        &CargoPackageArchiveOptions::default(),
    );
    assert_eq!(package.operation.operation, PackageOperation::Package);
    assert_eq!(
        package.cargo.test_unwrap("package cargo plan").args,
        ["package", "--locked", "-p", "sifr-app"]
    );

    let publish = session.plan_publish(
        &CargoFeatureSelection::default(),
        &selection,
        &CargoPublishOptions {
            dry_run: true,
            ..CargoPublishOptions::default()
        },
    );
    assert_eq!(publish.operation.operation, PackageOperation::Publish);
    assert_eq!(
        publish.cargo.test_unwrap("publish cargo plan").args,
        ["publish", "--locked", "-p", "sifr-app", "--dry-run"]
    );

    let vendor = session.plan_vendor(
        &PathBuf::from("vendor"),
        &CargoVendorOptions {
            versioned_dirs: true,
            ..CargoVendorOptions::default()
        },
    );
    assert_eq!(vendor.operation.operation, PackageOperation::Vendor);
    assert_eq!(
        vendor.cargo.test_unwrap("vendor cargo plan").args,
        ["vendor", "--locked", "--versioned-dirs", "vendor"]
    );
}
