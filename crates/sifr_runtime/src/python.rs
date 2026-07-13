#![allow(unsafe_code)]

use pyo3::{ffi, prelude::*};
use std::ffi::{CStr, CString};
use std::fmt;
use std::mem::MaybeUninit;
use std::sync::{Mutex, MutexGuard};

mod arrow_ops;
mod async_runtime;
mod bridge_loader;
mod buffer_ops;
mod call_depth;
mod callback_ops;
mod config_verify;
mod context_ops;
#[cfg(test)]
mod context_ops_tests;
mod coroutine_ops;
mod dlpack_ops;
mod foreign_object;
mod object_ops;
#[cfg(test)]
mod object_ops_tests;
mod opaque_ops;
mod python_error;
#[cfg(test)]
mod python_test_support;
mod recursive_ops;
mod resource_identity;
mod resource_ops;
pub use arrow_ops::{
    arrow_array, arrow_capsule_names, arrow_schema, arrow_stream, release_arrow, ArrowHandle,
    PythonArrowCapsuleMetadata,
};
pub use async_runtime::{async_runtime_diagnostics, PythonAsyncRuntimeDiagnostics};
pub use bridge_loader::PythonBridgeSource;
pub use buffer_ops::{
    buffer_shape, buffer_strides, buffer_suboffsets, buffer_u8, copy_buffer_u8, release_buffer,
    BufferHandle, PythonBufferMetadata,
};
use call_depth::{enter_python_call, python_call_depth};
pub use callback_ops::{
    close_callback, local_callback, local_callback_echo, threadsafe_callback,
    threadsafe_callback_echo, CallbackHandle, PythonCallbackMetadata,
};
use config_verify::verify_interpreter_config;
pub use context_ops::{
    attach_secondary_python_error, context_exit_normal, context_exit_python_error,
    context_exit_sifr_cause, record_context_cleanup_evidence, record_context_ignored_suppression,
    take_context_cleanup_evidence, ContextCleanupEvidence, PythonExitDecision, SifrExitCause,
    SifrExitCauseKind,
};
pub use coroutine_ops::run_coroutine_blocking;
pub use dlpack_ops::{
    dlpack_shape, dlpack_strides, dlpack_tensor, release_dlpack, DlpackHandle,
    PythonDlpackTensorMetadata,
};
pub use foreign_object::ForeignObject;
pub use object_ops::{
    call_attr, call_object, call_object_borrowed, call_object_owned, close_object,
    copy_dict_str_bool, copy_dict_str_bytes, copy_dict_str_float, copy_dict_str_i32,
    copy_dict_str_int, copy_dict_str_str, copy_dict_str_u8, copy_list_bool, copy_list_bytes,
    copy_list_float, copy_list_i32, copy_list_int, copy_list_str, copy_list_u8, copy_record_fields,
    copy_tuple_bool, copy_tuple_bytes, copy_tuple_float, copy_tuple_i32, copy_tuple_int,
    copy_tuple_str, copy_tuple_u8, enter_context, exit_context, expect_instance, from_bool,
    from_bytes, from_dict_str, from_float, from_int, from_list, from_none, from_record, from_str,
    from_tuple, get_attr, get_item_str, import_module, resolve_target, temporary_argument_handle,
    to_bool, to_bytes, to_float, to_i16, to_i32, to_i64, to_i8, to_int, to_isize, to_none, to_str,
    to_u16, to_u32, to_u64, to_u8, to_usize, ObjectHandle,
};
pub use opaque_ops::semantic_close;
pub use python_error::PythonError;
pub use recursive_ops::{
    at_path, dict_str_items, from_dict_results, from_list_results, from_record_results,
    from_tuple_results, list_items, object_is_none, record_field, tuple_items,
};
pub use resource_identity::PythonResourceIdentity;
pub use resource_ops::{exit_context_with_error, resource_diagnostics, PythonResourceDiagnostics};

