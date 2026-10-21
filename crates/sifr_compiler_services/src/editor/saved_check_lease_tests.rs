//! Exercise the saved-reader owner, including the actual fork-before-exec gap.
use super::*;
use sifr_frontend::{DiskSourceProvider, SourceProvider, persistence::CapturingSourceProvider};
use std::{
    io::Write,
    os::unix::{fs::OpenOptionsExt, net::UnixStream, process::CommandExt},
    process::Command,
    thread::{self, JoinHandle},
    time::Duration,
};

fn write_private(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?
        .write_all(bytes.as_ref())
}

struct Fixture {
    _temp: tempfile::TempDir,
    cache: PathBuf,
    workspace: PathBuf,
    context: String,
    namespace: PathBuf,
    generation: PathBuf,
    workspace_id: String,
    generation_id: String,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        sifr_cache_storage::directory(&workspace).unwrap();
        let workspace = workspace.canonicalize().unwrap();
        let source = workspace.join("main.sifr");
        write_private(&source, "def main() -> None:\n    pass\n").unwrap();
        let inputs =
            manifestless_inputs(&CompilerContext::for_test(), "metadata-one", &source).unwrap();
        let context = inputs.identity().unwrap();
        let mut disk = DiskSourceProvider::new();
        let mut capture = CapturingSourceProvider::new(&mut disk);
        capture.read_file(&source).unwrap();
        let record = CompletedCheck::capture(&source, inputs, &capture, &[]).unwrap();
        let bytes = serde_json::to_vec(&record).unwrap();
        let record_id = stamp("project-record-v1", &bytes).unwrap();
        let workspace_id = stamp("project-workspace-v1", &workspace).unwrap();
        let manifest = Manifest {
            schema: 2,
            workspace: workspace_id.clone(),
            context: context.clone(),
            records: vec![record_id.clone()],
        };
        let generation_id = stamp("project-generation-v1", &manifest).unwrap();
        let cache = temp.path().join("cache");
        let projects = cache.join("projects");
        let namespace = projects.join(&workspace_id);
        let root = namespace.join(&context);
        let generations = root.join("generations");
        let generation = generations.join(&generation_id);
        sifr_cache_storage::directory(&generation).unwrap();
        drop(sifr_cache_storage::open_entry_lock(&projects, &workspace_id, false).unwrap());
        drop(sifr_cache_storage::open_entry_lock(&generations, &generation_id, false).unwrap());
        let (device, inode) = file_identity(&workspace).unwrap();
        let created_ns = fs::metadata(&workspace)
            .unwrap()
            .created()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        write_private(
            namespace.join("owner.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema": 1, "workspace": workspace, "device": device,
                "inode": inode, "created_ns": created_ns,
            }))
            .unwrap(),
        )
        .unwrap();
        write_private(generation.join(record_id), bytes).unwrap();
        write_private(
            generation.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        write_private(
            root.join("latest"),
            serde_json::to_vec(&serde_json::json!({
                "schema": 1, "workspace": workspace_id,
                "context": context, "generation": generation_id,
            }))
            .unwrap(),
        )
        .unwrap();
        Self {
            _temp: temp,
            cache,
            workspace,
            context,
            namespace,
            generation,
            workspace_id,
            generation_id,
        }
    }

    fn load(&self) -> io::Result<Generation> {
        load_generation(&self.cache, &self.workspace, &self.context)
    }

    fn locks(&self) -> [File; 2] {
        [
            sifr_cache_storage::open_entry_lock(
                &self.cache.join("projects"),
                &self.workspace_id,
                false,
            )
            .unwrap(),
            sifr_cache_storage::open_entry_lock(
                self.generation.parent().unwrap(),
                &self.generation_id,
                false,
            )
            .unwrap(),
        ]
    }

    fn assert_available(&self) {
        for (name, lease) in ["namespace", "generation"].into_iter().zip(self.locks()) {
            let acquired = lease.try_lock();
            assert!(acquired.is_ok(), "{name} remains locked: {acquired:?}");
            lease.unlock().unwrap();
        }
    }
}

struct PausedChild {
    socket: UnixStream,
    spawn: Option<JoinHandle<io::Result<std::process::ExitStatus>>>,
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
        // SAFETY: fixed-size socket I/O is the only work before exec. No
        // allocation, shared Rust lock, or mutable process-global state is used.
        unsafe {
            command.pre_exec(move || {
                child.write_all(b"R")?;
                child.read_exact(&mut [0])?;
                Ok(())
            });
        }
        let spawn = thread::spawn(move || command.spawn()?.wait());
        let mut paused = Self {
            socket: parent,
            spawn: Some(spawn),
        };
        let mut ready = [0];
        if let Err(error) = paused.socket.read_exact(&mut ready) {
            let _ = paused.finish();
            panic!("child did not reach pre-exec: {error}");
        }
        assert_eq!(ready, *b"R");
        paused
    }

    fn finish(&mut self) -> io::Result<()> {
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
        // Reap even while a failing assertion unwinds; never double-panic.
        let _ = self.finish();
    }
}

#[test]
fn saved_reader_releases_both_leases_before_unrelated_child_exec() {
    let fixture = Fixture::new();
    let reader = fixture.load().unwrap();
    assert_eq!(reader.records.len(), 1);
    for lease in fixture.locks() {
        assert!(matches!(
            lease.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
    }
    let mut child = PausedChild::start();
    drop(reader);
    fixture.assert_available();
    child.finish().unwrap();
}

#[test]
fn rejected_saved_reader_releases_every_acquired_lease() {
    for invalid_owner in [true, false] {
        let fixture = Fixture::new();
        drop(fixture.load().unwrap());
        let path = if invalid_owner {
            fixture.namespace.join("owner.json")
        } else {
            fixture.generation.join("manifest.json")
        };
        write_private(path, b"{}").unwrap();
        assert!(fixture.load().is_err());
        fixture.assert_available();
    }
}
