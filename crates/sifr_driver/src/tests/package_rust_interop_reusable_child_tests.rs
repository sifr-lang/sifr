use super::*;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

struct OwnedChildren {
    owner: Child,
    groups: Vec<i32>,
}
impl Drop for OwnedChildren {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let _ = self.owner.kill();
        let _ = self.owner.wait();
        for group in &self.groups {
            // SAFETY: these groups were created only by this control's owner.
            unsafe {
                libc::kill(-*group, libc::SIGKILL);
            }
        }
    }
}

fn wait_ready(path: &Path, child: &mut Child) {
    let start = Instant::now();
    while !path.exists() {
        assert!(
            child.try_wait().unwrap().is_none(),
            "control child exited before readiness"
        );
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "control child readiness timed out"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn intended_fixture_child_survives_parent_death_without_unrelated_inheritance() {
    const ENV: &str = "SIFR_ADVANCED_FIXTURE_CHILD_CONTROL";
    if let Some(path) = std::env::var_os(ENV) {
        let path = PathBuf::from(path);
        let fixture = acquire(&path.join("cache"), &path, inputs()).unwrap();
        fs::write(
            path.join("fixture-root"),
            fixture.root.as_os_str().as_encoded_bytes(),
        )
        .unwrap();
        for (label, intended) in [("bound", true), ("unrelated", false)] {
            let marker = path.join(format!("{label}-ready"));
            let mut command = Command::new("sh");
            command
                .args([
                    "-c",
                    "printf ready > \"$1\"; exec sleep 60",
                    "fixture-child",
                ])
                .arg(&marker)
                .process_group(0)
                .stdin(Stdio::null())
                .stdout(Stdio::null());
            if intended {
                fixture.leases().bind(&mut command).unwrap();
            }
            let mut child = command.spawn().unwrap();
            fs::write(path.join(format!("{label}-pid")), child.id().to_string()).unwrap();
            wait_ready(&marker, &mut child);
        }
        fs::write(path.join("owner-ready"), "ready").unwrap();
        loop {
            std::thread::park();
        }
    }
    let temp = private_root();
    let name = format!(
        "{}::intended_fixture_child_survives_parent_death_without_unrelated_inheritance",
        module_path!().strip_prefix("sifr_driver::").unwrap()
    );
    let owner = Command::new(std::env::current_exe().unwrap())
        .args([&name, "--exact", "--nocapture", "--test-threads=1"])
        .env(ENV, temp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let mut children = OwnedChildren {
        owner,
        groups: Vec::new(),
    };
    wait_ready(&temp.path().join("owner-ready"), &mut children.owner);
    for label in ["bound", "unrelated"] {
        children.groups.push(
            fs::read_to_string(temp.path().join(format!("{label}-pid")))
                .unwrap()
                .parse()
                .unwrap(),
        );
    }
    let root = PathBuf::from(fs::read_to_string(temp.path().join("fixture-root")).unwrap());
    let contender = crate::build::native_child_leases::open(
        root.parent().unwrap(),
        root.file_name().unwrap().to_str().unwrap(),
    )
    .unwrap();
    assert!(matches!(
        contender.try_lock(),
        Err(fs::TryLockError::WouldBlock)
    ));
    children.owner.kill().unwrap();
    children.owner.wait().unwrap();
    assert!(
        matches!(contender.try_lock(), Err(fs::TryLockError::WouldBlock)),
        "intended child must retain actual fixture owner after parent death"
    );
    let bound = children.groups.remove(0);
    #[allow(unsafe_code)]
    // SAFETY: the test-created bound child has acknowledged readiness in this group.
    unsafe {
        assert_eq!(libc::kill(-bound, libc::SIGKILL), 0);
    }
    let start = Instant::now();
    loop {
        match contender.try_lock() {
            Ok(()) => break,
            Err(fs::TryLockError::WouldBlock) if start.elapsed() < Duration::from_secs(5) => {
                std::thread::sleep(Duration::from_millis(5))
            }
            error => panic!("unrelated child retained fixture owner: {error:?}"),
        }
    }
    #[allow(unsafe_code)]
    // SAFETY: signal zero checks this control's still-owned unrelated child group.
    unsafe {
        assert_eq!(libc::kill(-children.groups[0], 0), 0);
    }
    assert_eq!(read_inputs(&root).unwrap(), inputs());
}
