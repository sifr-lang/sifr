use super::*;
use std::{
    io::Write,
    process::{Child, Command, Stdio},
    time::Duration,
};
fn until(mut predicate: impl FnMut() -> bool) {
    let start = Instant::now();
    while !predicate() {
        assert!(
            start.elapsed() < Duration::from_secs(120),
            "DX.6 subprocess safety deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
struct ChildRun {
    child: Child,
    report: PathBuf,
}
impl ChildRun {
    fn spawn(scratch: &Scratch, cache: &Path, configuration: &str, name: &str, mode: &str) -> Self {
        let report = scratch.0.join(format!("{name}.json"));
        let log = fs::File::create(report.with_extension("log")).unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "metadata_producer::tests::processes::dx6_process_child",
                "--ignored",
                "--nocapture",
            ])
            .env("DX6_CHILD_CACHE", cache)
            .env("DX6_CHILD_SOURCE", root())
            .env("DX6_CHILD_CONFIGURATION", configuration)
            .env("DX6_CHILD_REPORT", &report)
            .env("DX6_CHILD_MODE", mode)
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap();
        Self { child, report }
    }
    fn event(&self, event: &str) -> bool {
        fs::read_to_string(self.report.with_extension("events"))
            .is_ok_and(|text| text.lines().any(|line| line == event))
    }
    fn finish(&mut self) -> serde_json::Value {
        let mut status = None;
        until(|| {
            status = self.child.try_wait().unwrap();
            status.is_some()
        });
        assert!(
            status.unwrap().success(),
            "{}",
            fs::read_to_string(self.report.with_extension("log")).unwrap()
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&self.report).unwrap()).unwrap();
        assert_eq!(report["status"], "assertions-passed");
        report
    }
}
impl Drop for ChildRun {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
#[test]
#[ignore = "subprocess protocol for DX.6 producer concurrency and ownership tests"]
fn dx6_process_child() {
    // This ignored entry is a subprocess protocol, not a standalone acceptance case.
    if std::env::var_os("DX6_CHILD_REPORT").is_none() {
        return;
    }
    let cache = PathBuf::from(std::env::var_os("DX6_CHILD_CACHE").unwrap());
    let source = PathBuf::from(std::env::var_os("DX6_CHILD_SOURCE").unwrap());
    let configuration = std::env::var("DX6_CHILD_CONFIGURATION").unwrap();
    let report = PathBuf::from(std::env::var_os("DX6_CHILD_REPORT").unwrap());
    let mode = std::env::var("DX6_CHILD_MODE").unwrap();
    let id = identity(&configuration);
    let seen = AtomicUsize::new(0);
    let started = Instant::now();
    let metadata = ensure_with_hook(
        &id,
        &source,
        TARGET,
        &cache,
        &AtomicBool::new(false),
        |stage| {
            let (bit, name) = match stage {
                Stage::Captured => (1, "captured"),
                Stage::Owned => (2, "owned"),
                Stage::Waiting => (4, "waiting"),
                Stage::Staged => (8, "staged"),
                Stage::Published => (16, "published"),
            };
            if seen.fetch_or(bit, Ordering::SeqCst) & bit == 0 {
                let mut events = fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(report.with_extension("events"))
                    .unwrap();
                writeln!(events, "{name}").unwrap();
                events.sync_all().unwrap();
            }
            if stage == Stage::Staged && mode == "pause-staged" {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
            Ok(())
        },
    )
    .unwrap();
    let preparation_seconds = started.elapsed().as_secs_f64();
    assertion(&metadata, &id, &source);
    fs::write(report,serde_json::to_vec(&serde_json::json!({"status":"assertions-passed","compiler_identity":id.as_str(),"metadata_id":metadata.metadata_id,"path":metadata.path,"production_seconds":metadata.production_seconds,"preparation_seconds":preparation_seconds,"total_seconds":started.elapsed().as_secs_f64()})).unwrap()).unwrap();
}
#[test]
fn process_waiters_warm_reuse_and_distinct_configurations() {
    let scratch = Scratch::new();
    let cache = scratch.0.join("cache");
    let started = Instant::now();
    let mut children = (0..4)
        .map(|i| {
            ChildRun::spawn(
                &scratch,
                &cache,
                "dx6-process-family-a",
                &format!("cold-{i}"),
                "normal",
            )
        })
        .collect::<Vec<_>>();
    let cold = children
        .iter_mut()
        .map(ChildRun::finish)
        .collect::<Vec<_>>();
    assert_eq!(
        cold.iter()
            .filter(|v| !v["production_seconds"].is_null())
            .count(),
        1
    );
    assert!(
        cold.iter()
            .all(|v| v["metadata_id"] == cold[0]["metadata_id"])
    );
    let mut children = (0..4)
        .map(|i| {
            ChildRun::spawn(
                &scratch,
                &cache,
                "dx6-process-family-a",
                &format!("warm-{i}"),
                "normal",
            )
        })
        .collect::<Vec<_>>();
    let warm = children
        .iter_mut()
        .map(ChildRun::finish)
        .collect::<Vec<_>>();
    assert!(warm.iter().all(|v| v["production_seconds"].is_null()));
    let mut a = ChildRun::spawn(
        &scratch,
        &cache,
        "dx6-process-family-b",
        "distinct-b",
        "normal",
    );
    let mut b = ChildRun::spawn(
        &scratch,
        &cache,
        "dx6-process-family-c",
        "distinct-c",
        "normal",
    );
    let a = a.finish();
    let b = b.finish();
    assert_ne!(a["path"], b["path"]);
    assert_ne!(a["compiler_identity"], b["compiler_identity"]);
    assert!(!a["production_seconds"].is_null() && !b["production_seconds"].is_null());
    writeln!(
        std::io::stderr(),
        "DX6 process evidence {}",
        serde_json::json!({"cold":cold,"warm":warm,"distinct":[a,b],"assertions":10,"seconds":started.elapsed().as_secs_f64()})
    )
    .unwrap();
}
#[test]
fn killed_staged_producer_releases_waiters() {
    let scratch = Scratch::new();
    let cache = scratch.0.join("cache");
    let mut victim = ChildRun::spawn(
        &scratch,
        &cache,
        "dx6-process-death",
        "victim",
        "pause-staged",
    );
    until(|| victim.event("staged"));
    let mut waiters = (0..2)
        .map(|i| {
            ChildRun::spawn(
                &scratch,
                &cache,
                "dx6-process-death",
                &format!("waiter-{i}"),
                "normal",
            )
        })
        .collect::<Vec<_>>();
    until(|| waiters.iter().all(|waiter| waiter.event("waiting")));
    victim.child.kill().unwrap();
    assert!(!victim.child.wait().unwrap().success());
    let reports = waiters.iter_mut().map(ChildRun::finish).collect::<Vec<_>>();
    assert_eq!(
        reports
            .iter()
            .filter(|v| !v["production_seconds"].is_null())
            .count(),
        1
    );
    assert!(!fs::read_dir(cache.join("metadata")).unwrap().any(|entry| {
        entry
            .unwrap()
            .path()
            .extension()
            .is_some_and(|v| v == "stage")
    }));
}
#[test]
fn wait_cancellation_and_cancelled_producer_retry() {
    let scratch = Scratch::new();
    let source = root();
    let cache = scratch.0.join("cache");
    let id = identity("dx6-cancel");
    let owned = AtomicBool::new(false);
    let release = AtomicBool::new(false);
    let waiting = AtomicBool::new(false);
    let cancel = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let owner = scope.spawn(|| {
            ensure_with_hook(
                &id,
                &source,
                TARGET,
                &cache,
                &AtomicBool::new(false),
                |stage| {
                    if stage == Stage::Owned {
                        owned.store(true, Ordering::SeqCst);
                        until(|| release.load(Ordering::SeqCst));
                    }
                    Ok(())
                },
            )
        });
        until(|| owned.load(Ordering::SeqCst));
        let waiter = scope.spawn(|| {
            ensure_with_hook(&id, &source, TARGET, &cache, &cancel, |stage| {
                if stage == Stage::Waiting {
                    waiting.store(true, Ordering::SeqCst);
                }
                Ok(())
            })
        });
        until(|| waiting.load(Ordering::SeqCst));
        cancel.store(true, Ordering::SeqCst);
        let result = waiter.join().unwrap();
        release.store(true, Ordering::SeqCst);
        assert!(result.err().unwrap().0.contains("cancelled"));
        assert!(owner.join().unwrap().is_ok());
    });
    let cache = scratch.0.join("cancelled-producer");
    let cancel = AtomicBool::new(false);
    let result = ensure_with_hook(&id, &source, TARGET, &cache, &cancel, |stage| {
        if stage == Stage::Staged {
            cancel.store(true, Ordering::SeqCst);
        }
        Ok(())
    });
    assert!(result.err().unwrap().0.contains("cancelled"));
    assert!(
        ensure_development_metadata(&id, &source, TARGET, &cache, &AtomicBool::new(false)).is_ok()
    );
}

