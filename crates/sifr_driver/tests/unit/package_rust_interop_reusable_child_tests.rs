use super::*;
use std::io::Read;
use std::os::fd::OwnedFd;
use std::os::unix::{net::UnixStream, process::CommandExt};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

const CONTROL: &str = "SIFR_ADVANCED_FIXTURE_CHILD_CONTROL";
const ROLE: &str = "SIFR_ADVANCED_FIXTURE_CHILD_ROLE";
const STOP_AFTER: &str = "SIFR_ADVANCED_FIXTURE_STOP_AFTER";

struct OwnedChildren {
    owner: Child,
    owner_exit: UnixStream,
    group_exit: UnixStream,
}

impl OwnedChildren {
    fn spawn(root: &Path, stop_after: Option<usize>) -> Self {
        let (owner_exit, owner_input) = UnixStream::pair().unwrap();
        let (group_exit, group_output) = UnixStream::pair().unwrap();
        for stream in [&owner_exit, &group_exit] {
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
        }
        let mut command = control_command(root, "owner");
        command
            .process_group(0)
            .stdin(Stdio::from(OwnedFd::from(owner_input)))
            .stdout(Stdio::from(OwnedFd::from(group_output)));
        if let Some(count) = stop_after {
            command.env(STOP_AFTER, count.to_string());
        } else {
            command.env_remove(STOP_AFTER);
        }
        // spawn establishes the group before returning; no readiness operation
        // can fail between creating the process and installing its owner guard.
        let owner = command.spawn().unwrap();
        Self {
            owner,
            owner_exit,
            group_exit,
        }
    }

    fn kill_owner_without_reaping(&mut self) {
        self.owner.kill().unwrap();
        // Descendants replace stdin, so only the owner holds this endpoint.
        // EOF proves owner death while its unreaped PID still reserves the group
        // identity. Only Drop may wait/reap it, after signalling that group.
        assert_eq!(self.owner_exit.read(&mut [0]).unwrap(), 0);
    }
}

impl Drop for OwnedChildren {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: spawn created this dedicated group, all children inherit it,
        // and the owner PID has never been reaped. Its identity cannot be reused
        // before this signal, even if readiness failed or the owner has exited.
        unsafe {
            libc::kill(-(self.owner.id() as i32), libc::SIGKILL);
        }
        let _ = self.owner.wait();
    }
}

fn control_command(root: &Path, role: &str) -> Command {
    let name = format!(
        "{}::intended_fixture_child_survives_parent_death_without_unrelated_inheritance",
        module_path!().strip_prefix("sifr_driver::").unwrap()
    );
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([&name, "--exact", "--nocapture", "--test-threads=1"])
        .env(CONTROL, root)
        .env(ROLE, role);
    command
}

fn wait_ready(path: &Path) {
    let start = Instant::now();
    while !path.exists() {
        // Do not try_wait here: reaping the owner would release the group ID
        // before the guard can use it to clean up a readiness failure.
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "control child readiness timed out"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn remain_alive() -> ! {
    loop {
        std::thread::park();
    }
}

fn child_role(path: &Path) -> ! {
    let role = std::env::var(ROLE).unwrap();
    if role == "bound" || role == "unrelated" {
        fs::write(path.join(format!("{role}-ready")), "ready").unwrap();
        remain_alive();
    }
    assert_eq!(role, "owner");
    let fixture = acquire(&path.join("cache"), path, inputs()).unwrap();
    fs::write(
        path.join("fixture-root"),
        fixture.root.as_os_str().as_encoded_bytes(),
    )
    .unwrap();
    let stop_after = std::env::var(STOP_AFTER)
        .ok()
        .map(|value| value.parse::<usize>().unwrap());
    fs::write(path.join("owner-started"), "ready").unwrap();
    if stop_after == Some(0) {
        remain_alive();
    }
    for (index, (label, intended)) in [("bound", true), ("unrelated", false)]
        .into_iter()
        .enumerate()
    {
        let marker = path.join(format!("{label}-ready"));
        let mut command = control_command(path, label);
        // Keep the dedicated owner group and its output-lifetime channel, but
        // close the owner-only death channel in every descendant before exec.
        command.stdin(Stdio::null()).stdout(Stdio::inherit());
        if intended {
            fixture.leases().bind(&mut command).unwrap();
        }
        let child = command.spawn().unwrap();
        fs::write(path.join(format!("{label}-pid")), child.id().to_string()).unwrap();
        wait_ready(&marker);
        if stop_after == Some(index + 1) {
            remain_alive();
        }
    }
    fs::write(path.join("owner-ready"), "ready").unwrap();
    remain_alive();
}

#[test]
fn intended_fixture_child_survives_parent_death_without_unrelated_inheritance() {
    if let Some(path) = std::env::var_os(CONTROL) {
        child_role(&PathBuf::from(path));
    }
    let temp = private_root();
    let mut children = OwnedChildren::spawn(temp.path(), None);
    wait_ready(&temp.path().join("owner-ready"));
    let child_pid = |label| {
        fs::read_to_string(temp.path().join(format!("{label}-pid")))
            .unwrap()
            .parse::<i32>()
            .unwrap()
    };
    let bound = child_pid("bound");
    let unrelated = child_pid("unrelated");
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
    children.kill_owner_without_reaping();
    assert!(
        matches!(contender.try_lock(), Err(fs::TryLockError::WouldBlock)),
        "intended child must retain actual fixture owner after parent death"
    );
    #[allow(unsafe_code)]
    // SAFETY: this ready child belongs to our unreaped owner's dedicated group
    // and remains parked until explicitly killed; it has no expiry/reap path.
    unsafe {
        assert_eq!(libc::kill(bound, libc::SIGKILL), 0);
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
    // SAFETY: signal zero observes the acknowledged, still-parked unrelated child.
    unsafe {
        assert_eq!(libc::kill(unrelated, 0), 0);
    }
    assert_eq!(read_inputs(&root).unwrap(), inputs());
}

#[test]
fn owned_children_are_cleaned_up_when_readiness_unwinds() {
    for count in 0..=2 {
        let temp = private_root();
        let children = OwnedChildren::spawn(temp.path(), Some(count));
        let marker = ["owner-started", "bound-ready", "unrelated-ready"][count];
        wait_ready(&temp.path().join(marker));
        assert!(!temp.path().join("owner-ready").exists());
        let mut group_exit = children.group_exit.try_clone().unwrap();
        let result = std::panic::catch_unwind(move || {
            let _children = children;
            panic!("injected readiness failure");
        });
        assert!(result.is_err());
        // Every child inherits stdout and keeps it open while parked. EOF is
        // process-lifetime evidence, not merely a missing readiness marker.
        let mut output = Vec::new();
        group_exit.read_to_end(&mut output).unwrap();
        let root = PathBuf::from(fs::read_to_string(temp.path().join("fixture-root")).unwrap());
        let contender = crate::build::native_child_leases::open(
            root.parent().unwrap(),
            root.file_name().unwrap().to_str().unwrap(),
        )
        .unwrap();
        contender.try_lock().unwrap();
    }
}
