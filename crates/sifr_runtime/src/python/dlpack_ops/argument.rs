use super::abi::{DLTENSOR_NAME, DLTENSOR_VERSIONED_NAME, ManagedTensor};
use super::{DlpackEntry, DlpackHandle, PythonError, closed_error, dlpack_error, dlpack_store};
use crate::python::ObjectHandle;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::PyCapsule;
use std::ffi::CStr;

pub struct PythonDlpackArgument {
    entry: Option<DlpackEntry>,
    object: Option<ObjectHandle>,
    // This structural pin survives semantic close/reset of exposed handles.
    capsule: Option<Py<PyAny>>,
}

impl PythonDlpackArgument {
    pub fn object(&self) -> Result<ObjectHandle, PythonError> {
        let object = self
            .object
            .as_ref()
            .ok_or_else(|| dlpack_error("Python DLPack argument is already finalized"))?;
        super::super::object_ops::temporary_argument_handle(object)
    }

    pub fn finish(mut self) -> Result<(), PythonError> {
        let entry = self
            .entry
            .take()
            .ok_or_else(|| dlpack_error("Python DLPack argument is already finalized"))?;
        attach_finalize(entry, self.object.take(), self.capsule.take())
            .map_err(PythonError::runtime)?
    }
}

impl Drop for PythonDlpackArgument {
    fn drop(&mut self) {
        if let Some(entry) = self.entry.take() {
            let _ignored = attach_finalize(entry, self.object.take(), self.capsule.take());
        }
    }
}

fn attach_finalize(
    mut entry: DlpackEntry,
    object: Option<ObjectHandle>,
    capsule: Option<Py<PyAny>>,
) -> Result<Result<(), PythonError>, super::super::PythonRuntimeError> {
    let initialized = super::super::ensure_initialized();
    // Until attached, the producer-named argument capsule is the only deleter
    // owner. CPython attachment for cleanup is independent of semantic reset.
    entry._tensor.released = true;
    let result = Python::try_attach(move |py| finalize(py, entry, object, capsule))
        .ok_or(super::super::PythonRuntimeError::NotInitialized)?;
    initialized?;
    Ok(result)
}

pub fn prepare_dlpack_argument(handle: DlpackHandle) -> Result<PythonDlpackArgument, PythonError> {
    super::super::attach(|py| {
        let mut entry = {
            let mut store = dlpack_store()?;
            if store
                .tensors
                .get(&handle.0)
                .is_some_and(|entry| entry.token == handle.1)
            {
                store.tensors.remove(&handle.0)
            } else {
                return Err(closed_error(handle.0));
            }
        }
        .ok_or_else(|| closed_error(handle.0))?;
        let (object, capsule) = argument_capsule(py, &mut entry)?;
        Ok(PythonDlpackArgument {
            entry: Some(entry),
            object: Some(object),
            capsule: Some(capsule),
        })
    })
    .map_err(PythonError::runtime)?
}

#[allow(unsafe_code)]
fn argument_capsule(
    py: Python<'_>,
    entry: &mut DlpackEntry,
) -> Result<(ObjectHandle, Py<PyAny>), PythonError> {
    let tensor = entry._tensor.tensor;
    let pointer = std::ptr::NonNull::new(tensor.pointer())
        .ok_or_else(|| dlpack_error("DLPack argument pointer is null"))?;
    // SAFETY: the entry owns the live managed allocation. The static name and
    // destructor implement one-shot DLPack ownership while attached; no Rust
    // borrow of the payload survives its exposure to the foreign consumer.
    let capsule = unsafe {
        PyCapsule::new_with_pointer_and_destructor(
            py,
            pointer,
            tensor.capsule_name(),
            Some(argument_capsule_destructor),
        )
    }
    .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "DLPack argument capsule"))?;
    // Once created, this capsule owns release, even when handle creation fails.
    entry._tensor.released = true;
    let capsule = capsule.into_any().unbind();
    let object = super::super::object_ops::store_object(capsule.clone_ref(py))?;
    Ok((object, capsule))
}

#[allow(unsafe_code)]
fn finalize(
    py: Python<'_>,
    mut entry: DlpackEntry,
    object: Option<ObjectHandle>,
    capsule: Option<Py<PyAny>>,
) -> Result<(), PythonError> {
    let capsule =
        capsule.ok_or_else(|| dlpack_error("DLPack argument capsule pin is unavailable"))?;
    let bound = capsule
        .bind(py)
        .cast::<PyCapsule>()
        .map_err(|_| dlpack_error("generated DLPack consumer argument is not a PyCapsule"))?;
    let name = capsule_name(bound)?;
    let tensor = entry._tensor.tensor;
    if name == tensor.used_capsule_name() {
        // The foreign consumer may already have released the managed header.
        // Inspect capsule state only; never dereference that header here.
        entry._tensor.released = true;
    } else if name == tensor.capsule_name() {
        let pointer = bound.pointer_checked(Some(name)).map_err(|error| {
            PythonError::from_pyerr(py, error, "zero-copy", "DLPack argument identity")
        })?;
        if pointer.as_ptr() != tensor.pointer() {
            return Err(dlpack_error("DLPack argument capsule pointer changed"));
        }
        // SAFETY: the structural capsule pin retains the immutable name/payload;
        // renaming revokes all retained shells before the entry releases once.
        let rename =
            unsafe { ffi::PyCapsule_SetName(bound.as_ptr(), tensor.used_capsule_name().as_ptr()) };
        if rename != 0 {
            return Err(PythonError::from_pyerr(
                py,
                PyErr::fetch(py),
                "zero-copy",
                "mark unconsumed DLPack argument used",
            ));
        }
        entry._tensor.released = false;
    } else {
        return Err(dlpack_error("DLPack argument capsule name changed"));
    }
    drop(object);
    drop(entry);
    drop(capsule);
    super::super::foreign_object::drain_pending_releases(py);
    Ok(())
}

#[allow(unsafe_code)]
fn capsule_name<'a>(capsule: &'a Bound<'_, PyCapsule>) -> Result<&'a CStr, PythonError> {
    // SAFETY: attached bound capsule retains its NUL-terminated C name.
    let name = unsafe { ffi::PyCapsule_GetName(capsule.as_ptr()) };
    if name.is_null() {
        return Err(dlpack_error("DLPack consumer capsule has no name"));
    }
    // SAFETY: checked non-null; the capsule pins the name through this read.
    Ok(unsafe { CStr::from_ptr(name) })
}

#[allow(unsafe_code)]
unsafe extern "C" fn argument_capsule_destructor(capsule: *mut ffi::PyObject) {
    // SAFETY: CPython invokes this callback attached with a live capsule. Only
    // the original producer name authorizes release; consumed headers may be
    // freed, so used names exit before looking at their payload or deleter.
    let name = unsafe { ffi::PyCapsule_GetName(capsule) };
    if name.is_null() {
        return;
    }
    // SAFETY: CPython retains its NUL-terminated name for the callback duration.
    let name = unsafe { CStr::from_ptr(name) };
    if name != DLTENSOR_NAME && name != DLTENSOR_VERSIONED_NAME {
        return;
    }
    // SAFETY: name was checked against the producer state of this live capsule.
    let pointer = unsafe { ffi::PyCapsule_GetPointer(capsule, name.as_ptr()) };
    if let Ok(tensor) = ManagedTensor::from_capsule_name(pointer, name) {
        // SAFETY: this unconsumed capsule is the unique managed-header owner.
        unsafe { tensor.release() };
    }
}
