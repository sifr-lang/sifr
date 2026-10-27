use super::unix::stage;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::{fs::PermissionsExt, net::UnixStream, process::CommandExt};
use std::process::{Command, ExitStatus};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Hold an actual inherited descriptor across a synchronized fork/exec gap.
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

fn names(root: &std::path::Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn inherited_writer_blocks_publication_until_real_exec() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("program");
    fs::write(&destination, b"old publication").unwrap();
    let staged = stage(std::path::Path::new("/bin/true"), &destination).unwrap();
    let mut child = PausedChild::start();
    let mut observed_block = false;
    staged
        .publish(
            &destination,
            Duration::from_secs(10),
            || Ok(()),
            || {
                assert_eq!(fs::read(&destination).unwrap(), b"old publication");
                observed_block = true;
                child.release().unwrap();
            },
        )
        .unwrap();
    assert!(
        observed_block,
        "the inherited writer must prevent publication"
    );
    assert_eq!(names(root.path()), ["program"]);
    assert_eq!(
        fs::read(&destination).unwrap(),
        fs::read("/bin/true").unwrap()
    );
    assert_eq!(
        fs::metadata(&destination).unwrap().permissions().mode(),
        fs::metadata("/bin/true").unwrap().permissions().mode()
    );
    assert!(Command::new(&destination).status().unwrap().success());
}

#[test]
fn publication_timeout_preserves_destination_and_removes_staging() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("program");
    fs::write(&destination, b"old publication").unwrap();
    let staged = stage(std::path::Path::new("/bin/true"), &destination).unwrap();
    let mut child = PausedChild::start();
    let error = staged
        .publish(&destination, Duration::ZERO, || Ok(()), || {})
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(fs::read(&destination).unwrap(), b"old publication");
    assert_eq!(names(root.path()), ["program"]);
    child.release().unwrap();
}

#[test]
fn publication_cancellation_preserves_destination_and_removes_staging() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("program");
    fs::write(&destination, b"old publication").unwrap();
    let staged = stage(std::path::Path::new("/bin/true"), &destination).unwrap();
    let mut child = PausedChild::start();
    let error = staged
        .publish(
            &destination,
            Duration::from_secs(10),
            || {
                Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "injected cancellation",
                ))
            },
            || {},
        )
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert_eq!(fs::read(&destination).unwrap(), b"old publication");
    assert_eq!(names(root.path()), ["program"]);
    child.release().unwrap();
}

#[test]
fn copy_and_rename_failures_preserve_existing_paths() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("program");
    fs::write(&destination, b"old publication").unwrap();
    assert!(super::copy(&root.path().join("absent"), &destination).is_err());
    assert!(super::copy(root.path(), &destination).is_err());
    assert_eq!(fs::read(&destination).unwrap(), b"old publication");
    fs::remove_file(&destination).unwrap();
    fs::create_dir(&destination).unwrap();
    assert!(super::copy(std::path::Path::new("/bin/true"), &destination).is_err());
    assert!(destination.is_dir());
    assert_eq!(names(root.path()), ["program"]);
}
