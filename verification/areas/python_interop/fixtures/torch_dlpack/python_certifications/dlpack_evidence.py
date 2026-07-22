import ctypes
import gc
import importlib.metadata
import json
import sys

import torch


CAPSULE_NAME = ctypes.pythonapi.PyCapsule_GetName
CAPSULE_NAME.argtypes = [ctypes.py_object]
CAPSULE_NAME.restype = ctypes.c_char_p
CAPSULE_POINTER = ctypes.pythonapi.PyCapsule_GetPointer
CAPSULE_POINTER.argtypes = [ctypes.py_object, ctypes.c_char_p]
CAPSULE_POINTER.restype = ctypes.c_void_p


class DLPackVersion(ctypes.Structure):
    _fields_ = [("major", ctypes.c_uint32), ("minor", ctypes.c_uint32)]


class DLDevice(ctypes.Structure):
    _fields_ = [("device_type", ctypes.c_int32), ("device_id", ctypes.c_int32)]


class DLDataType(ctypes.Structure):
    _fields_ = [("code", ctypes.c_uint8), ("bits", ctypes.c_uint8), ("lanes", ctypes.c_uint16)]


class DLTensor(ctypes.Structure):
    _fields_ = [
        ("data", ctypes.c_void_p),
        ("device", DLDevice),
        ("ndim", ctypes.c_int32),
        ("dtype", DLDataType),
        ("shape", ctypes.POINTER(ctypes.c_int64)),
        ("strides", ctypes.POINTER(ctypes.c_int64)),
        ("byte_offset", ctypes.c_uint64),
    ]


class DLManagedTensorVersioned(ctypes.Structure):
    pass


DELETER = ctypes.CFUNCTYPE(None, ctypes.POINTER(DLManagedTensorVersioned))
DLManagedTensorVersioned._fields_ = [
    ("version", DLPackVersion),
    ("manager_ctx", ctypes.c_void_p),
    ("deleter", ctypes.c_void_p),
    ("flags", ctypes.c_uint64),
    ("dl_tensor", DLTensor),
]


def count_deleter(capsule, counter):
    pointer = CAPSULE_POINTER(capsule, b"dltensor_versioned")
    managed = ctypes.cast(pointer, ctypes.POINTER(DLManagedTensorVersioned))
    original = DELETER(managed.contents.deleter)

    @DELETER
    def counted(value):
        counter.append(1)
        original(value)

    managed.contents.deleter = ctypes.cast(counted, ctypes.c_void_p).value
    return counted, original


def main() -> None:
    target = sys.argv[1]
    if target != "torch.Tensor":
        raise RuntimeError(f"unexpected certification target: {target}")
    source = torch.tensor([3, 5, 8], dtype=torch.int64)
    producer_module = type(source).__module__
    producer_type = type(source).__qualname__
    source_pointer = int(source.data_ptr())
    capsule = source.__dlpack__(max_version=(1, 0), copy=False)
    before = CAPSULE_NAME(capsule)
    if before not in {b"dltensor", b"dltensor_versioned"}:
        raise RuntimeError(f"unexpected fresh capsule name: {before!r}")
    if before != b"dltensor_versioned":
        raise RuntimeError("PyTorch did not honor the requested versioned DLPack contract")
    deleter_calls = []
    deleter_guards = count_deleter(capsule, deleter_calls)
    consumed = torch.from_dlpack(capsule)
    after = CAPSULE_NAME(capsule)
    if after not in {b"used_dltensor", b"used_dltensor_versioned"}:
        raise RuntimeError(f"capsule was not consumed exactly once: {after!r}")
    if int(consumed.data_ptr()) != source_pointer:
        raise RuntimeError("DLPack consumer did not preserve the source pointer")
    if consumed.tolist() != [3, 5, 8]:
        raise RuntimeError("DLPack consumer returned unexpected values")
    del source
    del consumed
    del capsule
    gc.collect()
    if len(deleter_calls) != 1:
        raise RuntimeError(f"DLPack deleter ran {len(deleter_calls)} times instead of exactly once")
    del deleter_guards
    print(
        json.dumps(
            {
                "target": target,
                "producer_module": producer_module,
                "producer_type": producer_type,
                "distributions": [
                    {"name": "torch", "version": importlib.metadata.version("torch")}
                ],
                "device": "cpu",
                "stream_policy": "none",
                "pointer_identity_verified": True,
                "exact_deleter_count": len(deleter_calls),
                "copy_performed": False,
                "within_run_assertions": True,
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
