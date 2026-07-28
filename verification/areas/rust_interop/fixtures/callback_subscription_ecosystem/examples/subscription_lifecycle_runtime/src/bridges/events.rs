use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::thread::ThreadId;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sifr_runtime::interop::{
    CallbackBackpressure, CallbackOverflow, CallbackShutdown, Handle, RustPanicErrorBridge,
};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use super::protocols;
use super::support::{
    active_work, cancel_task, join_task, spawn_tracked, watcher_started, watcher_stopped,
    EventCallback, Observations, SubscriptionError, TemporaryDirectory, OPERATION_TIMEOUT,
};

type CallbackTask = JoinHandle<Result<(), SubscriptionError>>;

pub struct Subscription {
    callback: EventCallback,
    observations: Observations,
    tasks: Mutex<Option<Vec<CallbackTask>>>,
    cancellation_task: Mutex<Option<CallbackTask>>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    temporary_directory: Mutex<Option<TemporaryDirectory>>,
    verified: AtomicBool,
}

impl fmt::Debug for Subscription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Subscription")
            .field("verified", &self.verified.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

pub fn subscribe(callback: EventCallback) -> Result<Handle<Subscription>, SubscriptionError> {
    verify_callback_policy(&callback)?;
    let websocket_listener = bind_loopback("WebSocket")?;
    let websocket_address = websocket_listener
        .local_addr()
        .map_err(|error| SubscriptionError::context("WebSocket local address", error))?;
    let redis_listener = bind_loopback("Redis")?;
    let redis_address = redis_listener
        .local_addr()
        .map_err(|error| SubscriptionError::context("Redis local address", error))?;
    let temporary_directory = TemporaryDirectory::create()?;
    let observations = Observations::default();
    let watcher = create_watcher(
        temporary_directory.path()?,
        callback.clone(),
        observations.clone(),
        std::thread::current().id(),
    )?;

    let tasks = vec![
        spawn_tracked(protocols::run_websocket(
            websocket_listener,
            websocket_address,
            callback.clone(),
            observations.clone(),
        )),
        spawn_tracked(protocols::run_redis(
            redis_listener,
            redis_address,
            callback.clone(),
            observations.clone(),
        )),
    ];
    let cancellation_task = spawn_tracked(async {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        Err(SubscriptionError::new(
            "cancellation sentinel was not cancelled",
        ))
    });
    std::fs::write(temporary_directory.path()?.join("event.txt"), b"notify")
        .map_err(|error| SubscriptionError::context("notify trigger", error))?;

    Ok(Handle::new(Subscription {
        callback,
        observations,
        tasks: Mutex::new(Some(tasks)),
        cancellation_task: Mutex::new(Some(cancellation_task)),
        watcher: Mutex::new(Some(watcher)),
        temporary_directory: Mutex::new(Some(temporary_directory)),
        verified: AtomicBool::new(false),
    }))
}

pub async fn verify(subscription: &Handle<Subscription>) -> Result<String, SubscriptionError> {
    let subscription = subscription
        .inner_ref()
        .map_err(|error| SubscriptionError::context("subscription state", error))?;
    let tasks = mutex_value(&subscription.tasks).take().unwrap_or_default();
    for task in tasks {
        join_task(task, "subscription protocol task").await?;
    }
    wait_for_notify(&subscription.observations).await?;

    let (overflow, queue_drained) = verify_bounded_queue(&subscription.callback)?;
    let handler_error = verify_handler_error(&subscription.callback)?;
    let panic = verify_callback_panic(&subscription.callback)?;
    subscription.verified.store(true, Ordering::SeqCst);
    let observations = subscription.observations.protocol_summary()?;
    let foreign_thread = subscription.observations.foreign_thread()?;
    Ok(format!(
        "{observations};overflow={overflow};handler-error={handler_error};panic={panic};\
         foreign-thread={foreign_thread};queue-drained={queue_drained}"
    ))
}

pub async fn aclose(mut subscription: Handle<Subscription>) -> Result<(), SubscriptionError> {
    let subscription = subscription
        .inner_mut()
        .map_err(|error| SubscriptionError::context("subscription state", error))?;
    if !subscription.verified.load(Ordering::SeqCst) {
        return Err(SubscriptionError::new(
            "subscription must be verified before async close",
        ));
    }
    let remaining_tasks = mutex_value(&subscription.tasks).take().unwrap_or_default();
    for task in remaining_tasks {
        join_task(task, "subscription close protocol task").await?;
    }
    let cancellation_task = mutex_value(&subscription.cancellation_task)
        .take()
        .ok_or_else(|| SubscriptionError::new("cancellation handle was already consumed"))?;
    cancel_task(cancellation_task).await?;

    if mutex_value(&subscription.watcher).take().is_some() {
        watcher_stopped();
    }
    let temp_removed = mutex_value(&subscription.temporary_directory)
        .take()
        .ok_or_else(|| SubscriptionError::new("temporary directory was already consumed"))?
        .remove()?;
    if active_work() != 0 {
        return Err(SubscriptionError::new(format!(
            "subscription close retained {} active callbacks or watchers",
            active_work()
        )));
    }
    set_close_observation(format!(
        "shutdown=drain;cancelled=true;active=0;temp-removed={temp_removed}"
    ));
    Ok(())
}

pub fn close_observation() -> Result<String, SubscriptionError> {
    CLOSE_OBSERVATION.with(|observation| {
        observation
            .borrow()
            .clone()
            .ok_or_else(|| SubscriptionError::new("async close was not observed"))
    })
}

pub fn map_panic(error: RustPanicErrorBridge) -> SubscriptionError {
    SubscriptionError::new(error.to_string())
}

fn verify_callback_policy(callback: &EventCallback) -> Result<(), SubscriptionError> {
    let policy = callback.policy();
    if policy.backpressure != CallbackBackpressure::Bounded(2)
        || policy.overflow != CallbackOverflow::Error
        || policy.shutdown != CallbackShutdown::Drain
    {
        return Err(SubscriptionError::new(format!(
            "unexpected retained callback policy: {policy:?}"
        )));
    }
    Ok(())
}

fn create_watcher(
    directory: &std::path::Path,
    callback: EventCallback,
    observations: Observations,
    owner_thread: ThreadId,
) -> Result<RecommendedWatcher, SubscriptionError> {
    let mut watcher =
        notify::recommended_watcher(move |result: notify::Result<notify::Event>| match result {
            Ok(event) if matches!(event.kind, EventKind::Create(_)) => {
                let foreign_thread = std::thread::current().id() != owner_thread;
                match callback.call(("notify:create".to_string(),)) {
                    Ok(Ok(())) => {
                        observations.record_notify("notify:create".to_string(), foreign_thread);
                    }
                    Ok(Err(error)) => {
                        observations.record_failure(format!("notify callback: {error}"));
                    }
                    Err(error) => {
                        observations.record_failure(format!("notify callback: {error}"));
                    }
                }
            }
            Ok(_) => {}
            Err(error) => observations.record_failure(format!("notify watcher: {error}")),
        })
        .map_err(|error| SubscriptionError::context("create notify watcher", error))?;
    watcher
        .watch(directory, RecursiveMode::NonRecursive)
        .map_err(|error| SubscriptionError::context("watch notify directory", error))?;
    watcher_started();
    Ok(watcher)
}

async fn wait_for_notify(observations: &Observations) -> Result<(), SubscriptionError> {
    tokio::time::timeout(OPERATION_TIMEOUT, async {
        while !observations.complete() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .map_err(|_| SubscriptionError::new("notify callback timed out"))?;
    observations.summary().map(|_| ())
}

fn verify_bounded_queue(
    callback: &EventCallback,
) -> Result<(&'static str, usize), SubscriptionError> {
    let mut queue = VecDeque::with_capacity(2);
    enqueue_callback_event(&mut queue, "queue:first")?;
    enqueue_callback_event(&mut queue, "queue:second")?;
    let overflow = match enqueue_callback_event(&mut queue, "queue:overflow") {
        Err(error) if error.to_string() == "bounded callback queue overflow" => "error",
        Err(error) => return Err(error),
        Ok(()) => {
            return Err(SubscriptionError::new(
                "bounded callback queue accepted an event past capacity",
            ));
        }
    };
    let mut drained = 0;
    while let Some(event) = queue.pop_front() {
        invoke_expected_success(callback, event, "drain callback queue")?;
        drained += 1;
    }
    Ok((overflow, drained))
}

fn enqueue_callback_event(
    queue: &mut VecDeque<String>,
    event: &str,
) -> Result<(), SubscriptionError> {
    const CALLBACK_QUEUE_CAPACITY: usize = 2;

    if queue.len() >= CALLBACK_QUEUE_CAPACITY {
        return Err(SubscriptionError::new(
            "bounded callback queue overflow",
        ));
    }
    queue.push_back(event.to_string());
    Ok(())
}

fn verify_handler_error(callback: &EventCallback) -> Result<String, SubscriptionError> {
    match callback.call(("handler-error".to_string(),)) {
        Ok(Err(error)) if error == "expected-handler-error" => Ok(error),
        Ok(Err(error)) => Err(SubscriptionError::new(format!(
            "unexpected handler error: {error}"
        ))),
        Ok(Ok(())) => Err(SubscriptionError::new(
            "handler error unexpectedly returned success",
        )),
        Err(error) => Err(SubscriptionError::context(
            "handler error callback panicked",
            error,
        )),
    }
}

fn verify_callback_panic(callback: &EventCallback) -> Result<String, SubscriptionError> {
    match callback.call(("panic".to_string(),)) {
        Err(error) if error.to_string() == "Rust bridge panicked" => Ok(error.to_string()),
        Err(error) => Err(SubscriptionError::new(format!(
            "callback panic was not stably redacted: {error}"
        ))),
        Ok(_) => Err(SubscriptionError::new(
            "callback panic unexpectedly crossed the Rust boundary",
        )),
    }
}

fn invoke_expected_success(
    callback: &EventCallback,
    event: String,
    context: &str,
) -> Result<(), SubscriptionError> {
    match callback.call((event,)) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(SubscriptionError::new(format!("{context}: {error}"))),
        Err(error) => Err(SubscriptionError::context(context, error)),
    }
}

fn bind_loopback(context: &str) -> Result<TcpListener, SubscriptionError> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|error| SubscriptionError::context(&format!("{context} bind"), error))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| SubscriptionError::context(&format!("{context} nonblocking"), error))?;
    TcpListener::from_std(listener)
        .map_err(|error| SubscriptionError::context(&format!("{context} Tokio listener"), error))
}

fn mutex_value<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn set_close_observation(value: String) {
    CLOSE_OBSERVATION.with(|observation| {
        *observation.borrow_mut() = Some(value);
    });
}

thread_local! {
    static CLOSE_OBSERVATION: std::cell::RefCell<Option<String>> = const {
        std::cell::RefCell::new(None)
    };
}

impl Drop for Subscription {
    fn drop(&mut self) {
        for task in mutex_value(&self.tasks).take().unwrap_or_default() {
            task.abort();
        }
        if let Some(task) = mutex_value(&self.cancellation_task).take() {
            task.abort();
        }
        if mutex_value(&self.watcher).take().is_some() {
            watcher_stopped();
        }
        mutex_value(&self.temporary_directory).take();
    }
}
