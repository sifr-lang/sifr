use super::*;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn lock(root: &Path, key: &str) -> File {
    crate::open_entry_lock(root, key, false).unwrap()
}

fn sleeper() -> Command {
    let mut command = Command::new("sleep");
    command
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

#[allow(unsafe_code)]
fn flags(file: &File) -> i32 {
    // SAFETY: File owns a valid live descriptor.
    let result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };
    assert!(result >= 0);
    result
}

fn root() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::Builder::new()
        .permissions(<std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700))
        .tempdir()
        .unwrap();
    let path = temp.path().canonicalize().unwrap();
    (temp, path)
}

#[test]
fn unrelated_child_never_retains_an_owner_or_prepared_command_lease() {
    let (_temp, root) = root();
    let owner = lock(&root, "suite");
    owner.try_lock().unwrap();
    let mut intended = sleeper();
    inherit_lease_for_command(&mut intended, &owner).unwrap();
    let unrelated = Running(sleeper().spawn().unwrap());
    drop(owner);
    let contender = lock(&root, "suite");
    assert!(matches!(
        contender.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    drop(intended);
    contender.try_lock().unwrap();
    assert!(unrelated.0.id() > 0);
}

#[test]
fn intended_child_retains_lease_without_changing_parent_descriptor_flags() {
    let (_temp, root) = root();
    let owner = lock(&root, "suite");
    owner.try_lock().unwrap();
    let before = flags(&owner);
    assert_ne!(before & libc::FD_CLOEXEC, 0);
    let mut command = sleeper();
    inherit_lease_for_command(&mut command, &owner).unwrap();
    assert_eq!(flags(&owner), before);
    let intended = Running(command.spawn().unwrap());
    assert_eq!(flags(&owner), before);
    let unrelated = Running(sleeper().spawn().unwrap());
    drop(command);
    drop(owner);
    let contender = lock(&root, "suite");
    assert!(matches!(
        contender.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    drop(intended);
    contender.try_lock().unwrap();
    drop(unrelated);
}

#[test]
fn parallel_owners_pass_only_their_own_lease() {
    let (_temp, root) = root();
    let first = lock(&root, "first");
    let second = lock(&root, "second");
    first.try_lock().unwrap();
    second.try_lock().unwrap();
    let spawn = |lease: &File| {
        let mut command = sleeper();
        inherit_lease_for_command(&mut command, lease).unwrap();
        Running(command.spawn().unwrap())
    };
    let (first_child, second_child) = std::thread::scope(|scope| {
        let a = scope.spawn(|| spawn(&first));
        let b = scope.spawn(|| spawn(&second));
        (a.join().unwrap(), b.join().unwrap())
    });
    drop(first);
    drop(second);
    let first_waiter = lock(&root, "first");
    let second_waiter = lock(&root, "second");
    assert!(first_waiter.try_lock().is_err());
    assert!(second_waiter.try_lock().is_err());
    drop(first_child);
    first_waiter.try_lock().unwrap();
    assert!(matches!(
        second_waiter.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    drop(second_child);
    second_waiter.try_lock().unwrap();
}

#[test]
fn failed_spawn_and_unused_command_release_owned_descriptors_on_drop() {
    let (_temp, root) = root();
    for should_spawn in [false, true] {
        let owner = lock(&root, "suite");
        owner.try_lock().unwrap();
        let mut command = Command::new(root.join("absent-program"));
        inherit_lease_for_command(&mut command, &owner).unwrap();
        drop(owner);
        if should_spawn {
            assert_eq!(command.spawn().unwrap_err().kind(), io::ErrorKind::NotFound);
        }
        let waiter = lock(&root, "suite");
        assert!(matches!(
            waiter.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        drop(command);
        assert_released(&waiter);
    }
}

#[test]
fn command_reuse_and_multiple_leases_keep_each_live_consumer_protected() {
    let (_temp, root) = root();
    let owners = [lock(&root, "first"), lock(&root, "second")];
    let mut command = sleeper();
    for owner in &owners {
        owner.try_lock().unwrap();
        inherit_lease_for_command(&mut command, owner).unwrap();
    }
    let first = Running(command.spawn().unwrap());
    let second = Running(command.spawn().unwrap());
    drop(command);
    drop(owners);
    let waiters = [lock(&root, "first"), lock(&root, "second")];
    drop(first);
    for waiter in &waiters {
        assert!(matches!(
            waiter.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
    }
    drop(second);
    for waiter in &waiters {
        waiter.try_lock().unwrap();
    }
}

#[test]
fn killed_owner_helper() {
    let Some(root) = std::env::var_os("SIFR_CHILD_LEASE_TEST_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let owner = lock(&root, "suite");
    owner.try_lock().unwrap();
    let mut command = sleeper();
    inherit_lease_for_command(&mut command, &owner).unwrap();
    let mut child = Running(command.spawn().unwrap());
    std::fs::write(root.join("child.pid.stage"), child.0.id().to_string()).unwrap();
    std::fs::rename(root.join("child.pid.stage"), root.join("child.pid")).unwrap();
    child.0.wait().unwrap();
}

struct Orphan(u32);
impl Drop for Orphan {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: PID was returned by our helper's spawn and has a bounded life.
        unsafe {
            libc::kill(i32::try_from(self.0).unwrap(), libc::SIGKILL);
        }
    }
}

#[test]
fn killing_the_owner_does_not_release_a_live_childs_lease() {
    let (_temp, root) = root();
    let mut owner = Running(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "child_lease::tests::killed_owner_helper",
                "--nocapture",
            ])
            .env("SIFR_CHILD_LEASE_TEST_ROOT", &root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let ready = root.join("child.pid");
    let deadline = Instant::now() + Duration::from_secs(10);
    let child = loop {
        if let Ok(pid) = std::fs::read_to_string(&ready) {
            if let Ok(pid) = pid.parse() {
                break Orphan(pid);
            }
        }
        assert!(
            owner.0.try_wait().unwrap().is_none(),
            "owner exited before readiness"
        );
        assert!(Instant::now() < deadline, "owner readiness deadline");
        std::thread::sleep(Duration::from_millis(10));
    };
    owner.0.kill().unwrap();
    owner.0.wait().unwrap();
    let waiter = lock(&root, "suite");
    assert!(matches!(
        waiter.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    drop(child);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match waiter.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) => {
                assert!(Instant::now() < deadline, "child lease did not release");
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("unexpected lock error: {error}"),
        }
    }
}

// A concurrent fork briefly copies CLOEXEC descriptors before its exec closes
// them. Bound that transition; a persistent inherited sleep lease lasts 30s and
// must still fail this check. No test worker or child creation is serialized.
fn assert_released(file: &File) {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match file.try_lock() {
            Ok(()) => return,
            Err(std::fs::TryLockError::WouldBlock) => {
                assert!(Instant::now() < deadline, "descriptor remained leased");
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("unexpected lock error: {error}"),
        }
    }
}
