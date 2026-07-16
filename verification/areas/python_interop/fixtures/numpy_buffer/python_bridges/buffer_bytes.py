import ctypes


_buffers: list[bytearray] = []


def reset_buffer_observer() -> None:
    _buffers.clear()


def buffer_count() -> int:
    return len(_buffers)


def all_buffers_resizable_after_release() -> bool:
    try:
        for buffer in _buffers:
            buffer.append(0)
            buffer.pop()
    except BufferError:
        return False
    return True


def last_buffer_pointer() -> int:
    return ctypes.addressof(ctypes.c_ubyte.from_buffer(_buffers[-1]))


def last_buffer_value(index: int) -> int:
    return _buffers[-1][index]


def make_buffer(payload: bytes) -> bytearray:
    buffer = bytearray(payload)
    _buffers.append(buffer)
    return buffer
