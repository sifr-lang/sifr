from __future__ import annotations

import ctypes
import importlib.metadata
import json
import sys


class ArrowSchema(ctypes.Structure):
    _fields_ = [
        ("format", ctypes.c_void_p),
        ("name", ctypes.c_void_p),
        ("metadata", ctypes.c_void_p),
        ("flags", ctypes.c_int64),
        ("n_children", ctypes.c_int64),
        ("children", ctypes.c_void_p),
        ("dictionary", ctypes.c_void_p),
        ("release", ctypes.c_void_p),
        ("private_data", ctypes.c_void_p),
    ]


class ArrowArray(ctypes.Structure):
    _fields_ = [
        ("length", ctypes.c_int64),
        ("null_count", ctypes.c_int64),
        ("offset", ctypes.c_int64),
        ("n_buffers", ctypes.c_int64),
        ("n_children", ctypes.c_int64),
        ("buffers", ctypes.c_void_p),
        ("children", ctypes.c_void_p),
        ("dictionary", ctypes.c_void_p),
        ("release", ctypes.c_void_p),
        ("private_data", ctypes.c_void_p),
    ]


class ArrowArrayStream(ctypes.Structure):
    _fields_ = [
        ("get_schema", ctypes.c_void_p),
        ("get_next", ctypes.c_void_p),
        ("get_last_error", ctypes.c_void_p),
        ("release", ctypes.c_void_p),
        ("private_data", ctypes.c_void_p),
    ]


CAPSULE_POINTER = ctypes.pythonapi.PyCapsule_GetPointer
CAPSULE_POINTER.argtypes = [ctypes.py_object, ctypes.c_char_p]
CAPSULE_POINTER.restype = ctypes.c_void_p
RELEASE = ctypes.CFUNCTYPE(None, ctypes.c_void_p)


def capsule_pointer(capsule: object, name: bytes) -> int:
    pointer = CAPSULE_POINTER(capsule, name)
    if not pointer:
        raise RuntimeError(f"{name.decode()} capsule had a null payload")
    return pointer


def release_once(capsule: object, name: bytes, structure: type[ctypes.Structure]) -> bool:
    first = capsule_pointer(capsule, name)
    second = capsule_pointer(capsule, name)
    if first != second:
        return False
    value = ctypes.cast(first, ctypes.POINTER(structure))
    callback = value.contents.release
    if not callback:
        return False
    RELEASE(callback)(first)
    return not value.contents.release


def producer(target: str) -> tuple[object, str]:
    if target == "pyarrow.array":
        import pyarrow

        return pyarrow.array([1, 2, 3]), "pyarrow"
    if target == "pandas.DataFrame":
        import pandas

        return pandas.DataFrame({"value": [1, 2, 3]}), "pandas"
    if target == "polars.Series":
        import polars

        return polars.Series("value", [1, 2, 3]), "polars"
    raise RuntimeError(f"unsupported Arrow certification target: {target}")


def verify_capsules(value: object) -> bool:
    if hasattr(value, "__arrow_c_array__"):
        schema, array = value.__arrow_c_array__()
        return release_once(schema, b"arrow_schema", ArrowSchema) and release_once(
            array, b"arrow_array", ArrowArray
        )
    stream = value.__arrow_c_stream__()
    return release_once(stream, b"arrow_array_stream", ArrowArrayStream)


def main() -> None:
    target = sys.argv[1]
    value, distribution = producer(target)
    pointer_identity_verified = verify_capsules(value)
    print(
        json.dumps(
            {
                "producer_module": type(value).__module__,
                "producer_type": type(value).__name__,
                "distributions": [
                    {
                        "name": distribution,
                        "version": importlib.metadata.version(distribution),
                    }
                ],
                "schema_mode": "omitted",
                "pointer_identity_verified": pointer_identity_verified,
                "exact_release_count": 1,
                "copy_performed": False,
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
