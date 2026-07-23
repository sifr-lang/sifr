use super::abi::{ManagedTensor, DLTENSOR_NAME, DLTENSOR_VERSIONED_NAME};
use super::{closed_error, dlpack_error, dlpack_store, DlpackEntry, DlpackHandle, PythonError};
use crate::python::ObjectHandle;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::PyCapsule;
use std::ffi::CStr;

pub struct PythonDlpackArgument {
    entry: Option<DlpackEntry>,
    object: Option<ObjectHandle>,
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
        let object = self.object.take();
        super::super::attach(move |py| finalize(py, entry, object)).map_err(PythonError::runtime)?
    }
}

impl Drop for PythonDlpackArgument {
    fn drop(&mut self) {
        let Some(entry) = self.entry.take() else {
            return;
        };
        let object = self.object.take();
        let _ignored = super::super::attach(move |py| finalize(py, entry, object));
    }
}

pub fn prepare_dlpack_argument(handle: DlpackHandle) -> Result<PythonDlpackArgument, PythonError> {
    let entry = {
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
    let (entry, object) = super::super::attach(move |py| {
        let mut entry = entry;
        let object = argument_capsule(py, &mut entry)?;
        Ok((entry, object))
    })
    .map_err(PythonError::runtime)??;
    Ok(PythonDlpackArgument {
        entry: Some(entry),
        object: Some(object),
    })
}

fn argument_capsule(py: Python<'_>, entry: &mut DlpackEntry) -> Result<ObjectHandle, PythonError> {
    let tensor = entry._tensor.tensor;
    let pointer = std::ptr::NonNull::new(tensor.pointer())
        .ok_or_else(|| dlpack_error("DLPack argument pointer is null"))?;
    let capsule = unsafe {
        PyCapsule::new_with_pointer_and_destructor(
            py,
            pointer,
            tensor.capsule_name(),
            Some(argument_capsule_destructor),
        )
    }
    .map_err(|error| PythonError::from_pyerr(py, error, "zero-copy", "DLPack argument capsule"))?;
    // Once the capsule exists, its destructor owns the tensor if handle
    // attachment fails. Temporarily disarm the entry so that failure cannot
    // invoke the deleter a second time; successful attachment restores the
    // entry/capsule reconciliation protocol used by `finalize`.
    entry._tensor.released = true;
    match super::super::object_ops::store_object(capsule.into_any().unbind()) {
        Ok(object) => {
            entry._tensor.released = false;
            Ok(object)
        }
        Err(error) => Err(error),
    }
}

fn finalize(
    py: Python<'_>,
    mut entry: DlpackEntry,
    object: Option<ObjectHandle>,
) -> Result<(), PythonError> {
    let Some(object) = object else {
        drop(entry);
        return Ok(());
    };
    let capsule = match super::super::object_ops::clone_handle(py, &object) {
        Ok(capsule) => capsule,
        Err(error) => return relinquish_to_capsule(py, entry, object, error),
    };
    let capsule = match capsule.bind(py).cast::<PyCapsule>() {
        Ok(capsule) => capsule,
        Err(_) => {
            let error = dlpack_error("generated DLPack consumer argument is not a PyCapsule");
            return relinquish_to_capsule(py, entry, object, error);
        }
    };
    let name = match capsule_name(capsule) {
        Ok(name) => name,
        Err(error) => return relinquish_to_capsule(py, entry, object, error),
    };
    let tensor = entry._tensor.tensor;
    if name == tensor.used_capsule_name() {
        entry._tensor.released = true;
    } else {
        let rename = unsafe {
            ffi::PyCapsule_SetName(capsule.as_ptr(), tensor.used_capsule_name().as_ptr())
        };
        if rename != 0 {
            let error = PythonError::from_pyerr(
                py,
                PyErr::fetch(py),
                "zero-copy",
                "mark unconsumed DLPack argument used",
            );
            // The capsule still has its original name, so its destructor remains
            // the sole owner of the managed-tensor deleter. Relinquish the entry
            // before either Python reference can be dropped.
            entry._tensor.released = true;
            drop(object);
            drop(entry);
            return Err(error);
        }
    }
    drop(object);
    drop(entry);
    super::super::foreign_object::drain_pending_releases(py);
    Ok(())
}

fn relinquish_to_capsule(
    py: Python<'_>,
    mut entry: DlpackEntry,
    object: ObjectHandle,
    error: PythonError,
) -> Result<(), PythonError> {
    // The compiler-created capsule still has its producer name. Its destructor
    // is therefore the sole deleter owner when an internal handle invariant
    // prevents reconciliation from inspecting or renaming the capsule.
    entry._tensor.released = true;
    drop(object);
    drop(entry);
    super::super::foreign_object::drain_pending_releases(py);
    Err(error)
}

fn capsule_name<'a>(capsule: &'a Bound<'_, PyCapsule>) -> Result<&'a CStr, PythonError> {
    let name = unsafe { ffi::PyCapsule_GetName(capsule.as_ptr()) };
    if name.is_null() {
        return Err(dlpack_error("DLPack consumer capsule has no name"));
    }
    Ok(unsafe { CStr::from_ptr(name) })
}

unsafe extern "C" fn argument_capsule_destructor(capsule: *mut ffi::PyObject) {
    let name = unsafe { ffi::PyCapsule_GetName(capsule) };
    if name.is_null() {
        return;
    }
    let name = unsafe { CStr::from_ptr(name) };
    if name != DLTENSOR_NAME && name != DLTENSOR_VERSIONED_NAME {
        return;
    }
    let pointer = unsafe { ffi::PyCapsule_GetPointer(capsule, name.as_ptr()) };
    if let Ok(tensor) = ManagedTensor::from_capsule_name(pointer, name) {
        unsafe { tensor.release() };
    }
}
