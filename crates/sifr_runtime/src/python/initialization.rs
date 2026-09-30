//! CPython process initialization and PyConfig ownership boundary.
//!
//! RuntimeState's mutex serializes this module's entry. CPython stays initialized
//! for the process lifetime: no other runtime path finalizes it. PyConfig and its
//! allocated strings are exclusively owned here and cleared once after every
//! configuration/initialization result. CPython copies argv/path strings before
//! Rust temporary storage ends; successful initialization owns the current thread
//! state/GIL, which SaveThread releases before PyO3 attaches any runtime callers.
#![allow(unsafe_code)]

use super::{PythonRuntimeConfig, PythonRuntimeError};
use pyo3::ffi;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;

pub(super) fn initialize_cpython_with_config(
    config: &PythonRuntimeConfig,
) -> Result<(), PythonRuntimeError> {
    // SAFETY: process-global observation under runtime initialization authority;
    // the runtime never finalizes CPython or mutates it concurrently here.
    if unsafe { ffi::Py_IsInitialized() } != 0 {
        return Ok(());
    }

    // SAFETY: CPython initializes every PyConfig field before assume_init. The
    // stack allocation is exclusively owned until Clear, including error paths.
    let mut raw_config = MaybeUninit::<ffi::PyConfig>::uninit();
    unsafe {
        ffi::PyConfig_InitPythonConfig(raw_config.as_mut_ptr());
    }
    let mut raw_config = unsafe { raw_config.assume_init() };
    raw_config.install_signal_handlers = 0;
    raw_config.parse_argv = 0;
    raw_config.use_environment = 0;
    raw_config.user_site_directory = 0;
    raw_config.write_bytecode = 0;
    raw_config.module_search_paths_set = 1;

    let configure_result = configure_raw_python_config(&mut raw_config, config);
    // SAFETY: all config fields and copied strings belong to this live config.
    // Initialization takes no alias or ownership of the Rust stack allocation.
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
    // SAFETY: successful initialization attached this thread and owns its GIL;
    // detaching hands thread-state attachment to later scoped PyO3 entry.
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
    // SAFETY: both pointers refer to the same exclusive initialized PyConfig;
    // CPython copies value before the CString expires, replacing only its field.
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
    // SAFETY: argv has exactly one live C string; CPython copies it into config.
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
    // SAFETY: preflight rejects embedded NUL; wide is a live terminated string,
    // copied into the exclusively owned list before its Rust storage expires.
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
    // SAFETY: PyStatus is a by-value status returned by these CPython APIs.
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
    // SAFETY: non-null err_msg is CPython-owned, static terminated error text;
    // copy it now without transferring ownership or admitting mutation.
    unsafe { CStr::from_ptr(status.err_msg) }
        .to_string_lossy()
        .into_owned()
}

pub(super) fn validate_config_input(
    config: &PythonRuntimeConfig,
) -> Result<(), PythonRuntimeError> {
    for (field, value) in [
        ("interpreter", config.interpreter.as_str()),
        ("executable", config.executable.as_str()),
        ("sys_prefix", config.sys_prefix.as_str()),
        ("sys_base_prefix", config.sys_base_prefix.as_str()),
    ]
    .into_iter()
    .chain(
        config
            .sys_path
            .iter()
            .map(|path| ("sys_path", path.as_str())),
    ) {
        if value.contains('\0') {
            return Err(PythonRuntimeError::PythonOperationFailed(format!(
                "configure Python {field}: value contains NUL byte"
            )));
        }
    }
    Ok(())
}