static RUNTIME_STATE: Mutex<RuntimeState> = Mutex::new(RuntimeState::new());

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonRuntimeConfig {
    pub venv_root: String,
    pub interpreter: String,
    pub executable: String,
    pub sys_prefix: String,
    pub sys_base_prefix: String,
    pub probe_digest: String,
    pub implementation_name: String,
    pub implementation_version: String,
    pub cpython_version_tuple: Vec<u64>,
    pub sys_path: Vec<String>,
    pub site_packages: Vec<String>,
    pub required_import_roots: Vec<String>,
    pub trusted_import_roots: Vec<String>,
    pub native_import_roots: Vec<String>,
    pub trusted_native_roots: Vec<String>,
    pub bridge_sources: Vec<PythonBridgeSource>,
    pub start_async_loop: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythonRuntimeInitStatus {
    Initialized,
    AlreadyInitialized,
}

#[derive(Debug)]
pub struct PythonRuntimeGuard {
    _private: (),
}

impl Drop for PythonRuntimeGuard {
    fn drop(&mut self) {
        let _ignored = async_runtime::shutdown();
        let _ignored = Python::try_attach(foreign_object::drain_pending_releases);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PythonRuntimeDiagnostics {
    pub initialized: bool,
    pub live_objects: usize,
    pub leaked_objects: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PythonRuntimeError {
    NotInitialized,
    ConflictingEnvironment {
        selected_interpreter: String,
        attempted_interpreter: String,
    },
    InterpreterVersionMismatch {
        expected: String,
        actual: String,
    },
    InterpreterConfigMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
    PythonOperationFailed(String),
    ReservedBridgeCollision {
        module: String,
    },
    AsyncRuntimeFailed(String),
    AsyncRuntimeNotRunning,
    AsyncRuntimeStopping,
    OutstandingResources {
        live_objects: usize,
        leaked_objects: usize,
    },
    StateUnavailable,
}

impl fmt::Display for PythonRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "Python runtime has not been initialized"),
            Self::ConflictingEnvironment {
                selected_interpreter,
                attempted_interpreter,
            } => write!(
                f,
                "Python runtime was initialized with '{selected_interpreter}' and cannot be reinitialized with '{attempted_interpreter}'"
            ),
            Self::InterpreterVersionMismatch { expected, actual } => write!(
                f,
                "selected Python environment uses CPython {expected}, but the embedded interpreter is CPython {actual}"
            ),
            Self::InterpreterConfigMismatch {
                field,
                expected,
                actual,
            } => write!(
                f,
                "selected Python environment expected {field} '{expected}', but the embedded interpreter reported '{actual}'"
            ),
            Self::PythonOperationFailed(message) => {
                write!(f, "Python runtime operation failed: {message}")
            }
            Self::ReservedBridgeCollision { module } => write!(
                f,
                "reserved Python bridge namespace collision at '{module}'"
            ),
            Self::AsyncRuntimeFailed(message) => {
                write!(f, "owned Python asyncio runtime failed: {message}")
            }
            Self::AsyncRuntimeNotRunning => {
                write!(f, "owned Python asyncio runtime is not running")
            }
            Self::AsyncRuntimeStopping => {
                write!(f, "owned Python asyncio runtime is stopping")
            }
            Self::OutstandingResources {
                live_objects,
                leaked_objects,
            } => write!(
                f,
                "Python runtime shutdown blocked by {live_objects} live object(s) and {leaked_objects} leaked object(s)"
            ),
            Self::StateUnavailable => write!(f, "Python runtime state is unavailable"),
        }
    }
}

impl std::error::Error for PythonRuntimeError {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeState {
    config: Option<PythonRuntimeConfig>,
    initialized: bool,
    live_objects: usize,
    leaked_objects: usize,
}

impl RuntimeState {
    const fn new() -> Self {
        Self {
            config: None,
            initialized: false,
            live_objects: 0,
            leaked_objects: 0,
        }
    }
}

pub fn initialize_runtime(
    config: PythonRuntimeConfig,
) -> Result<PythonRuntimeInitStatus, PythonRuntimeError> {
    let mut state = runtime_state()?;
    if let Some(selected) = &state.config {
        if selected != &config {
            return Err(PythonRuntimeError::ConflictingEnvironment {
                selected_interpreter: selected.interpreter.clone(),
                attempted_interpreter: config.interpreter,
            });
        }
        if state.initialized {
            return Ok(PythonRuntimeInitStatus::AlreadyInitialized);
        }
    }

    state.config = Some(config.clone());
    initialize_cpython_with_config(&config)?;
    configure_interpreter(&config)?;
    Python::try_attach(|py| bridge_loader::install(py, &config.bridge_sources))
        .ok_or(PythonRuntimeError::NotInitialized)??;
    Python::try_attach(context_ops::register_boundary_error)
        .ok_or(PythonRuntimeError::NotInitialized)??;
    if config.start_async_loop {
        async_runtime::start()?;
    }
    state.initialized = true;
    Ok(PythonRuntimeInitStatus::Initialized)
}

pub fn runtime_guard() -> Result<PythonRuntimeGuard, PythonRuntimeError> {
    ensure_initialized()?;
    Ok(PythonRuntimeGuard { _private: () })
}

pub fn attach<F, R>(f: F) -> Result<R, PythonRuntimeError>
where
    F: for<'py> FnOnce(Python<'py>) -> R,
{
    ensure_initialized()?;
    Python::try_attach(|py| {
        foreign_object::drain_pending_releases(py);
        f(py)
    })
    .ok_or(PythonRuntimeError::NotInitialized)
}

