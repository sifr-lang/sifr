from __future__ import annotations

import ctypes
import gc
import importlib.metadata
import json
import sys
from collections.abc import Callable


class ArrowSchema(ctypes.Structure):
    pass


class ArrowArray(ctypes.Structure):
    pass


ArrowSchema._fields_ = [
    ("format", ctypes.c_void_p),
    ("name", ctypes.c_void_p),
    ("metadata", ctypes.c_void_p),
    ("flags", ctypes.c_int64),
    ("n_children", ctypes.c_int64),
    ("children", ctypes.POINTER(ctypes.POINTER(ArrowSchema))),
    ("dictionary", ctypes.POINTER(ArrowSchema)),
    ("release", ctypes.c_void_p),
    ("private_data", ctypes.c_void_p),
]

ArrowArray._fields_ = [
    ("length", ctypes.c_int64),
    ("null_count", ctypes.c_int64),
    ("offset", ctypes.c_int64),
    ("n_buffers", ctypes.c_int64),
    ("n_children", ctypes.c_int64),
    ("buffers", ctypes.POINTER(ctypes.c_void_p)),
    ("children", ctypes.POINTER(ctypes.POINTER(ArrowArray))),
    ("dictionary", ctypes.POINTER(ArrowArray)),
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
GET_SCHEMA = ctypes.CFUNCTYPE(
    ctypes.c_int, ctypes.c_void_p, ctypes.POINTER(ArrowSchema)
)
GET_NEXT = ctypes.CFUNCTYPE(
    ctypes.c_int, ctypes.c_void_p, ctypes.POINTER(ArrowArray)
)


def capsule_pointer(capsule: object, name: bytes) -> int:
    pointer = CAPSULE_POINTER(capsule, name)
    if not pointer:
        raise RuntimeError(f"{name.decode()} capsule had a null payload")
    return pointer


def instrument_release(
    pointer: int, structure: type[ctypes.Structure]
) -> tuple[list[int], Callable[[int], None]]:
    value = ctypes.cast(pointer, ctypes.POINTER(structure))
    original_address = value.contents.release
    if not original_address:
        raise RuntimeError("Arrow capsule had no release callback")
    original = RELEASE(original_address)
    calls = [0]

    @RELEASE
    def counted(raw_pointer: int) -> None:
        calls[0] += 1
        original(raw_pointer)

    value.contents.release = ctypes.cast(counted, ctypes.c_void_p).value
    return calls, counted


def non_null_buffer_addresses(value: object) -> list[int]:
    return [buffer.address for buffer in value.buffers() if buffer is not None]


def verify_array(value: object, requested: bool) -> tuple[bool, bool]:
    expected = non_null_buffer_addresses(value)
    requested_calls: list[int] | None = None
    requested_callback: Callable[[int], None] | None = None
    if requested:
        import pyarrow

        requested_capsule = pyarrow.int64().__arrow_c_schema__()
        requested_pointer = capsule_pointer(requested_capsule, b"arrow_schema")
        requested_calls, requested_callback = instrument_release(
            requested_pointer, ArrowSchema
        )
        schema_capsule, array_capsule = value.__arrow_c_array__(requested_capsule)
    else:
        schema_capsule, array_capsule = value.__arrow_c_array__()
    schema_pointer = capsule_pointer(schema_capsule, b"arrow_schema")
    array_pointer = capsule_pointer(array_capsule, b"arrow_array")
    array = ctypes.cast(array_pointer, ctypes.POINTER(ArrowArray)).contents
    leaf = array.children[0].contents if array.n_children else array
    observed = [
        int(leaf.buffers[index])
        for index in range(leaf.n_buffers)
        if leaf.buffers[index]
    ]
    schema_calls, schema_callback = instrument_release(schema_pointer, ArrowSchema)
    array_calls, array_callback = instrument_release(array_pointer, ArrowArray)
    RELEASE(ctypes.cast(schema_pointer, ctypes.POINTER(ArrowSchema)).contents.release)(
        schema_pointer
    )
    RELEASE(ctypes.cast(array_pointer, ctypes.POINTER(ArrowArray)).contents.release)(
        array_pointer
    )
    gc.collect()
    _keepalive = (schema_callback, array_callback, requested_callback)
    requested_released = requested_calls is None or requested_calls == [1]
    return (
        expected == observed,
        schema_calls == [1] and array_calls == [1] and requested_released,
    )


def verify_schema(value: object) -> tuple[bool, bool]:
    capsule = value.__arrow_c_schema__()
    pointer = capsule_pointer(capsule, b"arrow_schema")
    schema = ctypes.cast(pointer, ctypes.POINTER(ArrowSchema))
    observed_format = ctypes.string_at(schema.contents.format)
    release_calls, release_callback = instrument_release(pointer, ArrowSchema)
    RELEASE(schema.contents.release)(pointer)
    _keepalive = release_callback
    return observed_format == b"l", release_calls == [1]


def verify_stream(value: object, source_address: int) -> tuple[bool, bool]:
    stream_capsule = value.__arrow_c_stream__()
    stream_pointer = capsule_pointer(stream_capsule, b"arrow_array_stream")
    stream = ctypes.cast(stream_pointer, ctypes.POINTER(ArrowArrayStream))
    release_calls, release_callback = instrument_release(
        stream_pointer, ArrowArrayStream
    )
    schema = ArrowSchema()
    array = ArrowArray()
    if GET_SCHEMA(stream.contents.get_schema)(stream_pointer, ctypes.byref(schema)) != 0:
        raise RuntimeError("Arrow stream get_schema failed")
    if GET_NEXT(stream.contents.get_next)(stream_pointer, ctypes.byref(array)) != 0:
        raise RuntimeError("Arrow stream get_next failed")
    leaf = array.children[0].contents if array.n_children else array
    observed = [
        int(leaf.buffers[index])
        for index in range(leaf.n_buffers)
        if leaf.buffers[index]
    ][-1]
    schema_calls, schema_callback = instrument_release(
        ctypes.addressof(schema), ArrowSchema
    )
    array_calls, array_callback = instrument_release(ctypes.addressof(array), ArrowArray)
    RELEASE(schema.release)(ctypes.addressof(schema))
    RELEASE(array.release)(ctypes.addressof(array))
    RELEASE(stream.contents.release)(stream_pointer)
    gc.collect()
    _keepalive = (release_callback, schema_callback, array_callback)
    releases_verified = (
        release_calls == [1] and schema_calls == [1] and array_calls == [1]
    )
    return source_address == observed, releases_verified


def producer(target: str) -> tuple[object, str, str, str, str, int | None]:
    if target == "pyarrow.array":
        import pyarrow

        return (
            pyarrow.array([1, 2, 3]),
            "pyarrow",
            "array",
            "parameter",
            "buffer_address",
            None,
        )
    if target == "pyarrow.int64":
        import pyarrow

        return (
            pyarrow.int64(),
            "pyarrow",
            "schema",
            "omitted",
            "schema_format",
            None,
        )
    if target == "pandas.DataFrame":
        import pandas

        value = pandas.DataFrame({"value": [1, 2, 3]})
        address = int(value["value"].to_numpy(copy=False).__array_interface__["data"][0])
        return value, "pandas", "stream", "omitted", "buffer_address", address
    if target == "polars.Series":
        import polars

        value = polars.Series("value", [1, 2, 3])
        address = non_null_buffer_addresses(value.to_arrow())[-1]
        return value, "polars", "stream", "omitted", "buffer_address", address
    raise RuntimeError(f"unsupported Arrow certification target: {target}")


def main() -> None:
    target = sys.argv[1]
    value, distribution, kind, schema_mode, identity_method, source_address = producer(target)
    if kind == "array":
        pointer_identity_verified, release_verified = verify_array(
            value, schema_mode == "parameter"
        )
    elif kind == "schema":
        pointer_identity_verified, release_verified = verify_schema(value)
    else:
        if source_address is None:
            raise RuntimeError("stream certification requires a source buffer address")
        pointer_identity_verified, release_verified = verify_stream(value, source_address)
    print(
        json.dumps(
            {
                "target": target,
                "kind": kind,
                "identity_method": identity_method,
                "producer_module": type(value).__module__,
                "producer_type": type(value).__name__,
                "distributions": [
                    {
                        "name": distribution,
                        "version": importlib.metadata.version(distribution),
                    }
                ],
                "schema_mode": schema_mode,
                "pointer_identity_verified": pointer_identity_verified,
                "exact_release_count": 1 if release_verified else 0,
                "copy_performed": not pointer_identity_verified,
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
