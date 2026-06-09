use crate::stdlib_filter::strip_rust_items_by_name;
use std::collections::HashSet;

pub(crate) fn replace_parallel_runtime_items(rust_code: &str) -> String {
    let strip_names = HashSet::from([
        "WorkerRuntimeError",
        "WorkerError",
        "PoolConfig",
        "Pool",
        "map",
        "try_map",
    ]);
    let mut replaced = strip_rust_items_by_name(rust_code, &strip_names);
    if !replaced.trim().is_empty() {
        replaced.push('\n');
    }
    replaced.push_str(parallel_runtime_rust_code());
    replaced
}

pub(crate) fn parallel_runtime_rust_code() -> &'static str {
    r#"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PoolConfig {
    workers: i64,
}

impl PoolConfig {
    fn new(workers: i64) -> Self {
        return Self { workers };
    }
}

impl std::fmt::Display for PoolConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PoolConfig(workers={})", self.workers)
    }
}

struct Pool {
    _pool: Option<rayon::ThreadPool>,
    _failure: Option<WorkerRuntimeError>,
    _workers: usize,
}

impl Pool {
    fn new(config: PoolConfig) -> Self {
        let workers = __sifr_parallel_worker_count(config.workers);
        match __sifr_build_parallel_pool(workers) {
            Ok(pool) => {
                return Self {
                    _pool: Some(pool),
                    _failure: None,
                    _workers: workers,
                };
            }
            Err(error) => {
                return Self {
                    _pool: None,
                    _failure: Some(error),
                    _workers: workers,
                };
            }
        }
    }
}

impl std::fmt::Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pool")
            .field("workers", &self._workers)
            .finish()
    }
}

impl std::fmt::Display for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pool(workers={})", self._workers)
    }
}

fn __sifr_parallel_worker_count(workers: i64) -> usize {
    if workers < 1 {
        return 1usize;
    }
    let requested = workers as u64;
    let max = usize::MAX as u64;
    if requested > max {
        return usize::MAX;
    }
    return requested as usize;
}

fn __sifr_default_parallel_worker_count() -> usize {
    return std::thread::available_parallelism()
        .map_or(1usize, std::num::NonZeroUsize::get);
}

static __SIFR_DEFAULT_PARALLEL_POOL: std::sync::OnceLock<
    Result<rayon::ThreadPool, WorkerRuntimeError>,
> = std::sync::OnceLock::new();

fn __sifr_build_parallel_pool(workers: usize) -> Result<rayon::ThreadPool, WorkerRuntimeError> {
    match rayon::ThreadPoolBuilder::new().num_threads(workers).build() {
        Ok(pool) => Ok(pool),
        Err(error) => Err(WorkerRuntimeError::new(format!(
            "parallel worker pool could not start: {}",
            error
        ))),
    }
}

fn __sifr_default_parallel_pool() -> Result<&'static rayon::ThreadPool, WorkerRuntimeError> {
    match __SIFR_DEFAULT_PARALLEL_POOL
        .get_or_init(|| __sifr_build_parallel_pool(__sifr_default_parallel_worker_count()))
    {
        Ok(pool) => Ok(pool),
        Err(error) => Err(error.clone()),
    }
}

fn __sifr_parallel_map<T: Send, U: Send, F: Fn(T) -> U + Send + Sync>(
    items: Vec<T>,
    worker: F,
) -> Result<Vec<U>, WorkerRuntimeError> {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};
    let pool = __sifr_default_parallel_pool()?;
    return pool.install(|| {
        __sifr_with_silent_worker_panic_hook(|| {
            items
                .into_par_iter()
                .map(|item| {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker(item)))
                        .map_err(|_| WorkerRuntimeError::new("parallel worker panicked".to_string()))
                })
                .collect()
        })
    });
}

fn __sifr_parallel_try_map<T: Send, U: Send, E, F: Fn(T) -> Result<U, E> + Send + Sync>(
    items: Vec<T>,
    worker: F,
) -> Result<Vec<U>, WorkerError>
where
    E: Send + std::fmt::Display,
{
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};
    let pool = __sifr_default_parallel_pool().map_err(__sifr_worker_error_from_runtime)?;
    return pool.install(|| {
        __sifr_with_silent_worker_panic_hook(|| {
            items
                .into_par_iter()
                .map(|item| {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker(item))) {
                        Ok(Ok(value)) => Ok(value),
                        Ok(Err(error)) => Err(WorkerError::new(format!("{}", error))),
                        Err(_) => Err(WorkerError::new("parallel worker panicked".to_string())),
                    }
                })
                .collect()
        })
    });
}

fn __sifr_pool_map<T: Send, U: Send, F: Fn(T) -> U + Send + Sync>(
    pool: &Pool,
    items: Vec<T>,
    worker: F,
) -> Result<Vec<U>, WorkerRuntimeError> {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};
    let Some(worker_pool) = pool._pool.as_ref() else {
        match &pool._failure {
            Some(error) => return Err(error.clone()),
            None => {
                return Err(WorkerRuntimeError::new(
                    "parallel worker pool is unavailable".to_string(),
                ));
            }
        }
    };
    return worker_pool.install(|| {
        __sifr_with_silent_worker_panic_hook(|| {
            items
                .into_par_iter()
                .map(|item| {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker(item)))
                        .map_err(|_| WorkerRuntimeError::new("parallel worker panicked".to_string()))
                })
                .collect()
        })
    });
}

fn __sifr_pool_try_map<T: Send, U: Send, E, F: Fn(T) -> Result<U, E> + Send + Sync>(
    pool: &Pool,
    items: Vec<T>,
    worker: F,
) -> Result<Vec<U>, WorkerError>
where
    E: Send + std::fmt::Display,
{
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};
    let Some(worker_pool) = pool._pool.as_ref() else {
        match &pool._failure {
            Some(error) => return Err(__sifr_worker_error_from_runtime(error.clone())),
            None => {
                return Err(WorkerError::new(
                    "parallel worker pool is unavailable".to_string(),
                ));
            }
        }
    };
    return worker_pool.install(|| {
        __sifr_with_silent_worker_panic_hook(|| {
            items
                .into_par_iter()
                .map(|item| {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker(item))) {
                        Ok(Ok(value)) => Ok(value),
                        Ok(Err(error)) => Err(WorkerError::new(format!("{}", error))),
                        Err(_) => Err(WorkerError::new("parallel worker panicked".to_string())),
                    }
                })
                .collect()
        })
    });
}

fn __sifr_worker_error_from_runtime(error: WorkerRuntimeError) -> WorkerError {
    return WorkerError::new(error.message);
}
"#
}