pub fn detach<F, R>(py: Python<'_>, f: F) -> R
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    py.detach(f)
}

pub fn shutdown_diagnostics() -> Result<PythonRuntimeDiagnostics, PythonRuntimeError> {
    let state = runtime_state()?;
    Ok(PythonRuntimeDiagnostics {
        initialized: state.initialized,
        live_objects: state.live_objects,
        leaked_objects: state.leaked_objects,
    })
}

pub fn validate_shutdown() -> Result<(), PythonRuntimeError> {
    attach(foreign_object::drain_pending_releases)?;
    let diagnostics = shutdown_diagnostics()?;
    if diagnostics.live_objects == 0 && diagnostics.leaked_objects == 0 {
        return Ok(());
    }
    Err(PythonRuntimeError::OutstandingResources {
        live_objects: diagnostics.live_objects,
        leaked_objects: diagnostics.leaked_objects,
    })
}

fn configure_interpreter(config: &PythonRuntimeConfig) -> Result<(), PythonRuntimeError> {
    attach_initialized(|py| {
        verify_interpreter_config(py, config)?;
        let sys = py.import("sys").map_err(|error| py_error(&error))?;
        let path = sys.getattr("path").map_err(|error| py_error(&error))?;
        for entry in config
            .site_packages
            .iter()
            .chain(config.sys_path.iter())
            .rev()
        {
            path.call_method1("insert", (0, entry.as_str()))
                .map_err(|error| py_error(&error))?;
        }
        Ok(())
    })
}

fn initialize_cpython_with_config(config: &PythonRuntimeConfig) -> Result<(), PythonRuntimeError> {
    if unsafe { ffi::Py_IsInitialized() } != 0 {
        return Ok(());
    }

    let mut raw_config = MaybeUninit::<ffi::PyConfig>::uninit();
    unsafe {
        ffi::PyConfig_InitPythonConfig(raw_config.as_mut_ptr());
    }
    let mut raw_config = unsafe { raw_config.assume_init() };
    raw_config.install_signal_handlers = 0;
    raw_config.parse_argv = 0;
    raw_config.use_environment = 0;
    raw_config.user_site_directory = 0;
    raw_config.module_search_paths_set = 1;

    let configure_result = configure_raw_python_config(&mut raw_config, config);
    let initialize_result = configure_result.and_then(|()| {
        py_status_result(
            unsafe { ffi::Py_InitializeFromConfig(&raw const raw_config) },
            "initialize CPython",
        )
    });
    unsafe {
        ffi::PyConfig_Clear(&raw mut raw_config);
    }
    initialize_result?;
    unsafe {
        ffi::PyEval_SaveThread();
    }
    Ok(())
}

fn configure_raw_python_config(
    raw_config: &mut ffi::PyConfig,
    config: &PythonRuntimeConfig,
) -> Result<(), PythonRuntimeError> {
    let raw_config_ptr = std::ptr::from_mut(raw_config);
    set_config_string(
        raw_config_ptr,
        std::ptr::addr_of_mut!(raw_config.executable),
        &config.executable,
        "set Python executable",
    )?;
    set_config_string(
        raw_config_ptr,
        std::ptr::addr_of_mut!(raw_config.base_executable),
        &config.executable,
        "set Python base executable",
    )?;
    set_config_string(
        raw_config_ptr,
        std::ptr::addr_of_mut!(raw_config.program_name),
        &config.interpreter,
        "set Python program name",
    )?;
    set_optional_config_string(
        raw_config_ptr,
        std::ptr::addr_of_mut!(raw_config.prefix),
        &config.sys_prefix,
        "set Python prefix",
    )?;
    set_optional_config_string(
        raw_config_ptr,
        std::ptr::addr_of_mut!(raw_config.exec_prefix),
        &config.sys_prefix,
        "set Python exec prefix",
    )?;
    set_optional_config_string(
        raw_config_ptr,
        std::ptr::addr_of_mut!(raw_config.base_prefix),
        &config.sys_base_prefix,
        "set Python base prefix",
    )?;
    set_optional_config_string(
        raw_config_ptr,
        std::ptr::addr_of_mut!(raw_config.base_exec_prefix),
        &config.sys_base_prefix,
        "set Python base exec prefix",
    )?;
    set_config_argv(raw_config, &config.interpreter)?;
    for path in &config.sys_path {
        append_module_search_path(raw_config, path)?;
    }
    Ok(())
}

