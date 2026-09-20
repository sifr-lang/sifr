use super::*;
use crate::cache_storage;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    path::PathBuf,
    thread,
    time::Duration,
};

fn prune(cache: &Path, workspace: &Path, pressure: bool, dry: bool) -> ProjectPruneReport {
    housekeeping::prune_workspace(cache, workspace, pressure, dry).unwrap()
}
fn wait(marker: &Path) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(marker.exists(), "worker reached its real lock boundary");
}
fn stage(store: &storage::Store, suffix: &str) -> (PathBuf, fs::File) {
    let name = format!("{}.stage-{suffix}", "a".repeat(64));
    let parent = store.root.join("generations");
    let lease = cache_storage::process_entry_lock(&parent, &name).unwrap();
    cache_storage::directory(&parent.join(&name)).unwrap();
    fs::write(parent.join(&name).join("payload"), "abandoned").unwrap();
    (parent.join(name), lease)
}

#[test]
fn abandoned_stages_reclaimed_live_locks_preserved() {
    let (root, file, cache) = tests::fixture();
    assert_eq!(tests::run(&cache, &file).1.status, "published");
    let store = tests::store(&cache, &file);
    let latest = store.latest().unwrap();
    let writer_lock = store.root.join(".locks/writer");
    let writer_inode = fs::metadata(&writer_lock).unwrap().ino();
    let (abandoned, lease) = stage(&store, "10-20");
    drop(lease);
    let abandoned_lock = abandoned
        .parent()
        .unwrap()
        .join(".locks")
        .join(abandoned.file_name().unwrap());
    let (live, live_lease) = stage(&store, "10-21");
    live_lease.try_lock().unwrap();
    let live_lock = live
        .parent()
        .unwrap()
        .join(".locks")
        .join(live.file_name().unwrap());
    let live_inode = fs::metadata(&live_lock).unwrap().ino();
    let scratch = store.root.join("latest.stage-10-22");
    fs::write(&scratch, "partial").unwrap();
    let outside = root.path().join("outside");
    fs::write(&outside, "preserve").unwrap();
    let alias = store.root.join("latest.stage-10-23");
    symlink(&outside, &alias).unwrap();
    let hint = file.parent().unwrap().join(".sifrbuildinfo.stage-10-24");
    fs::copy(file.parent().unwrap().join(".sifrbuildinfo"), &hint).unwrap();
    let unknown_hint = file.parent().unwrap().join(".sifrbuildinfo.stage-10-25");
    fs::write(&unknown_hint, "unknown owner").unwrap();
    let report = store.prune(true, false).unwrap();
    assert!(report.deleted_entries >= 4);
    assert!(!abandoned.exists() && !abandoned_lock.exists() && !scratch.exists() && !hint.exists());
    assert!(latest.path.exists() && live.exists() && unknown_hint.exists());
    assert_eq!(fs::metadata(&live_lock).unwrap().ino(), live_inode);
    assert_eq!(fs::metadata(&writer_lock).unwrap().ino(), writer_inode);
    assert_eq!(fs::read_to_string(&outside).unwrap(), "preserve");
    assert!(alias.symlink_metadata().unwrap().file_type().is_symlink());
    drop(live_lease);
    store.prune(true, false).unwrap();
    assert!(!live.exists() && !live_lock.exists());
    assert_eq!(store.prune(true, false).unwrap().deleted_entries, 0);
    // Actual process contention and killed writer at both payload and pointer
    // scratch boundaries. Reclamation cannot race an active writer.
    for point in ["before-rename", "pointer-scratch"] {
        fs::write(
            &file,
            format!("def main() -> None:\n    value: str = \"{point}\"\n"),
        )
        .unwrap();
        let marker = root.path().join(point);
        let mut child = tests::worker(&file, &cache, point, &marker);
        wait(&marker);
        assert_eq!(
            store.prune(true, false).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(store.prune(true, false).unwrap().deleted_entries > 0);
        assert_eq!(fs::metadata(&writer_lock).unwrap().ino(), writer_inode);
        assert_eq!(tests::run(&cache, &file).1.status, "published");
    }
}

#[test]
fn orphan_prune_requires_owner_and_inactivity() {
    let (root, file, cache) = tests::fixture();
    tests::run(&cache, &file);
    let workspace = file.parent().unwrap().to_path_buf();
    let store = tests::store(&cache, &file);
    let context = store.root.clone();
    let namespace = context.parent().unwrap().to_path_buf();
    let reader = store.latest().unwrap();
    let marker = root.path().join("orphan-reader");
    let mut child = tests::worker(&file, &cache, "reader", &marker);
    wait(&marker);
    drop(store); // The detached reader still owns its namespace lease.
    fs::remove_dir_all(&workspace).unwrap();
    assert_eq!(prune(&cache, &workspace, true, false).deleted_entries, 0);
    assert!(context.exists());
    drop(reader);
    assert_eq!(prune(&cache, &workspace, true, false).deleted_entries, 0);
    child.kill().unwrap();
    child.wait().unwrap();
    let owner_path = namespace.join("owner.json");
    let owner = fs::read(&owner_path).unwrap();
    fs::write(&owner_path, "{}").unwrap();
    assert_eq!(prune(&cache, &workspace, true, false).deleted_entries, 0);
    fs::write(&owner_path, &owner).unwrap();
    // A replacement directory or dangling symlink is never a deleted owner.
    fs::create_dir(&workspace).unwrap();
    assert_eq!(prune(&cache, &workspace, true, false).deleted_entries, 0);
    fs::remove_dir(&workspace).unwrap();
    symlink(root.path().join("missing"), &workspace).unwrap();
    assert_eq!(prune(&cache, &workspace, true, false).deleted_entries, 0);
    fs::remove_file(&workspace).unwrap();
    let other = root.path().join("other");
    fs::create_dir(&other).unwrap();
    let other_store =
        storage::Store::open(&cache, &other, &tests::context().identity().unwrap()).unwrap();
    let active = cache_storage::process_entry_lock(&context, "writer").unwrap();
    active.try_lock().unwrap();
    assert_eq!(prune(&cache, &workspace, true, false).deleted_entries, 0);
    drop(active);
    let foreign = root.path().join("foreign");
    fs::create_dir(&foreign).unwrap();
    fs::write(foreign.join("keep"), "untouched").unwrap();
    let nested_alias = context.join("alias");
    symlink(&foreign, &nested_alias).unwrap();
    assert_eq!(prune(&cache, &workspace, true, false).deleted_entries, 0);
    assert_eq!(
        fs::read_to_string(foreign.join("keep")).unwrap(),
        "untouched"
    );
    fs::remove_file(&nested_alias).unwrap();
    let before = fs::metadata(
        namespace
            .parent()
            .unwrap()
            .join(".locks")
            .join(namespace.file_name().unwrap()),
    )
    .unwrap()
    .ino();
    let dry = prune(&cache, &workspace, true, true);
    assert_eq!(
        (
            dry.examined_entries,
            dry.eligible_entries,
            dry.deleted_entries
        ),
        (1, 1, 0)
    );
    assert!(context.exists());
    let removed = prune(&cache, &workspace, true, false);
    assert_eq!(removed.deleted_entries, 1);
    assert!(!context.exists() && other_store.root.exists() && owner_path.exists());
    assert_eq!(prune(&cache, &workspace, true, false).deleted_entries, 0);
    assert_eq!(
        fs::metadata(
            namespace
                .parent()
                .unwrap()
                .join(".locks")
                .join(namespace.file_name().unwrap())
        )
        .unwrap()
        .ino(),
        before
    );
}

#[test]
fn pressure_prune_counts_are_truthful() {
    let (_root, file, cache) = tests::fixture();
    assert_eq!(
        prune(&cache, file.parent().unwrap(), false, false).examined_entries,
        0
    );
    assert!(!cache.exists());
    let empty = tests::store(&cache, &file);
    assert_eq!(
        prune(&cache, file.parent().unwrap(), true, true).deleted_entries,
        0
    );
    assert!(!empty.root.join(".locks").exists());
    drop(empty);
    tests::run(&cache, &file);
    let store = tests::store(&cache, &file);
    // Remove publication's now-obsolete stage lock before the counted fixture.
    store.prune(true, false).unwrap();
    let (path, lease) = stage(&store, "30-40");
    drop(lease);
    let unsafe_path = store.root.join("latest.stage-30-41");
    fs::write(&unsafe_path, "ambiguous").unwrap();
    fs::set_permissions(&unsafe_path, fs::Permissions::from_mode(0o666)).unwrap();
    let dry = store.prune(true, true).unwrap();
    assert_eq!(
        (
            dry.examined_entries,
            dry.eligible_entries,
            dry.deleted_entries
        ),
        (4, 2, 0)
    );
    assert_eq!(dry.eligible_generations, 0);
    assert!(path.exists());
    assert_eq!(store.prune(false, false).unwrap().examined_entries, 0);
    let actual = store.prune(true, false).unwrap();
    assert_eq!(
        (
            actual.examined_entries,
            actual.eligible_entries,
            actual.deleted_entries
        ),
        (4, 2, 2)
    );
    assert!(unsafe_path.exists());
    let repeat = store.prune(true, false).unwrap();
    assert_eq!(
        (
            repeat.examined_entries,
            repeat.eligible_entries,
            repeat.deleted_entries
        ),
        (2, 0, 0)
    );
}