#[test]
fn failed_producer_releases_all_waiters() {
    let scratch = Scratch::new();
    let source = root();
    let cache = scratch.0.join("cache");
    let id = identity("dx6-failed-owner");
    let staged = AtomicBool::new(false);
    let waiting = AtomicUsize::new(0);
    let published = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let producer = scope.spawn(|| {
            ensure_with_hook(
                &id,
                &source,
                TARGET,
                &cache,
                &AtomicBool::new(false),
                |stage| {
                    if stage == Stage::Staged {
                        staged.store(true, Ordering::SeqCst);
                        until(|| waiting.load(Ordering::SeqCst) == 2);
                        return Err(wire::MetadataError(
                            "original staged producer failure".into(),
                        ));
                    }
                    Ok(())
                },
            )
        });
        until(|| staged.load(Ordering::SeqCst));
        let handles = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    let seen = AtomicBool::new(false);
                    let metadata = ensure_with_hook(
                        &id,
                        &source,
                        TARGET,
                        &cache,
                        &AtomicBool::new(false),
                        |stage| {
                            if stage == Stage::Waiting && !seen.swap(true, Ordering::SeqCst) {
                                waiting.fetch_add(1, Ordering::SeqCst);
                            }
                            if stage == Stage::Published {
                                published.fetch_add(1, Ordering::SeqCst);
                            }
                            Ok(())
                        },
                    )
                    .unwrap();
                    assertion(&metadata, &id, &source);
                    metadata
                })
            })
            .collect::<Vec<_>>();
        assert!(
            producer
                .join()
                .unwrap()
                .err()
                .unwrap()
                .0
                .contains("original staged producer failure")
        );
        let results = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>();
        assert!(Arc::ptr_eq(&results[0], &results[1]));
    });
    assert_eq!(published.load(Ordering::SeqCst), 1);
}