fn set_optional_config_string(
    raw_config: *mut ffi::PyConfig,
    target: *mut *mut libc::wchar_t,
    value: &str,
    context: &'static str,
) -> Result<(), PythonRuntimeError> {
    if value.is_empty() {
        return Ok(());
    }
    set_config_string(raw_config, target, value, context)
}

fn set_config_string(
    raw_config: *mut ffi::PyConfig,
    target: *mut *mut libc::wchar_t,
    value: &str,
    context: &'static str,
) -> Result<(), PythonRuntimeError> {
    let value = CString::new(value).map_err(|_| {
        PythonRuntimeError::PythonOperationFailed(format!("{context}: value contains NUL byte"))
    })?;
    py_status_result(
        unsafe { ffi::PyConfig_SetBytesString(raw_config, target, value.as_ptr()) },
        context,
    )
}

fn set_config_argv(
    raw_config: &mut ffi::PyConfig,
    interpreter: &str,
) -> Result<(), PythonRuntimeError> {
    let interpreter = CString::new(interpreter).map_err(|_| {
        PythonRuntimeError::PythonOperationFailed(
            "set Python argv: value contains NUL byte".to_string(),
        )
    })?;
    let mut argv = [interpreter.as_ptr()];
    py_status_result(
        unsafe { ffi::PyConfig_SetBytesArgv(raw_config, 1, argv.as_mut_ptr()) },
        "set Python argv",
    )
}

fn append_module_search_path(
    raw_config: &mut ffi::PyConfig,
    path: &str,
) -> Result<(), PythonRuntimeError> {
    let wide = wide_string(path);
    py_status_result(
        unsafe {
            ffi::PyWideStringList_Append(&raw mut raw_config.module_search_paths, wide.as_ptr())
        },
        "set Python module search path",
    )
}

#[cfg(windows)]
fn wide_string(value: &str) -> Vec<libc::wchar_t> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(not(windows))]
fn wide_string(value: &str) -> Vec<libc::wchar_t> {
    value
        .chars()
        .map(|ch| ch as libc::wchar_t)
        .chain(std::iter::once(0))
        .collect()
}

fn py_status_result(
    status: ffi::PyStatus,
    context: &'static str,
) -> Result<(), PythonRuntimeError> {
    if unsafe { ffi::PyStatus_Exception(status) } == 0 {
        return Ok(());
    }
    Err(PythonRuntimeError::PythonOperationFailed(format!(
        "{context}: {}",
        py_status_message(status)
    )))
}

fn py_status_message(status: ffi::PyStatus) -> String {
    if status.err_msg.is_null() {
        return format!("status exit code {}", status.exitcode);
    }
    unsafe { CStr::from_ptr(status.err_msg) }
        .to_string_lossy()
        .into_owned()
}

fn attach_initialized<F, R>(f: F) -> Result<R, PythonRuntimeError>
where
    F: for<'py> FnOnce(Python<'py>) -> Result<R, PythonRuntimeError>,
{
    Python::try_attach(f).ok_or(PythonRuntimeError::NotInitialized)?
}

fn ensure_initialized() -> Result<(), PythonRuntimeError> {
    let state = runtime_state()?;
    if state.initialized {
        Ok(())
    } else {
        Err(PythonRuntimeError::NotInitialized)
    }
}

fn runtime_state() -> Result<MutexGuard<'static, RuntimeState>, PythonRuntimeError> {
    RUNTIME_STATE
        .lock()
        .map_err(|_| PythonRuntimeError::StateUnavailable)
}

