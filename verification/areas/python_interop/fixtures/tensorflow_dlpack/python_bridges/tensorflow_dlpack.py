import ctypes

import tensorflow as tf


class DLDevice(ctypes.Structure):
    _fields_ = [("device_type", ctypes.c_int32), ("device_id", ctypes.c_int32)]


class DLDataType(ctypes.Structure):
    _fields_ = [
        ("code", ctypes.c_uint8),
        ("bits", ctypes.c_uint8),
        ("lanes", ctypes.c_uint16),
    ]


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


class DLManagedTensor(ctypes.Structure):
    _fields_ = [
        ("dl_tensor", DLTensor),
        ("manager_ctx", ctypes.c_void_p),
        ("deleter", ctypes.c_void_p),
    ]


CAPSULE_POINTER = ctypes.pythonapi.PyCapsule_GetPointer
CAPSULE_POINTER.argtypes = [ctypes.py_object, ctypes.c_char_p]
CAPSULE_POINTER.restype = ctypes.c_void_p
LAST_SOURCE_POINTER: int | None = None


def capsule_data_pointer(capsule: object) -> int:
    pointer = CAPSULE_POINTER(capsule, b"dltensor")
    if not pointer:
        raise RuntimeError("TensorFlow returned a null DLPack capsule")
    managed = ctypes.cast(pointer, ctypes.POINTER(DLManagedTensor)).contents
    if not managed.dl_tensor.data:
        raise RuntimeError("TensorFlow returned a null DLPack data pointer")
    return int(managed.dl_tensor.data) + int(managed.dl_tensor.byte_offset)


class Exporter:
    def __init__(self, values: list[int]) -> None:
        self.tensor = tf.constant(values, dtype=tf.int64)

    def data_pointer(self) -> int:
        return capsule_data_pointer(tf.experimental.dlpack.to_dlpack(self.tensor))

    def __dlpack_device__(self) -> tuple[int, int]:
        return (1, 0)

    def __dlpack__(
        self,
        *,
        stream: int | None,
        max_version: tuple[int, int],
        copy: bool,
    ) -> object:
        if stream is not None:
            raise RuntimeError("CPU TensorFlow export requires stream=None")
        if max_version != (1, 0) or copy is not False:
            raise RuntimeError("TensorFlow bridge requires max_version=(1, 0), copy=False")
        return tf.experimental.dlpack.to_dlpack(self.tensor)


def make(values: list[int]) -> Exporter:
    global LAST_SOURCE_POINTER
    exporter = Exporter(values)
    LAST_SOURCE_POINTER = exporter.data_pointer()
    return exporter


def consume(capsule: object) -> tf.Tensor:
    return tf.experimental.dlpack.from_dlpack(capsule)


def pointer_stable(tensor: tf.Tensor) -> bool:
    if LAST_SOURCE_POINTER is None:
        raise RuntimeError("TensorFlow bridge has no source pointer evidence")
    observed = capsule_data_pointer(tf.experimental.dlpack.to_dlpack(tensor))
    return LAST_SOURCE_POINTER == observed


def sum_int(tensor: tf.Tensor) -> int:
    return int(tf.reduce_sum(tensor).numpy())
