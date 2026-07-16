import ctypes


_buffer_acquisitions = 0
_buffer_releases = 0
_last_buffer_pointer = 0


class _ObservedBuffer:
    def __init__(self, payload: bytes):
        self.storage = bytearray(payload)

    def __buffer__(self, _flags: int):
        global _buffer_acquisitions, _last_buffer_pointer
        _buffer_acquisitions += 1
        _last_buffer_pointer = ctypes.addressof(ctypes.c_ubyte.from_buffer(self.storage))
        return memoryview(self.storage)

    def __release_buffer__(self, _view) -> None:
        global _buffer_releases
        _buffer_releases += 1


def reset_buffer_observer() -> None:
    global _buffer_acquisitions, _buffer_releases, _last_buffer_pointer
    _buffer_acquisitions = 0
    _buffer_releases = 0
    _last_buffer_pointer = 0


def buffer_acquisition_count() -> int:
    return _buffer_acquisitions


def buffer_release_count() -> int:
    return _buffer_releases


def last_buffer_pointer() -> int:
    return _last_buffer_pointer


def make_buffer(payload: bytes) -> _ObservedBuffer:
    return _ObservedBuffer(payload)