fn runtime_config() -> Result<PythonRuntimeConfig, PythonRuntimeError> {
    runtime_state()?
        .config
        .clone()
        .ok_or(PythonRuntimeError::NotInitialized)
}

pub(super) fn update_object_count(delta: isize) -> Result<(), PythonRuntimeError> {
    let mut state = runtime_state()?;
    if delta.is_positive() {
        state.live_objects = state.live_objects.saturating_add(delta.cast_unsigned());
    } else {
        state.live_objects = state.live_objects.saturating_sub(delta.unsigned_abs());
    }
    Ok(())
}

fn py_error(error: &PyErr) -> PythonRuntimeError {
    PythonRuntimeError::PythonOperationFailed(error.to_string())
}

#[cfg(test)]
fn reset_runtime_state_for_tests() {
    let _ignored = async_runtime::shutdown();
    async_runtime::reset_for_tests();
    let _ignored = Python::try_attach(bridge_loader::reset_for_tests);
    let mut state = runtime_state().expect("runtime state should be available");
    *state = RuntimeState::new();
    foreign_object::reset_pending_releases_for_tests();
    context_ops::reset_context_state_for_tests();
    object_ops::reset_object_store_for_tests();
}

#[cfg(test)]
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
fn test_guard() -> MutexGuard<'static, ()> {
    match TEST_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
