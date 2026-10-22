use super::*;
use std::{
    io::{Read, Write},
    os::unix::{fs::DirBuilderExt, net::UnixStream, process::CommandExt},
    path::PathBuf,
    process::{Command, ExitStatus},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "sifr-process-lease-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        File::create(path.join("lease")).unwrap();
        Self(path)
    }
    fn open(&self) -> File {
        File::options()
            .read(true)
            .write(true)
            .open(self.0.join("lease"))
            .unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A synchronized real fork/exec gap. Timeouts only bound failed controls;
/// readiness and release use socket messages rather than timing assumptions.
struct PausedChild {
    socket: UnixStream,
    spawn: Option<JoinHandle<std::io::Result<ExitStatus>>>,
}
impl PausedChild {
    #[allow(unsafe_code)]
    fn start() -> Self {
        let (parent, mut child) = UnixStream::pair().unwrap();
        parent
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        child
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut command = Command::new("/bin/true");
        // SAFETY: only fixed-size socket reads/writes occur before exec. The
        // closure does not allocate, take Rust locks, or access shared state.
        unsafe {
            command.pre_exec(move || {
                child.write_all(b"R")?;
                child.read_exact(&mut [0])?;
                Ok(())
            });
        }
        let spawn = thread::spawn(move || command.spawn()?.wait());
        let mut guard = Self {
            socket: parent,
            spawn: Some(spawn),
        };
        let mut ready = [0];
        if let Err(error) = guard.socket.read_exact(&mut ready) {
            let _ = guard.release();
            panic!("child did not reach pre-exec: {error}");
        }
        assert_eq!(ready, *b"R");
        guard
    }
    fn release(&mut self) -> io::Result<()> {
        let Some(spawn) = self.spawn.take() else {
            return Ok(());
        };
        let _ = self.socket.write_all(b"G");
        let status = spawn
            .join()
            .map_err(|_| io::Error::other("child owner thread panicked"))??;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!("owned child failed: {status}")))
        }
    }
}
impl Drop for PausedChild {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

#[test]
fn final_owner_releases_before_unrelated_child_exec() {
    for shared in [false, true] {
        let scratch = Scratch::new();
        let lease = if shared {
            ProcessLease::shared(scratch.open())
        } else {
            ProcessLease::exclusive(scratch.open())
        }
        .unwrap();
        let first = Arc::new(lease);
        let last = Arc::clone(&first);
        let mut child = PausedChild::start();
        drop(first);
        let contender = scratch.open();
        assert!(matches!(
            contender.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        drop(last);
        contender
            .try_lock()
            .expect("final parent owner must release before unrelated child exec");
        contender.unlock().unwrap();
        child.release().unwrap();
    }
}

#[test]
fn independent_readers_keep_their_own_lifetimes() {
    let scratch = Scratch::new();
    let first = ProcessLease::shared(scratch.open()).unwrap();
    let second = ProcessLease::shared(scratch.open()).unwrap();
    let contender = scratch.open();
    drop(first);
    assert!(matches!(
        contender.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    drop(second);
    contender.try_lock().unwrap();
    contender.unlock().unwrap();
}

#[test]
fn failed_admission_preserves_live_owner_exclusion() {
    let scratch = Scratch::new();
    let owner = ProcessLease::exclusive(scratch.open()).unwrap();
    assert_eq!(
        ProcessLease::exclusive(scratch.open())
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::WouldBlock
    );
    assert_eq!(
        ProcessLease::shared(scratch.open()).err().unwrap().kind(),
        io::ErrorKind::WouldBlock
    );
    let contender = scratch.open();
    assert!(matches!(
        contender.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    drop(owner);
    contender.try_lock().unwrap();
    contender.unlock().unwrap();
}

fn failed_child(mode: u8) -> PausedChild {
    let (socket, peer) = UnixStream::pair().unwrap();
    drop(peer);
    PausedChild {
        socket,
        spawn: Some(thread::spawn(move || match mode {
            0 => Err(io::Error::other("injected spawn failure")),
            1 => panic!("injected owner-thread panic"),
            2 => Command::new("/bin/false").status(),
            _ => unreachable!("test control mode"),
        })),
    }
}

#[test]
fn explicit_child_release_reports_failures_without_repeating_cleanup() {
    for mode in 0..3 {
        let mut child = failed_child(mode);
        assert!(child.release().is_err());
        assert!(child.release().is_ok());
    }
}

#[test]
fn child_cleanup_preserves_the_original_unwind() {
    for mode in 0..3 {
        let result = std::panic::catch_unwind(move || {
            let _child = failed_child(mode);
            panic!("original assertion failure");
        });
        let payload = result.unwrap_err();
        assert_eq!(
            payload.downcast_ref::<&str>(),
            Some(&"original assertion failure")
        );
    }
}