fn test_config(label: &str) -> PythonRuntimeConfig {
    let mut config = python_test_support::local_python_config();
    config.probe_digest = format!("digest-{label}");
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_and_accepts_repeated_same_config() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        let config = test_config("same");

        assert_eq!(
            initialize_runtime(config.clone()),
            Ok(PythonRuntimeInitStatus::Initialized)
        );
        assert_eq!(
            initialize_runtime(config),
            Ok(PythonRuntimeInitStatus::AlreadyInitialized)
        );
    }

    #[test]
    fn rejects_reinitialization_with_different_config() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("first")).expect("first init should succeed");

        let error =
            initialize_runtime(test_config("second")).expect_err("different config should fail");

        assert!(matches!(
            error,
            PythonRuntimeError::ConflictingEnvironment { .. }
        ));
    }

    #[test]
    fn attach_requires_runtime_initialization() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();

        let result = attach(|py| py.version_info().major);

        assert_eq!(result, Err(PythonRuntimeError::NotInitialized));
    }

    #[test]
    fn attach_and_detach_run_under_initialized_runtime() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("detach")).expect("init should succeed");

        let result = attach(|py| detach(py, || 41 + 1));

        assert_eq!(result, Ok(42));
    }

    #[test]
    fn owned_object_tracking_is_released_on_next_attach() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("object")).expect("init should succeed");
        let object = attach(|py| ForeignObject::new(py.None())).expect("attach should succeed");
        assert!(object.is_ok());
        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 1,
                leaked_objects: 0,
            }
        );

        drop(object);
        attach(|_py| ()).expect("attach should drain the dropped object");

        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn detached_thread_drop_is_drained_on_next_attach() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("detached-drop")).expect("init should succeed");
        let object = attach(|py| ForeignObject::new(py.None())).expect("attach should succeed");
        let object = object.expect("object should be created");

        std::thread::spawn(move || drop(object))
            .join()
            .expect("detached drop thread should finish");

        assert_eq!(foreign_object::pending_release_count(), 1);
        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 1,
                leaked_objects: 0,
            }
        );
        attach(|_py| ()).expect("attach should drain pending releases");
        assert_eq!(foreign_object::pending_release_count(), 0);
        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn runtime_guard_drains_pending_releases_at_epilogue() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("epilogue-drop")).expect("init should succeed");
        let runtime_guard = runtime_guard().expect("runtime guard should be available");
        let object = attach(|py| ForeignObject::new(py.None())).expect("attach should succeed");
        let object = object.expect("object should be created");
        std::thread::spawn(move || drop(object))
            .join()
            .expect("detached drop thread should finish");
        assert_eq!(foreign_object::pending_release_count(), 1);

        drop(runtime_guard);

        assert_eq!(foreign_object::pending_release_count(), 0);
        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn shutdown_validation_reports_outstanding_objects() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("shutdown")).expect("init should succeed");
        let object = attach(|py| ForeignObject::new(py.None())).expect("attach should succeed");
        assert!(object.is_ok());

        let error = validate_shutdown().expect_err("live object should block shutdown");

        assert_eq!(
            error,
            PythonRuntimeError::OutstandingResources {
                live_objects: 1,
                leaked_objects: 0,
            }
        );
        drop(object);
    }

    #[test]
    fn object_operations_cover_import_attr_item_call_kwargs_and_context() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("ops")).expect("init should succeed");

        let builtins = import_module("builtins").expect("builtins import should succeed");
        let dict = get_attr(&builtins, "dict").expect("dict attr should succeed");
        let math = import_module("math").expect("math import should succeed");
        let kwargs_dict = call_object(&dict, &[], &[("module", math.clone())])
            .expect("kwargs call should succeed");
        let item = get_item_str(&kwargs_dict, "module").expect("item access should succeed");
        close_object(item).expect("item close should succeed");
        close_object(kwargs_dict).expect("dict close should succeed");
        close_object(math).expect("math close should succeed");
        close_object(dict).expect("dict callable close should succeed");

        let contextlib = import_module("contextlib").expect("contextlib import should succeed");
        let nullcontext =
            get_attr(&contextlib, "nullcontext").expect("nullcontext attr should succeed");
        let manager = call_object(&nullcontext, &[], &[]).expect("nullcontext call should succeed");
        let entered = enter_context(&manager).expect("context enter should succeed");
        exit_context(&manager).expect("context exit should succeed");
        close_object(entered).expect("entered context close should succeed");
        close_object(manager).expect("context manager close should succeed");
        close_object(nullcontext).expect("nullcontext close should succeed");
        close_object(contextlib).expect("contextlib close should succeed");
        close_object(builtins).expect("builtins close should succeed");

        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            PythonRuntimeDiagnostics {
                initialized: true,
                live_objects: 0,
                leaked_objects: 0,
            }
        );
    }

    #[test]
    fn import_failure_preserves_python_exception_context() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        let mut config = test_config("import-failure");
        config
            .required_import_roots
            .push("sifr_missing_py3_module".to_string());
        config
            .trusted_import_roots
            .push("sifr_missing_py3_module".to_string());
        initialize_runtime(config).expect("init should succeed");

        let error = import_module("sifr_missing_py3_module")
            .expect_err("missing module should fail as PythonError");

        assert_eq!(error.kind, "import");
        assert_eq!(error.exception_type, "ModuleNotFoundError");
        assert!(error.message.contains("sifr_missing_py3_module"));
        assert!(!error.traceback.is_empty());
    }

    #[test]
    fn trust_policy_rejects_undeclared_import_roots() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("trust")).expect("init should succeed");

        let error = import_module("json").expect_err("untrusted import should fail");

        assert_eq!(error.kind, "trust");
        assert_eq!(error.exception_type, "SIFR-PYTRUST");
        assert!(error.message.contains("json"));
    }

    #[test]
    fn failed_record_field_copy_does_not_leak_partial_handles() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("record-field-leak")).expect("init should succeed");
        let value = from_int(42).expect("field value should be stored");
        let record = from_record(&[("value", value.clone())]).expect("record should be stored");
        let before = shutdown_diagnostics().expect("diagnostics should be available");

        let error = copy_record_fields(&record, &["value", "missing"])
            .expect_err("missing field should fail");

        assert_eq!(error.kind, "conversion");
        assert_eq!(
            shutdown_diagnostics().expect("diagnostics should be available"),
            before
        );
        close_object(record).expect("record should close");
        close_object(value).expect("field value should close");
    }

    #[test]
    fn attribute_and_call_failures_preserve_python_error_families() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        initialize_runtime(test_config("operation-failures")).expect("init should succeed");
        let math = import_module("math").expect("math import should succeed");

        let attr_error = get_attr(&math, "does_not_exist")
            .expect_err("missing attribute should fail as PythonError");
        assert_eq!(attr_error.kind, "attribute");
        assert_eq!(attr_error.exception_type, "AttributeError");

        let builtins = import_module("builtins").expect("builtins import should succeed");
        let len = get_attr(&builtins, "len").expect("len attr should succeed");
        let call_error = call_object(&len, &[], &[]).expect_err("wrong arg count should fail");
        assert_eq!(call_error.kind, "call");
        assert_eq!(call_error.exception_type, "TypeError");

        close_object(len).expect("len close should succeed");
        close_object(builtins).expect("builtins close should succeed");
        close_object(math).expect("math close should succeed");
    }
}
